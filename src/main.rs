mod ast;
mod parser;
mod regex;

use regex::Regex;
use std::{env, io, process::ExitCode};

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 || args[1] != "-E" {
        eprintln!("Usage: {} -E <pattern>", args[0]);
        return ExitCode::from(2);
    }

    let regex = match Regex::new(&args[2]) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("invalid pattern: {e}");
            return ExitCode::from(2);
        }
    };

    let mut any_matched = false;
    for line in io::stdin().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("error reading input: {e}");
                return ExitCode::from(2);
            }
        };

        if regex.is_match(&line) {
            println!("{line}");
            any_matched = true;
        }
    }

    if any_matched {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
