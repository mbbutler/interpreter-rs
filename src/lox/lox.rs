use std::{
    fs,
    io::{self, BufRead, Write},
    path::Path,
    sync::LazyLock,
};

use crate::lox::interpreter::Interpreter;

use super::parser::Parser;
use super::scanner::Scanner;

static INTERPRETER: LazyLock<Interpreter> = LazyLock::new(|| Interpreter);

pub struct Lox;

impl Lox {
    pub fn run(source: &str) {
        let mut scanner = Scanner::new(source);
        if let Ok(tokens) = scanner.scan_tokens() {
            let mut parser = Parser::new(tokens);
            match parser.parse() {
                Ok(stmts) => {
                    if let Err(err) = INTERPRETER.interpret(&stmts) {
                        eprintln!("Runtime Error: {err}")
                    }
                }
                Err(err) => eprintln!("Parser Error: {err}"),
            }
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
                    if input.len() > 0 {
                        Self::run(&input);
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }
}
