use std::{
    fmt::Display,
    ops::{Neg, Not},
};

use super::{error::RuntimeError, interpreter::RuntimeResult, scanner::Token};

#[derive(Clone, Debug)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
    String(String),
}

impl Value {
    pub fn checked_add<'a>(&self, operator: &'a Token, rhs: &Value) -> RuntimeResult<'a, Value> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::Number(lhs + rhs)),
            (Self::String(lhs), Self::String(rhs)) => Ok(Self::String(format!("{lhs}{rhs}"))),
            _ => Err(RuntimeError::new(
                operator,
                "Operands must both be numbers or strings.".to_string(),
            )),
        }
    }

    pub fn checked_sub<'a>(&self, operator: &'a Token, rhs: &Value) -> RuntimeResult<'a, Value> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::Number(lhs - rhs)),
            _ => Err(RuntimeError::new(
                operator,
                "Operands must be numbers.".to_string(),
            )),
        }
    }

    pub fn checked_mul<'a>(&self, operator: &'a Token, rhs: &Value) -> RuntimeResult<'a, Value> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::Number(lhs * rhs)),
            _ => Err(RuntimeError::new(
                operator,
                "Operands must be numbers.".to_string(),
            )),
        }
    }

    pub fn checked_div<'a>(&self, operator: &'a Token, rhs: &Value) -> RuntimeResult<'a, Value> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::Number(lhs / rhs)),
            _ => Err(RuntimeError::new(
                operator,
                "Operands must be numbers.".to_string(),
            )),
        }
    }

    pub fn checked_gt<'a>(&self, operator: &'a Token, rhs: &Value) -> RuntimeResult<'a, Value> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::Bool(lhs > rhs)),
            _ => Err(RuntimeError::new(
                operator,
                "Operands must be numbers.".to_string(),
            )),
        }
    }

    pub fn checked_gte<'a>(&self, operator: &'a Token, rhs: &Value) -> RuntimeResult<'a, Value> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::Bool(lhs >= rhs)),
            _ => Err(RuntimeError::new(
                operator,
                "Operands must be numbers.".to_string(),
            )),
        }
    }

    pub fn checked_lt<'a>(&self, operator: &'a Token, rhs: &Value) -> RuntimeResult<'a, Value> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::Bool(lhs < rhs)),
            _ => Err(RuntimeError::new(
                operator,
                "Operands must be numbers.".to_string(),
            )),
        }
    }

    pub fn checked_lte<'a>(&self, operator: &'a Token, rhs: &Value) -> RuntimeResult<'a, Value> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::Bool(lhs >= rhs)),
            _ => Err(RuntimeError::new(
                operator,
                "Operands must be numbers.".to_string(),
            )),
        }
    }

    pub fn checked_negate<'a>(&self, operator: &'a Token) -> RuntimeResult<'a, Value> {
        match self {
            Self::Number(n) => Ok(Self::Number(-n)),
            _ => Err(RuntimeError::new(
                operator,
                "Operand must be a number.".to_string(),
            )),
        }
    }

    pub fn not(&self) -> Value {
        Self::Bool(!self.is_truthy())
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Nil => false,
            Self::Bool(b) => *b,
            _ => true,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::Number(lhs), Self::Number(rhs)) => lhs.partial_cmp(rhs),
            _ => panic!("Invalid PartialOrd operation for: {self} and {other}"),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Number(lhs), Self::Number(rhs)) => lhs == rhs,
            (Self::String(lhs), Self::String(rhs)) => lhs == rhs,
            (Self::Bool(lhs), Self::Bool(rhs)) => lhs == rhs,
            (Self::Nil, Self::Nil) => true,
            _ => false,
        }
    }
}

impl Neg for Value {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            Self::Number(n) => Self::Number(-n),
            _ => panic!("Invalid Negative operation Value: {self}"),
        }
    }
}

impl Not for Value {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Self::Bool(b) => Self::Bool(!b),
            _ => panic!("Invalid Not operation Value: {self}"),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil => write!(f, "nil"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Number(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{}", *n as isize)
                } else {
                    write!(f, "{n}")
                }
            }
            Value::String(s) => write!(f, "{s}"),
        }
    }
}
