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
    pub superclass: Option<Box<LoxClass>>,
}

impl LoxClass {
    pub fn new(
        name: &str,
        superclass: Option<LoxClass>,
        methods: HashMap<String, LoxFunction>,
    ) -> Self {
        Self {
            name: name.to_string(),
            methods,
            superclass: superclass.map(Box::new),
        }
    }

    pub fn find_method(&self, name: &str) -> Option<&LoxFunction> {
        self.methods
            .get(name)
            .or_else(|| self.superclass.as_ref().and_then(|sc| sc.find_method(name)))
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
