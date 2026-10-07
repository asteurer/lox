use crate::lox::token::{Token, TokenType};
use anyhow::{Error, anyhow};
use std::{collections::BTreeMap, iter::Peekable, str::CharIndices};

fn reserved_keywords() -> BTreeMap<&'static str, TokenType> {
    use TokenType as tt;
    BTreeMap::from([
        ("and", tt::And),
        ("class", tt::Class),
        ("else", tt::Else),
        ("false", tt::False),
        ("for", tt::For),
        ("fun", tt::Fun),
        ("if", tt::If),
        ("nil", tt::Nil),
        ("or", tt::Or),
        ("print", tt::Print),
        ("return", tt::Return),
        ("super", tt::Super),
        ("this", tt::This),
        ("true", tt::True),
        ("var", tt::Var),
        ("while", tt::While),
    ])
}

pub enum ScannerError {
    UnexpectedChar { ch: char, at: usize },
    UnterminatedString { at: usize },
}

impl std::fmt::Debug for ScannerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScannerError::UnexpectedChar { ch, at } => {
                write!(f, "Unexpected character '{ch}' at {at}")
            }
            ScannerError::UnterminatedString { at } => write!(f, "Unterminated string at {at}"),
        }
    }
}

impl From<ScannerError> for Error {
    fn from(value: ScannerError) -> Self {
        anyhow!("{value:?}")
    }
}

#[must_use = "scanners are lazy and do nothing unless consumed"]
pub struct Scanner<'a> {
    source: &'a str,
    chars: Peekable<CharIndices<'a>>,
    line: usize,
    done: bool,
}

impl std::iter::FusedIterator for Scanner<'_> {}

impl<'a> Scanner<'a> {
    /// An iterator over the tokens in a Lox source string.
    pub fn scan(source: &'a str) -> impl Iterator<Item = Result<Token, ScannerError>> {
        Self {
            source,
            chars: source.char_indices().peekable(),
            line: 1,
            done: false,
        }
    }

    /// Consume chars while `pred` is true; return the byte index once `pred` is false.
    fn eat_while(&mut self, pred: impl Fn(char) -> bool) -> usize {
        while self.chars.next_if(|(_, c)| pred(*c)).is_some() {}
        self.chars.peek().map_or(self.source.len(), |(i, _)| *i)
    }

    // If the next character matches what's expected, advance the iterator and return true
    fn match_eat(&mut self, expected: &char) -> bool {
        if let Some((_, next)) = self.chars.peek()
            && next.eq(expected)
        {
            // It matches what's expected, so we consume it
            self.chars.next();
            true
        } else {
            false
        }
    }
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Result<Token, ScannerError>;

    fn next(&mut self) -> Option<Self::Item> {
        use TokenType as tt;

        let (start, c) = loop {
            let (i, c) = match self.chars.next() {
                Some((i, c)) => (i, c),
                // Iterator is exhausted when marked as `done`
                None if self.done => return None,
                // When EOF reached, return EOF token and mark as `done`
                None => {
                    self.done = true;
                    return Some(Ok(Token::new(tt::EOF, self.line)));
                }
            };
            match c {
                // Consume comments
                '/' if self.chars.peek().is_some_and(|(_, n)| *n == '/') => {
                    self.eat_while(|c| c != '\n');
                }
                // Track + consume newlines
                '\n' => self.line += 1,
                // Consume whitespace
                c if c.is_whitespace() => {}
                _ => break (i, c),
            }
        };

        let tok_ty = match c {
            '(' => tt::LeftParen,
            ')' => tt::RightParen,
            '{' => tt::LeftBrace,
            '}' => tt::RightBrace,
            ',' => tt::Comma,
            '.' => tt::Dot,
            '-' => tt::Minus,
            '+' => tt::Plus,
            ';' => tt::Semicolon,
            '/' => tt::Slash,
            '*' => tt::Star,
            '!' => {
                if self.match_eat(&'=') {
                    tt::BangEqual
                } else {
                    tt::Bang
                }
            }
            '=' => {
                if self.match_eat(&'=') {
                    tt::EqualEqual
                } else {
                    tt::Equal
                }
            }
            '<' => {
                if self.match_eat(&'=') {
                    tt::LessEqual
                } else {
                    tt::Less
                }
            }
            '>' => {
                if self.match_eat(&'=') {
                    tt::GreaterEqual
                } else {
                    tt::Greater
                }
            }
            '"' => {
                let mut str = String::new();
                loop {
                    if let Some((_, next)) = self.chars.peek() {
                        if next.eq(&'"') {
                            // Consume the terminating quote
                            self.chars.next();
                            break;
                        } else if next.eq(&'\n') {
                            self.line += 1;
                        }
                        str.push(*next);
                    } else {
                        return Some(Err(ScannerError::UnterminatedString { at: start }));
                    }

                    // Advance the iterator
                    self.chars.next();
                }

                tt::Str(str)
            }
            digit if c.is_digit(10) => {
                let ends_with_dot = |num: &[char]| num.last().unwrap().eq(&'.');
                let mut num = vec![];
                num.push(digit);

                while let Some((i, next)) = self.chars.peek() {
                    if next.is_digit(10) {
                        num.push(*next);
                        self.chars.next();
                    } else if next.eq(&'.') && !ends_with_dot(&num) {
                        num.push('.');
                        self.chars.next();
                    } else {
                        // Guards against `123..456`
                        if ends_with_dot(&num) {
                            return Some(Err(ScannerError::UnexpectedChar { ch: *next, at: *i }));
                        }
                        break;
                    }
                }

                // TODO: Guard against cases like `123.`

                let num = num.iter().collect::<String>();
                match num.parse::<f64>() {
                    Ok(n) => return Some(Ok(Token::new(tt::Num(n), self.line))),
                    Err(e) => {
                        panic!("There's a bug in the scanner: {e}");
                    }
                }
            }
            alpha if c.is_alphabetic() => {
                let mut ident = String::new();
                ident.push(alpha);

                while let Some((_, next)) = self.chars.peek() {
                    if next.is_alphanumeric() || next.eq(&'_') {
                        ident.push(*next);
                        self.chars.next();
                    } else {
                        break;
                    }
                }

                let tt = if let Some(reserved) = reserved_keywords().get(ident.as_str()) {
                    reserved.clone()
                } else {
                    TokenType::Ident(ident)
                };

                return Some(Ok(Token::new(tt, self.line)));
            }
            other => {
                return Some(Err(ScannerError::UnexpectedChar {
                    ch: other,
                    at: start,
                }));
            }
        };

        Some(Ok(Token::new(tok_ty, self.line)))
    }
}
