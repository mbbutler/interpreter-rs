use std::fmt::Display;

use num_enum::{TryFromPrimitive, TryFromPrimitiveError};

pub type Result<T> = std::result::Result<T, InterpretError>;

#[derive(Debug)]
pub enum InterpretError {
    CompileError(String),
    RuntimeError(String),
}

impl<Enum: TryFromPrimitive> From<TryFromPrimitiveError<Enum>> for InterpretError {
    fn from(value: TryFromPrimitiveError<Enum>) -> Self {
        Self::RuntimeError(value.to_string())
    }
}

impl Display for InterpretError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CompileError(err) => write!(f, "{err}"),
            Self::RuntimeError(err) => write!(f, "{err}"),
        }
    }
}
