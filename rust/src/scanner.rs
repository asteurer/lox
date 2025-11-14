use crate::error::lox_error;
use std::fmt::Display;

#[derive(Clone, PartialEq, Debug)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Identifier,
    String,
    Number,
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    EOF,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::LeftParen => write!(f, "LeftParen"),
            TokenType::RightParen => write!(f, "RightParen"),
            TokenType::LeftBrace => write!(f, "LeftBrace"),
            TokenType::RightBrace => write!(f, "RightBrace"),
            TokenType::Comma => write!(f, "Comma"),
            TokenType::Dot => write!(f, "Dot"),
            TokenType::Minus => write!(f, "Minus"),
            TokenType::Plus => write!(f, "Plus"),
            TokenType::Semicolon => write!(f, "Semicolon"),
            TokenType::Slash => write!(f, "Slash"),
            TokenType::Star => write!(f, "Star"),
            TokenType::Bang => write!(f, "Bang"),
            TokenType::BangEqual => write!(f, "BangEqual"),
            TokenType::Equal => write!(f, "Equal"),
            TokenType::EqualEqual => write!(f, "EqualEqual"),
            TokenType::Greater => write!(f, "Greater"),
            TokenType::GreaterEqual => write!(f, "GreaterEqual"),
            TokenType::Less => write!(f, "Less"),
            TokenType::LessEqual => write!(f, "LessEqual"),
            TokenType::Identifier => write!(f, "Identifier"),
            TokenType::String => write!(f, "String"),
            TokenType::Number => write!(f, "Number"),
            TokenType::And => write!(f, "And"),
            TokenType::Class => write!(f, "Class"),
            TokenType::Else => write!(f, "Else"),
            TokenType::False => write!(f, "False"),
            TokenType::Fun => write!(f, "Fun"),
            TokenType::For => write!(f, "For"),
            TokenType::If => write!(f, "If"),
            TokenType::Nil => write!(f, "Nil"),
            TokenType::Or => write!(f, "Or"),
            TokenType::Print => write!(f, "Print"),
            TokenType::Return => write!(f, "Return"),
            TokenType::Super => write!(f, "Super"),
            TokenType::This => write!(f, "This"),
            TokenType::True => write!(f, "True"),
            TokenType::Var => write!(f, "Var"),
            TokenType::While => write!(f, "While"),
            TokenType::EOF => write!(f, "EOF"),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum TokenLiteral {
    Str(String),
    Num(f64),
    None,
}

impl Display for TokenLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenLiteral::Str(s) => write!(f, "'{}'", s),
            TokenLiteral::Num(n) => write!(f, "'{}'", n),
            TokenLiteral::None => write!(f, "None"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: TokenLiteral,
    #[allow(dead_code)] // TODO
    pub line: usize,
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.lexeme == other.lexeme
            && self.line == other.line
            && self.literal == other.literal
            && self.token_type == other.token_type
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "type: '{}', lexeme: '{}', literal: {}",
            self.token_type, self.lexeme, self.literal
        )
    }
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, literal: TokenLiteral, line: usize) -> Self {
        Self {
            token_type,
            lexeme,
            line,
            literal,
        }
    }
}

