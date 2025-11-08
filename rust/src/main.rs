mod error;
mod scanner;

use anyhow::{Error, anyhow};
use std::{
    env,
    fs::File,
    io::{self, BufRead, BufReader, Write},
    path::PathBuf,
};

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => run_prompt(),
        2 => run_file(PathBuf::from(args[1].to_owned())),
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
        if line == "\n" {
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
    Ok(println!("Ran line: '{}'", line))
}
