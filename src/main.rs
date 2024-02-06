use std::env;

mod lox;

use lox::interpreter::Interpreter;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        0 => unreachable!(),
        1 => Interpreter::run_prompt(),
        2 => Interpreter::run_file(&args[0]),
        _ => println!("Usage is: cargo run <path/to/script>"),
    }
}
