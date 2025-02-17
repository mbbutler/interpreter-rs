use std::fmt::Display;

use super::scanner::{Token, TokenType};

// pub type LoxResult<'a, T> = std::result::Result<T, LoxError<'a>>;

// pub enum LoxError<'a> {
//     Scanner(ScanError<'a>),
//     Parser(ParseError),
//     Runtime(RuntimeError<'a>),
// }

// impl<'a> Display for LoxError<'a> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Self::Scanner(err) => write!(f, "{}", err),
//             Self::Parser(err) => write!(f, "{}", err),
//             Self::Runtime(err) => write!(f, "{}", err),
//         }
//     }
// }

pub struct ScanError<'a> {
    msg: String,
    code: &'a str,
    col: usize,
    line: usize,
}

impl<'a> ScanError<'a> {
    pub fn new(msg: String, code: &'a str, col: usize, line: usize) -> Self {
        ScanError {
            msg,
            code,
            col,
            line,
        }
    }
}

impl<'a> Display for ScanError<'a> {
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

pub struct ParseError {
    token: Token,
    msg: String,
}

impl ParseError {
    pub fn new(token: Token, msg: String) -> Self {
        ParseError { token, msg }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.token.t_type {
            TokenType::Eof => write!(f, "[line {}] Error at end: {}", self.token.line, self.msg),
            _ => write!(
                f,
                "[line {}] Error at '{}': {}",
                self.token.line, self.token.lexeme, self.msg
            ),
        }
    }
}

pub struct RuntimeError<'a> {
    token: &'a Token,
    msg: String,
}

impl<'a> RuntimeError<'a> {
    pub fn new(token: &'a Token, msg: String) -> Self {
        RuntimeError { token, msg }
    }
}

impl Display for RuntimeError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[line {}] Error at '{}': {}",
            self.token.line, self.token.lexeme, &self.msg
        )
    }
}
