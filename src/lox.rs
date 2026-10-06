mod scanner;
mod token;

use anyhow::Result;
use scanner::Scanner;

use crate::lox::scanner::ScannerError;
use crate::lox::token::Token;

pub struct Lox;
impl Lox {
    pub fn run(line: String) -> Result<()> {
        let tokens = Scanner::scan(&line).collect::<Result<Vec<Token>, ScannerError>>()?;
        for tok in tokens {
            println!("{tok}")
        }
        Ok(())
    }

    pub fn error(line: Option<usize>, msg: String) {
        Self::report(line, None, msg);
    }

    fn report(line: Option<usize>, where_: Option<String>, msg: String) {
        let mut err = String::new();
        if let Some(line) = line {
            err.push_str(&format!("[line {line}] "));
        }
        if let Some(where_) = where_ {
            err.push_str(&format!("Error {where_}: {msg}"));
        } else {
            err.push_str(&format!("Error: {msg}"));
        }
        println!("{err}");
    }
}
