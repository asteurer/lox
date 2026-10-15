mod ast;
mod interpreter;
mod parser;
mod scanner;
mod token;

use anyhow::Result;
use scanner::Scanner;

use crate::lox::{ast::Expr, parser::Parser, scanner::ScannerError};
use token::{Token, TokenType};

pub struct Lox;
impl Lox {
    pub fn run(line: String) -> Result<()> {
        let tokens = Scanner::scan(&line).collect::<Result<Vec<Token>, ScannerError>>()?;
        println!("{tokens:?}");
        let mut parser = Parser::new(&tokens);
        let expr = parser.parse()?;
        println!("{}", Expr::parenthesize(&expr));
        let val = interpreter::evaluate(&expr)?;
        println!("{val}");
        Ok(())
    }

    pub fn error(token: Token, msg: &str) -> String {
        if token.ty.eq(&TokenType::EOF) {
            return Self::report(Some(token.line), Some("at end".to_string()), msg);
        } else {
            return Self::report(
                Some(token.line),
                Some(format!("at {}", token.ty.lexeme())),
                msg,
            );
        }
    }

    pub(crate) fn report(line: Option<usize>, where_: Option<String>, msg: &str) -> String {
        let mut err = String::new();
        if let Some(line) = line {
            err.push_str(&format!("[line {line}] "));
        }
        if let Some(where_) = where_ {
            err.push_str(&format!("Error {where_}: {msg}"));
        } else {
            err.push_str(&format!("Error: {msg}"));
        }

        err
    }
}
