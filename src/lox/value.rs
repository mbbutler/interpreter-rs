use std::{
    fmt::Display,
    ops::{Neg, Not},
};

use super::{
    error::RuntimeException, interpreter::RuntimeResult, lox_callable::LoxCallable, scanner::Token,
};

#[derive(Clone, Debug)]
pub enum Value {
    Bool(bool),
    Callable(LoxCallable),
    Nil,
    Number(f64),
    String(String),
}

impl Value {
    pub fn checked_add(&self, operator: &Token, rhs: &Value) -> RuntimeResult<Value> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::Number(lhs + rhs)),
            (Self::String(lhs), Self::String(rhs)) => Ok(Self::String(format!("{lhs}{rhs}"))),
            _ => Err(RuntimeException::new_error(
                operator.to_owned(),
                "Operands must both be numbers or strings.".to_string(),
            )),
        }
    }

    pub fn checked_sub(&self, operator: &Token, rhs: &Value) -> RuntimeResult<Value> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::Number(lhs - rhs)),
            _ => Err(RuntimeException::new_error(
                operator.to_owned(),
                "Operands must be numbers.".to_string(),
            )),
        }
    }

    pub fn checked_mul(&self, operator: &Token, rhs: &Value) -> RuntimeResult<Value> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::Number(lhs * rhs)),
            _ => Err(RuntimeException::new_error(
                operator.to_owned(),
                "Operands must be numbers.".to_string(),
            )),
        }
    }

    pub fn checked_div(&self, operator: &Token, rhs: &Value) -> RuntimeResult<Value> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::Number(lhs / rhs)),
            _ => Err(RuntimeException::new_error(
                operator.to_owned(),
                "Operands must be numbers.".to_string(),
            )),
        }
    }

    pub fn checked_gt(&self, operator: &Token, rhs: &Value) -> RuntimeResult<Value> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::Bool(lhs > rhs)),
            _ => Err(RuntimeException::new_error(
                operator.to_owned(),
                "Operands must be numbers.".to_string(),
            )),
        }
    }

    pub fn checked_gte(&self, operator: &Token, rhs: &Value) -> RuntimeResult<Value> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::Bool(lhs >= rhs)),
            _ => Err(RuntimeException::new_error(
                operator.to_owned(),
                "Operands must be numbers.".to_string(),
            )),
        }
    }

    pub fn checked_lt(&self, operator: &Token, rhs: &Value) -> RuntimeResult<Value> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::Bool(lhs < rhs)),
            _ => Err(RuntimeException::new_error(
                operator.to_owned(),
                "Operands must be numbers.".to_string(),
            )),
        }
    }

    pub fn checked_lte(&self, operator: &Token, rhs: &Value) -> RuntimeResult<Value> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::Bool(lhs <= rhs)),
            _ => Err(RuntimeException::new_error(
                operator.to_owned(),
                "Operands must be numbers.".to_string(),
            )),
        }
    }

    pub fn checked_negate(&self, operator: &Token) -> RuntimeResult<Value> {
        match self {
            Self::Number(n) => Ok(Self::Number(-n)),
            _ => Err(RuntimeException::new_error(
                operator.to_owned(),
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
            Value::Nil => write!(f, "nil"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Callable(func) => write!(f, "{func}"),
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
