mod lox;

use anyhow::{Context, Result, anyhow};
use clap::{ArgGroup, Parser};
use lox::Lox;
use std::{
    fs::File,
    io::{BufRead, BufReader, Write, stdin, stdout},
    path::PathBuf,
    process::exit,
};

#[derive(Parser)]
#[command(version, about, long_about = None, group(ArgGroup::new("input").required(true).args(["interactive", "file"])))]
struct Cli {
    /// Runs lines of lox interactively from the terminal
    #[arg(short = 'i', long)]
    interactive: bool,

    /// Runs lox from a `.lox` file
    file: Option<PathBuf>,
}

fn main() -> Result<()> {
    enum Input {
        Interactive,
        File(PathBuf),
    }

    let cli = Cli::parse();
    let input = match cli.file {
        Some(path) => Input::File(path),
        None => Input::Interactive,
    };

    match input {
        Input::File(p) => {
            if (p.extension().is_some() && !p.extension().unwrap().eq(".lox"))
                || p.extension().is_none()
            {
                return Err(anyhow!(
                    "supplied path doesn't have a file with a `.lox` extension"
                ));
            }

            let f = File::open(&p).with_context(|| {
                format!(
                    "failed to open {}",
                    p.to_str().expect("supplied path is not valid utf-8")
                )
            })?;
            for (i, line) in BufReader::new(f).lines().enumerate() {
                let line = line?;
                if let Err(msg) = Lox::run(line) {
                    Lox::error(Some(i + 1), msg.to_string());
                    exit(65);
                }
            }
        }
        Input::Interactive => {
            let mut buf = String::new();
            loop {
                print!("> ");
                stdout().flush()?;
                buf.clear();
                if stdin().read_line(&mut buf)? == 0 {
                    break; // EOF
                }
                let line = buf.trim_end(); // drop \n or \r\n
                if line == "quit" || line == "q" {
                    break;
                }
                if let Err(msg) = Lox::run(line.to_string()) {
                    Lox::error(None, msg.to_string());
                }
            }
        }
    }

    Ok(())
}
