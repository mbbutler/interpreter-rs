use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use super::{error::RuntimeException, interpreter::RuntimeResult, scanner::Token, value::Value};

#[derive(Default, Debug)]
pub struct Environment {
    values: HashMap<String, Value>,
    enclosing: Option<Arc<RwLock<Environment>>>,
}

impl Environment {
    pub fn new(enclosing: &Arc<RwLock<Environment>>) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            values: HashMap::new(),
            enclosing: Some(Arc::clone(enclosing)),
        }))
    }

    pub fn define(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_string(), value);
    }

    pub fn assign(&mut self, name: &Token, value: Value) -> RuntimeResult<()> {
        match self.values.get_mut(&name.lexeme) {
            Some(val) => {
                *val = value;
                Ok(())
            }
            None => {
                if let Some(enclosing) = self.enclosing.as_mut() {
                    enclosing.write()?.assign(name, value)
                } else {
                    Err(RuntimeException::new_error(
                        name.to_owned(),
                        format!("Undefined variable '{}'.", name.lexeme),
                    ))
                }
            }
        }
    }

    pub fn get(&self, token: &Token) -> RuntimeResult<Value> {
        match self.values.get(&token.lexeme) {
            Some(value) => Ok(value.to_owned()),
            None => {
                if let Some(enclosing) = &self.enclosing {
                    enclosing.read()?.get(token)
                } else {
                    Err(RuntimeException::new_error(
                        token.to_owned(),
                        format!("Undefined variable '{}'.", token.lexeme),
                    ))
                }
            }
        }
    }

    pub fn get_at(&self, distance: usize, token: &Token) -> RuntimeResult<Value> {
        if distance == 0 {
            match self.values.get(&token.lexeme) {
                Some(value) => Ok(value.to_owned()),
                None => Err(RuntimeException::new_error(
                    token.to_owned(),
                    format!("Undefined variable '{}'.", token.lexeme),
                )),
            }
        } else {
            self.enclosing
                .as_ref()
                .expect("Attempted to access None enclosing Environment.")
                .read()?
                .get_at(distance - 1, token)
        }
    }

    pub fn assign_at(&mut self, distance: usize, token: &Token, value: Value) -> RuntimeResult<()> {
        if distance == 0 {
            self.values.insert(token.lexeme.to_owned(), value);
            Ok(())
        } else {
            self.enclosing
                .as_ref()
                .expect("Attempted to access None enclosing Environment.")
                .write()?
                .assign_at(distance - 1, token, value)
        }
    }
}
