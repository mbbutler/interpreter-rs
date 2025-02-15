use std::env;

mod lox;

use lox::lox::Lox;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        0 => unreachable!(),
        1 => Lox::run_prompt(),
        2 => Lox::run_file(&args[0]),
        _ => println!("Usage is: cargo run <path/to/script>"),
    }
}
