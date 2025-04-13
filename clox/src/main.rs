mod chunk;
mod compiler;
mod error;
mod scanner;
mod stack;
mod value;
mod vm;

use std::io::{BufRead, Write};

use vm::VM;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.len() {
        1 => repl(),
        2 => run_file(&args[1]),
        _ => println!("Usage is: cargo run <path/to/script>"),
    }
}

fn repl() {
    let mut vm = VM::default();
    loop {
        let stdin = std::io::stdin();
        loop {
            print!("> ");
            std::io::stdout().flush().unwrap();
            if let Some(Ok(input)) = stdin.lock().lines().next() {
                if !input.is_empty() {
                    if let Err(err) = vm.interpret(&input) {
                        eprintln!("{err}");
                    }
                }
            } else {
                break;
            }
        }
    }
}

fn run_file(path: &str) {
    let input = std::fs::read_to_string(path).unwrap();
    let mut vm = VM::default();
    if let Err(err) = vm.interpret(&input) {
        eprint!("{err}");
    }
}