pub struct Scanner {
    source: String,
    start: usize,
    current: usize,
    line: usize,
    tokens: Vec<Token>,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source,
            start: 0,
            current: 0,
            line: 1,
            tokens: vec![],
        }
    }

    /// Scans the tokens of a source string
    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.push(Token::new(
            TokenType::EOF,
            String::new(),
            TokenLiteral::None,
            self.line,
        ));
        self.tokens.clone()
    }

    /// Indicates whether the scanner is at the end of a line
    fn is_at_end(&self) -> bool {
        return self.current >= self.source.len();
    }

    /// Scans a token
    fn scan_token(&mut self) {
        match self.advance() {
            '\n' => self.line += 1,
            ' ' | '\r' | '\t' => (), // Ignore whitespace
            '(' => self.add_token(TokenType::LeftParen, None),
            ')' => self.add_token(TokenType::RightParen, None),
            '{' => self.add_token(TokenType::LeftBrace, None),
            '}' => self.add_token(TokenType::RightBrace, None),
            ',' => self.add_token(TokenType::Comma, None),
            '.' => self.add_token(TokenType::Dot, None),
            '-' => self.add_token(TokenType::Minus, None),
            '+' => self.add_token(TokenType::Plus, None),
            ';' => self.add_token(TokenType::Semicolon, None),
            '*' => self.add_token(TokenType::Star, None),
            '!' => {
                let token = if self.match_token('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.add_token(token, None);
            }
            '=' => {
                let token = if self.match_token('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.add_token(token, None);
            }
            '<' => {
                let token = if self.match_token('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.add_token(token, None);
            }
            '>' => {
                let token = if self.match_token('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.add_token(token, None);
            }
            '/' => {
                // If it's a comment, advance to the end of the line
                if self.match_token('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                // Block comments (e.g. /*...*/)
                } else if self.match_token('*') {
                    loop {
                        self.advance();

                        if self.is_at_end() {
                            break;
                        } else if self.peek() == '*' && self.peek_next() == '/' {
                            self.advance();
                            self.advance();
                            break;
                        } else if self.peek() == '\n' {
                            self.line += 1;
                        }
                    }
                } else {
                    self.add_token(TokenType::Slash, None);
                }
            }
            '"' => {
                self.parse_string();
            }
            character => {
                if character.is_ascii_digit() {
                    self.number();
                } else if self.is_valid_identifier_char(character) {
                    self.identifier();
                } else {
                    lox_error(self.line, format!("Unexpected character: {character}"));
                }
            }
        }
    }

    /// Returns the current character and increments a pointer to the next character
    fn advance(&mut self) -> char {
        let ch = self.source.chars().nth(self.current).unwrap();
        self.current += 1;
        ch
    }

    /// Parses a portion of the source string into a token and appends it to the list of tokens
    fn add_token(&mut self, token_type: TokenType, literal: Option<TokenLiteral>) {
        let text = self.source[self.start..self.current].to_string();
        if let Some(v) = literal {
            self.tokens.push(Token::new(token_type, text, v, self.line));
        } else {
            self.tokens
                .push(Token::new(token_type, text, TokenLiteral::None, self.line));
        }
    }

    /// Indicates whether `expected` matches the current token and increments a pointer to the next character
    /// if it does.
    fn match_token(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.chars().nth(self.current).unwrap() != expected {
            return false;
        };

        self.current += 1;
        return true;
    }

    /// Looks at the current character without advancing the pointer
    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source.chars().nth(self.current).unwrap()
    }

    /// Looks at the next character without advancing the pointer
    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }
        self.source.chars().nth(self.current + 1).unwrap()
    }

    /// Parses a value surrounded by `""`
    fn parse_string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            lox_error(self.line, "Unterminated string.".to_string());
            return;
        }

        self.advance();

        // Trim surrounding quotes
        let literal = self.source[self.start + 1..self.current - 1].to_string();

        // This bypasses the `add_token` method, which will captures the quotes that surround a string,
        // resulting in something like `"\"Hello, world!"\"`, which is not ideal.
        self.tokens.push(Token::new(
            TokenType::String,
            literal.to_owned(),
            TokenLiteral::Str(literal),
            self.line,
        ));
    }

    /// Parses a float or integer value
    fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();
        }

        while self.peek().is_ascii_digit() {
            self.advance();
        }

        let number: f64 = match self.source[self.start..self.current].parse() {
            Ok(n) => n,
            Err(e) => {
                lox_error(self.line, format!("Unable to parse number: {}", e));
                return;
            }
        };

        self.add_token(TokenType::Number, Some(TokenLiteral::Num(number)));
    }

    /// Indicates whether a character is valid for an identifier
    fn is_valid_identifier_char(&self, c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }

    /// Parses an identifier
    fn identifier(&mut self) {
        while self.is_valid_identifier_char(self.peek()) {
            self.advance();
        }

        let literal = self.source[self.start..self.current].to_string();
        let token_type = match literal.as_str() {
            "and" => TokenType::And,
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "false" => TokenType::False,
            "for" => TokenType::For,
            "fun" => TokenType::Fun,
            "if" => TokenType::If,
            "nil" => TokenType::Nil,
            "or" => TokenType::Or,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "this" => TokenType::This,
            "true" => TokenType::True,
            "var" => TokenType::Var,
            "while" => TokenType::While,
            _ => TokenType::Identifier,
        };

        match token_type {
            TokenType::Identifier => self.add_token(token_type, Some(TokenLiteral::Str(literal))),
            _ => self.add_token(token_type, None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! new_token {
        ($token_type:ident, $lexeme:expr, $token_literal:expr) => {
            Token::new(
                TokenType::$token_type,
                $lexeme.to_string(),
                $token_literal,
                1 as usize,
            )
        };
        ($token_type:ident, $lexeme:expr) => {
            Token::new(
                TokenType::$token_type,
                $lexeme.to_string(),
                TokenLiteral::None,
                1usize,
            )
        };
    }

    #[test]
    fn basic() {
        let mut scanner = Scanner::new(String::from(
            "() {} , . - + ; / * ! != = == > >= < <= this_is_an_identifier \"Hello, world!\" 123.456 and class else false fun for if nil or print return super this true var while",
        ));
        let tokens = scanner.scan_tokens();
        let expected_tokens = vec![
            new_token!(LeftParen, "("),
            new_token!(RightParen, ")"),
            new_token!(LeftBrace, "{"),
            new_token!(RightBrace, "}"),
            new_token!(Comma, ","),
            new_token!(Dot, "."),
            new_token!(Minus, "-"),
            new_token!(Plus, "+"),
            new_token!(Semicolon, ";"),
            new_token!(Slash, "/"),
            new_token!(Star, "*"),
            new_token!(Bang, "!"),
            new_token!(BangEqual, "!="),
            new_token!(Equal, "="),
            new_token!(EqualEqual, "=="),
            new_token!(Greater, ">"),
            new_token!(GreaterEqual, ">="),
            new_token!(Less, "<"),
            new_token!(LessEqual, "<="),
            new_token!(
                Identifier,
                "this_is_an_identifier",
                TokenLiteral::Str("this_is_an_identifier".to_string())
            ),
            new_token!(
                String,
                "Hello, world!",
                TokenLiteral::Str("Hello, world!".to_string())
            ),
            new_token!(Number, "123.456", TokenLiteral::Num(123.456)),
            new_token!(And, "and"),
            new_token!(Class, "class"),
            new_token!(Else, "else"),
            new_token!(False, "false"),
            new_token!(Fun, "fun"),
            new_token!(For, "for"),
            new_token!(If, "if"),
            new_token!(Nil, "nil"),
            new_token!(Or, "or"),
            new_token!(Print, "print"),
            new_token!(Return, "return"),
            new_token!(Super, "super"),
            new_token!(This, "this"),
            new_token!(True, "true"),
            new_token!(Var, "var"),
            new_token!(While, "while"),
            new_token!(EOF, ""),
        ];

        for (idx, t) in tokens.iter().enumerate() {
            let e = &expected_tokens[idx];
            assert_eq!(t, e);
        }
    }
}
