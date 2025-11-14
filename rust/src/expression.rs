use crate::scanner::Token;
use std::fmt::Display;

pub enum Expression {
    Literal(Literal),
    Unary(Unary),
    Binary(Binary),
    Grouping(Grouping),
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Binary(expr) => write!(f, "{expr}"),
            Expression::Literal(expr) => write!(f, "{expr}"),
            Expression::Unary(expr) => write!(f, "{expr}"),
            Expression::Grouping(expr) => write!(f, "{expr}"),
        }
    }
}

pub struct Grouping(Box<Expression>);

impl Display for Grouping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(group {})", self.0)
    }
}

pub enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Nil => write!(f, "nil"),
            Literal::Boolean(v) => write!(f, "{v}"),
            Literal::Number(n) => write!(f, "{n}"),
            Literal::String(s) => write!(f, "{s}"),
        }
    }
}

pub struct Unary {
    operator: Token,
    right: Box<Expression>,
}

impl Display for Unary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {})", self.operator.lexeme, self.right,)
    }
}

pub struct Binary {
    left: Box<Expression>,
    operator: Token,
    right: Box<Expression>,
}

impl Display for Binary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {} {})", self.operator.lexeme, self.left, self.right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::{TokenLiteral, TokenType};

    #[test]
    fn display_simple() {
        let expr = Expression::Binary(Binary {
            left: Box::new(Expression::Literal(Literal::Number(1 as f64))),
            operator: Token {
                token_type: TokenType::Plus,
                lexeme: "+".to_string(),
                literal: TokenLiteral::None,
                line: 1,
            },
            right: Box::new(Expression::Literal(Literal::Number(2 as f64))),
        });

        assert_eq!("(+ 1 2)", format!("{expr}").as_str())
    }

    #[test]
    fn display_complex() {
        let expr = Expression::Binary(Binary {
            left: Box::new(Expression::Unary(Unary {
                operator: Token {
                    token_type: TokenType::Minus,
                    lexeme: "-".to_string(),
                    literal: TokenLiteral::None,
                    line: 1,
                },
                right: Box::new(Expression::Literal(Literal::Number(123 as f64))),
            })),
            operator: Token {
                token_type: TokenType::Star,
                lexeme: "*".to_string(),
                literal: TokenLiteral::None,
                line: 1,
            },
            right: Box::new(Expression::Grouping(Grouping(Box::new(
                Expression::Literal(Literal::Number(45.67)),
            )))),
        });

        assert_eq!("(* (- 123) (group 45.67))", format!("{expr}").as_str())
    }
}
