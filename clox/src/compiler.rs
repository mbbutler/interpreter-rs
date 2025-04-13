use crate::{error::Result, scanner::Scanner};

pub struct Compiler;

impl Compiler {
    pub fn compile(s: &str) -> Result<()> {
        let scanner = Scanner::new(s);
        let mut line = 0;
        for token in scanner {
            let token = token?;
            if token.line != line {
                print!("{:>4} ", token.line);
                line = token.line;
            } else {
                print!("   | ");
            }
            println!("{:?}", token);
        }
        Ok(())
    }
}
