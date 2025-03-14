use std::{cell::RefCell, fmt::Display, rc::Rc};

use super::{
    environment::Environment,
    error::RuntimeException,
    interpreter::{Interpreter, RuntimeResult},
    lox_callable::LoxCallable,
    lox_instance::LoxInstance,
    stmt::Function,
    value::Value,
};

#[derive(Clone, Debug)]
pub struct LoxFunction {
    declaration: Box<Function>,
    closure: Rc<RefCell<Environment>>,
    is_initializer: bool,
}

impl LoxFunction {
    pub fn new(
        declaration: &Function,
        closure: &Rc<RefCell<Environment>>,
        is_initializer: bool,
    ) -> Self {
        Self {
            declaration: Box::new(declaration.clone()),
            closure: Rc::clone(closure),
            is_initializer,
        }
    }

    pub fn bind(&self, instance: &LoxInstance) -> RuntimeResult<LoxFunction> {
        let environment = Environment::new(&self.closure);
        environment
            .borrow_mut()
            .define("this", Value::Instance(instance.clone()));
        Ok(LoxFunction::new(
            &self.declaration,
            &environment,
            self.is_initializer,
        ))
    }
}

impl LoxCallable for LoxFunction {
    fn call(&self, interpreter: &mut Interpreter, arguments: &[Value]) -> RuntimeResult<Value> {
        let environment = Environment::new(&self.closure);
        let mut env_write = environment.borrow_mut();
        for (param, arg) in self.declaration.params.iter().zip(arguments) {
            env_write.define(&param.lexeme, arg.to_owned());
        }
        drop(env_write);

        match interpreter.execute_block(&self.declaration.body, environment) {
            Ok(_) => {
                if self.is_initializer {
                    self.closure.borrow().get_at(0, "this")
                } else {
                    Ok(Value::Nil)
                }
            }
            Err(RuntimeException::Return(val)) => {
                if self.is_initializer {
                    self.closure.borrow().get_at(0, "this")
                } else {
                    Ok(val)
                }
            }
            Err(err) => Err(err),
        }
    }

    fn arity(&self) -> usize {
        self.declaration.params.len()
    }
}

impl Display for LoxFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.declaration.name.lexeme)
    }
}
