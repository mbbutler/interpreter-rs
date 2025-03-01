// use std::fmt::Display;
use std::{
    fmt::{Debug, Display},
    sync::{Arc, RwLock},
};

use super::{
    environment::Environment,
    error::RuntimeException,
    interpreter::{Interpreter, RuntimeResult},
    scanner::Token,
    stmt::Stmt,
    value::Value,
};

type LoxFnPtr = fn(&mut Interpreter, &[Value]) -> RuntimeResult<Value>;

pub trait LoxCallableFn {
    fn call(&self, interpreter: &mut Interpreter, arguments: &[Value]) -> RuntimeResult<Value>;
    fn arity(&self) -> usize;
}

#[derive(Clone, Debug)]
pub enum LoxCallable {
    Lox(LoxFn),
    Native(NativeFn),
}

impl LoxCallable {
    pub fn new_native(arity: usize, f: LoxFnPtr) -> Self {
        Self::Native(NativeFn::new(arity, f))
    }

    pub fn new_lox(
        name: Box<Token>,
        params: Vec<Token>,
        body: Vec<Stmt>,
        closure: &Arc<RwLock<Environment>>,
    ) -> Self {
        Self::Lox(LoxFn::new(name, params, body, closure))
    }
}

impl LoxCallableFn for LoxCallable {
    fn call(&self, interpreter: &mut Interpreter, arguments: &[Value]) -> RuntimeResult<Value> {
        match self {
            Self::Lox(lox_fn) => lox_fn.call(interpreter, arguments),
            Self::Native(native_fn) => native_fn.call(interpreter, arguments),
        }
    }

    fn arity(&self) -> usize {
        match self {
            Self::Lox(lox_fn) => lox_fn.arity(),
            Self::Native(native_fn) => native_fn.arity(),
        }
    }
}

impl Display for LoxCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lox(lox_fn) => write!(f, "{lox_fn}"),
            Self::Native(native_fn) => write!(f, "{native_fn}"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct LoxFn {
    name: Box<Token>,
    params: Vec<Token>,
    body: Vec<Stmt>,
    closure: Arc<RwLock<Environment>>,
}

impl LoxFn {
    pub fn new(
        name: Box<Token>,
        params: Vec<Token>,
        body: Vec<Stmt>,
        closure: &Arc<RwLock<Environment>>,
    ) -> Self {
        Self {
            name,
            params,
            body,
            closure: Arc::clone(closure),
        }
    }
}

impl LoxCallableFn for LoxFn {
    fn call(&self, interpreter: &mut Interpreter, arguments: &[Value]) -> RuntimeResult<Value> {
        let environment = Environment::new(&self.closure);
        let mut env_write = environment.write()?;
        for (param, arg) in self.params.iter().zip(arguments) {
            env_write.define(&param.lexeme, arg.to_owned());
        }
        drop(env_write);

        match interpreter.execute_block(&self.body, environment) {
            Ok(_) => Ok(Value::Nil),
            Err(RuntimeException::Return(val)) => Ok(val),
            Err(err) => Err(err),
        }
    }

    fn arity(&self) -> usize {
        self.params.len()
    }
}

impl Display for LoxFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.name.lexeme)
    }
}

#[derive(Clone, Debug)]
pub struct NativeFn {
    arity: usize,
    f: LoxFnPtr,
}

impl NativeFn {
    pub fn new(arity: usize, f: LoxFnPtr) -> Self {
        Self { arity, f }
    }
}

impl LoxCallableFn for NativeFn {
    fn call(&self, interpreter: &mut Interpreter, arguments: &[Value]) -> RuntimeResult<Value> {
        (self.f)(interpreter, arguments)
    }

    fn arity(&self) -> usize {
        self.arity
    }
}

impl Display for NativeFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native fn>")
    }
}
