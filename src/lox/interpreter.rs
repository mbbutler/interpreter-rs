use std::{
    fs,
    io::{self, BufRead, Write},
    path::Path,
};

use super::scanner::Parser;

pub struct Interpreter {}

impl Interpreter {
    pub fn run(source: &str) {
        let mut parser = Parser::new(source);
        let tokens = parser.scan_tokens();
        match tokens {
            Ok(tokens) => {
                for t in tokens.iter() {
                    println!("{:?}", t);
                }
            }
            Err(_) => {}
        }
    }

    pub fn run_file<T>(file_path: T)
    where
        T: AsRef<Path>,
    {
        let source = fs::read_to_string(file_path).expect("Should have been able to read the file");
        Self::run(&source);
    }

    pub fn run_prompt() {
        let stdin = io::stdin();
        println!("=== Welcome to the Lox REPL ===");
        loop {
            print!("  > ");
            let _ = io::stdout().flush();
            if let Some(str_result) = stdin.lock().lines().next() {
                if let Ok(input) = str_result {
                    Self::run(&input);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }
}
