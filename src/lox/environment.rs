use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::{error::RuntimeException, interpreter::RuntimeResult, scanner::Token, value::Value};

#[derive(Default, Debug)]
pub struct Environment {
    values: HashMap<String, Value>,
    pub enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new(enclosing: &Rc<RefCell<Environment>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            values: HashMap::new(),
            enclosing: Some(Rc::clone(enclosing)),
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
                if let Some(enclosing) = self.enclosing.as_ref() {
                    enclosing.borrow_mut().assign(name, value)
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
                    enclosing.borrow().get(token)
                } else {
                    Err(RuntimeException::new_error(
                        token.to_owned(),
                        format!("Undefined variable '{}'.", token.lexeme),
                    ))
                }
            }
        }
    }

    pub fn get_at(&self, distance: usize, name: &str) -> RuntimeResult<Value> {
        if distance == 0 {
            Ok(self
                .values
                .get(name)
                .map(|val| val.to_owned())
                .unwrap_or_else(|| panic!("Undefined variable '{name}'.")))
        } else {
            self.enclosing
                .as_ref()
                .expect("Attempted to access None enclosing Environment.")
                .borrow()
                .get_at(distance - 1, name)
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
                .borrow_mut()
                .assign_at(distance - 1, token, value)
        }
    }
}
