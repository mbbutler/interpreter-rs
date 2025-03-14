pub mod environment;
pub mod error;
pub mod expr;
pub mod interpreter;
pub mod lox_callable;
pub mod lox_class;
pub mod lox_function;
pub mod lox_instance;
pub mod parser;
pub mod resolver;
pub mod scanner;
pub mod stmt;
pub mod value;

use std::{
    fs,
    io::{self, BufRead, Write},
    path::Path,
};

use error::LoxError;
use interpreter::Interpreter;
use parser::Parser;
use resolver::Resolver;
use scanner::Scanner;

type LoxResult = Result<(), LoxError>;

pub struct Lox;

impl Lox {
    pub fn run(source: &str, interpreter: &mut Interpreter) -> LoxResult {
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens()?;
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse()?;
        let mut resolver = Resolver::new(interpreter);
        resolver.resolve_stmts(&stmts)?;
        interpreter.interpret(&stmts)?;
        Ok(())
    }

    pub fn run_file<T>(file_path: T)
    where
        T: AsRef<Path>,
    {
        let source = fs::read_to_string(file_path).expect("Should have been able to read the file");
        let mut interpreter = Interpreter::new();
        if let Err(err) = Self::run(&source, &mut interpreter) {
            eprintln!("{err}");
        }
    }

    pub fn run_prompt() {
        let mut interpreter = Interpreter::new();
        let stdin = io::stdin();
        println!("=== Welcome to the Lox REPL ===");
        loop {
            print!("  > ");
            let _ = io::stdout().flush();
            if let Some(Ok(input)) = stdin.lock().lines().next() {
                if !input.is_empty() {
                    if let Err(err) = Self::run(&input, &mut interpreter) {
                        eprintln!("{err}");
                    }
                }
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lox::interpreter::Interpreter;

    use super::Lox;

    #[test]
    fn closure() {
        let input = r#"
            fun makeCounter() {
                var i = 0;
                fun count() {
                    i = i + 1;
                    print i;
                }
                return count;
            }

            var counter = makeCounter();
            counter();
            counter();
        "#;
        let mut interpreter = Interpreter::new();
        assert!(Lox::run(input, &mut interpreter).is_ok());
    }

    #[test]
    fn for_loop() {
        let input = r#"
            var a = 0;
            var temp;

            for (var b = 1; a < 10000; b = temp + b) {
                print a;
                temp = a;
                a = b;
            }
        "#;
        let mut interpreter = Interpreter::new();
        assert!(Lox::run(input, &mut interpreter).is_ok());
    }

    #[test]
    fn lox_function() {
        let input = r#"
            fun sayHi(first, last) {
                print "Hi, " + first + " " + last + "!";
            }

            sayHi("Dear", "Reader");
        "#;
        let mut interpreter = Interpreter::new();
        assert!(Lox::run(input, &mut interpreter).is_ok());
    }

    #[test]
    fn native_function() {
        let input = r#"
            print clock();
        "#;
        let mut interpreter = Interpreter::new();
        assert!(Lox::run(input, &mut interpreter).is_ok());
    }

    #[test]
    fn resolution_error() {
        let input = r#"
            var a = "outer";
            {
                var a = a;
            }
        "#;
        let mut interpreter = Interpreter::new();
        assert!(Lox::run(input, &mut interpreter).is_err());
    }

    #[test]
    fn return_stmt() {
        let input = r#"
            fun fib(n) {
                if (n <= 1) return n;
                return fib(n - 2) + fib(n - 1);
            }

            for (var i = 0; i < 20; i = i + 1) {
                print fib(i);
            }
        "#;
        let mut interpreter = Interpreter::new();
        assert!(Lox::run(input, &mut interpreter).is_ok());
    }

    #[test]
    fn scope() {
        let input = r#"
            var a = "global a";
            var b = "global b";
            var c = "global c";
            {
                var a = "outer a";
                var b = "outer b";
                {
                    var a = "inner a";
                    print a;
                    print b;
                    print c;
                }
                print a;
                print b;
                print c;
            }
            print a;
            print b;
            print c;
        "#;
        let mut interpreter = Interpreter::new();
        assert!(Lox::run(input, &mut interpreter).is_ok());
    }

    #[test]
    fn double_assignment() {
        let input = r#"
        fun bad() {
            var a = "first";
            var a = "second";
        }
        "#;
        let mut interpreter = Interpreter::new();
        assert!(Lox::run(input, &mut interpreter).is_err());
    }

    #[test]
    fn top_level_return() {
        let input = r#"
            return "at top level";
        "#;
        let mut interpreter = Interpreter::new();
        assert!(Lox::run(input, &mut interpreter).is_err());
    }

    #[test]
    fn class_decl() {
        let input = r#"
            class DevonshireCream {
                serveOn() {
                    return "Scones";
                }
            }
            print DevonshireCream;
        "#;
        let mut interpreter = Interpreter::new();
        assert!(Lox::run(input, &mut interpreter).is_ok());
    }

    #[test]
    fn class_instance() {
        let input = r#"
            class Bagel {}
            var bagel = Bagel();
            print bagel;
        "#;
        let mut interpreter = Interpreter::new();
        assert!(Lox::run(input, &mut interpreter).is_ok());
    }

    #[test]
    fn class_property() {
        let input = r#"
            class Bacon {
                    eat() {
                        print "Crunch crunch crunch!";
                    }
                }
                
                Bacon().eat();
        "#;
        let mut interpreter = Interpreter::new();
        assert!(Lox::run(input, &mut interpreter).is_ok());
    }
}
