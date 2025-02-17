use std::collections::HashMap;

use super::{error::RuntimeError, interpreter::RuntimeResult, scanner::Token, value::Value};

#[derive(Default)]
pub struct Environment {
    values: HashMap<String, Value>,
    pub enclosing: Option<Box<Environment>>,
}

impl Environment {
    pub fn define(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_string(), value);
    }

    pub fn assign<'a>(&mut self, name: &'a Token, value: Value) -> RuntimeResult<'a, ()> {
        match self.values.get_mut(&name.lexeme) {
            Some(val) => {
                *val = value;
                Ok(())
            }
            None => {
                if let Some(enclosing) = self.enclosing.as_mut() {
                    enclosing.assign(name, value)
                } else {
                    Err(RuntimeError::new(
                        name,
                        format!("Undefined variable '{}'.", name.lexeme),
                    ))
                }
            }
        }
    }

    pub fn get<'a>(&self, token: &'a Token) -> RuntimeResult<'a, Value> {
        match self.values.get(&token.lexeme) {
            Some(value) => Ok(value.to_owned()),
            None => {
                if let Some(enclosing) = &self.enclosing {
                    enclosing.get(token)
                } else {
                    Err(RuntimeError::new(
                        token,
                        format!("Undefined variable '{}'.", token.lexeme),
                    ))
                }
            }
        }
    }
}
