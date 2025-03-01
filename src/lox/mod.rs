pub mod environment;
pub mod error;
pub mod expr;
pub mod interpreter;
pub mod lox_callable;
pub mod parser;
pub mod scanner;
pub mod stmt;
pub mod value;

use std::{
    fs,
    io::{self, BufRead, Write},
    path::Path,
    sync::{LazyLock, Mutex},
};

use interpreter::Interpreter;
use parser::Parser;
use scanner::Scanner;

static INTERPRETER: LazyLock<Mutex<Interpreter>> = LazyLock::new(|| Mutex::new(Interpreter::new()));

pub struct Lox;

impl Lox {
    pub fn run(source: &str) {
        let mut scanner = Scanner::new(source);
        if let Ok(tokens) = scanner.scan_tokens() {
            let mut parser = Parser::new(tokens);
            match parser.parse() {
                Ok(stmts) => {
                    let mut interpreter = INTERPRETER.lock().expect("Unable to lock INTERPRETER");
                    if let Err(err) = interpreter.interpret(&stmts) {
                        eprintln!("Runtime Error: {err}");
                    }
                }
                Err(errors) => {
                    for err in errors {
                        eprintln!("Parser Error: {err}");
                    }
                }
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
            if let Some(Ok(input)) = stdin.lock().lines().next() {
                if !input.is_empty() {
                    Self::run(&input);
                }
            } else {
                break;
            }
        }
    }
}
