use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use super::{
    error::RuntimeException, interpreter::RuntimeResult, lox_callable::CallableFn,
    lox_class::LoxClass, scanner::Token, value::Value,
};

#[derive(Debug, Clone)]
pub struct LoxInstance {
    pub class: LoxClass,
    fields: Rc<RefCell<HashMap<String, Value>>>,
}

impl LoxInstance {
    pub fn new(class: &LoxClass) -> Self {
        Self {
            class: class.clone(),
            fields: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn get(&self, name: &Token) -> RuntimeResult<Value> {
        if let Some(val) = self.fields.borrow().get(&name.lexeme) {
            Ok(val.to_owned())
        } else if let Some(method) = self.class.find_method(&name.lexeme) {
            Ok(Value::Callable(CallableFn::Lox(method.bind(self)?)))
        } else {
            Err(RuntimeException::new_error(
                name.to_owned(),
                format!("Undefined property '{}'.", &name.lexeme),
            ))
        }
    }

    pub fn set(&mut self, name: &Token, value: Value) {
        self.fields
            .borrow_mut()
            .insert(name.lexeme.to_owned(), value);
    }
}

impl Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", &self.class.name)
    }
}
