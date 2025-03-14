use std::{collections::HashMap, fmt::Display};

use super::{
    interpreter::{Interpreter, RuntimeResult},
    lox_callable::LoxCallable,
    lox_function::LoxFunction,
    lox_instance::LoxInstance,
    value::Value,
};

#[derive(Debug, Clone)]
pub struct LoxClass {
    pub name: String,
    pub methods: HashMap<String, LoxFunction>,
}

impl LoxClass {
    pub fn new(name: &str, methods: HashMap<String, LoxFunction>) -> Self {
        Self {
            name: name.to_string(),
            methods,
        }
    }

    pub fn find_method(&self, name: &str) -> Option<&LoxFunction> {
        self.methods.get(name)
    }
}

impl LoxCallable for LoxClass {
    fn call(&self, interpreter: &mut Interpreter, arguments: &[Value]) -> RuntimeResult<Value> {
        let instance = LoxInstance::new(self);
        if let Some(initializer) = self.find_method("init") {
            initializer.bind(&instance)?.call(interpreter, arguments)?;
        }
        Ok(Value::Instance(instance))
    }

    fn arity(&self) -> usize {
        match self.find_method("init") {
            Some(init_fn) => init_fn.arity(),
            None => 0,
        }
    }
}

impl Display for LoxClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
