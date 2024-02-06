use std::fmt::Display;

pub enum LoxError<'a> {
    Parser(ParseError<'a>),
    Runtime,
}

impl<'a> Display for LoxError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parser(err) => write!(f, "{}", err),
            Self::Runtime => write!(f, "Runtime error"),
        }
    }
}

pub struct ParseError<'a> {
    msg: String,
    code: &'a str,
    col: usize,
    line: usize,
}

impl<'a> ParseError<'a> {
    pub fn new(msg: String, code: &'a str, col: usize, line: usize) -> Self {
        ParseError {
            msg,
            code,
            col,
            line,
        }
    }
}

impl<'a> Display for ParseError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Error: {}", &self.msg)?;
        writeln!(f, "    {} | {}", self.line, self.code)?;
        write!(
            f,
            "{}^--- Here",
            &" ".repeat(self.col + 6 + self.line.to_string().len())
        )
    }
}
