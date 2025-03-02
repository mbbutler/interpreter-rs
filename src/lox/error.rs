use std::{
    fmt::Display,
    sync::{PoisonError, RwLockReadGuard, RwLockWriteGuard},
};

use super::{
    environment::Environment,
    scanner::{Token, TokenType},
    value::Value,
};

pub enum LoxError {
    Scanner(Vec<ScanError>),
    Parser(Vec<ParseError>),
    Resolver(ResolverError),
    Runtime(RuntimeException),
}

impl Display for LoxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Scanner(errs) => {
                for err in errs {
                    writeln!(f, "{}", err)?;
                }
                Ok(())
            }
            Self::Parser(errs) => {
                for err in errs {
                    writeln!(f, "{}", err)?;
                }
                Ok(())
            }
            Self::Resolver(err) => write!(f, "{}", err),
            Self::Runtime(err) => write!(f, "{}", err),
        }
    }
}

impl From<Vec<ScanError>> for LoxError {
    fn from(value: Vec<ScanError>) -> Self {
        Self::Scanner(value)
    }
}

impl From<Vec<ParseError>> for LoxError {
    fn from(value: Vec<ParseError>) -> Self {
        Self::Parser(value)
    }
}

impl From<ResolverError> for LoxError {
    fn from(value: ResolverError) -> Self {
        Self::Resolver(value)
    }
}

impl From<RuntimeException> for LoxError {
    fn from(value: RuntimeException) -> Self {
        Self::Runtime(value)
    }
}

#[derive(Clone)]
pub struct ScanError {
    msg: String,
    code: String,
    col: usize,
    line: usize,
}

impl ScanError {
    pub fn new(msg: String, code: &str, col: usize, line: usize) -> Self {
        ScanError {
            msg,
            code: code.to_string(),
            col,
            line,
        }
    }
}

impl Display for ScanError {
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

#[derive(Debug)]
pub enum RuntimeException {
    Error(RuntimeError),
    Return(Value),
}

impl RuntimeException {
    pub fn new_error(token: Token, msg: String) -> Self {
        Self::Error(RuntimeError { token, msg })
    }

    pub fn new_return(value: Value) -> Self {
        Self::Return(value)
    }
}

impl Display for RuntimeException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error(err) => write!(f, "{err}"),
            Self::Return(val) => write!(f, "{val}"),
        }
    }
}

#[derive(Debug)]
pub struct RuntimeError {
    token: Token,
    msg: String,
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[line {}] Error at '{}': {}",
            self.token.line, self.token.lexeme, &self.msg
        )
    }
}

impl From<PoisonError<RwLockWriteGuard<'_, Environment>>> for RuntimeException {
    fn from(value: PoisonError<RwLockWriteGuard<'_, Environment>>) -> Self {
        Self::Error(RuntimeError {
            token: Token::default(),
            msg: format!("RwLock is poisoned for writing: {value}"),
        })
    }
}

impl From<PoisonError<RwLockReadGuard<'_, Environment>>> for RuntimeException {
    fn from(value: PoisonError<RwLockReadGuard<'_, Environment>>) -> Self {
        Self::Error(RuntimeError {
            token: Token::default(),
            msg: format!("RwLock is poisoned for reading: {value}"),
        })
    }
}

pub struct ResolverError {
    token: Token,
    msg: String,
}

impl ResolverError {
    pub fn new(token: Token, msg: String) -> Self {
        Self { token, msg }
    }
}

impl Display for ResolverError {
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
