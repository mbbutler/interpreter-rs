use self::error::LoxError;

pub mod error;
pub mod interpreter;
pub mod scanner;

pub type Result<'a, T> = std::result::Result<T, LoxError<'a>>;
