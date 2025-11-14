mod error;
mod expression;
mod scanner;

use anyhow::{Error, anyhow};
use std::{
    env,
    fs::File,
    io::{self, BufRead, BufReader, Write},
    path::PathBuf,
};

use crate::scanner::Scanner;

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => run_prompt(),
        2 => {
            // run_file(PathBuf::from(args[1].to_owned()))
            todo!("Figure out how to implement multi-line quotes and block comments");
        }
        _ => Err(anyhow!("Usage: lox [script]")),
    }
}

fn run_prompt() -> Result<(), Error> {
    loop {
        print!("Input Lox: ");
        // Make sure prompt happens first
        io::stdout().flush().unwrap();

        let mut line = String::new();
        io::stdin().read_line(&mut line).unwrap();
        let line = line.trim_end().to_string();
        if line.is_empty() {
            break;
        }

        run(line)?;
    }

    Ok(())
}

fn run_file(path: PathBuf) -> Result<(), Error> {
    if !path.exists() {
        return Err(anyhow!("File does not exist"));
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line?;
        run(line)?;
    }

    Ok(())
}

fn run(line: String) -> Result<(), Error> {
    let mut s = Scanner::new(line.to_owned());
    let tokens = s.scan_tokens();

    for token in tokens.iter() {
        println!("TOKEN -> {token}");
    }

    Ok(())
}
