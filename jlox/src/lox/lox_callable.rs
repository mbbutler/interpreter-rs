// use std::fmt::Display;
use std::{
    cell::RefCell,
    fmt::{Debug, Display},
    rc::Rc,
};

use super::{
    environment::Environment,
    interpreter::{Interpreter, RuntimeResult},
    lox_function::LoxFunction,
    stmt::Function,
    value::Value,
};

type LoxFunctionPtr = fn(&mut Interpreter, &[Value]) -> RuntimeResult<Value>;

pub trait LoxCallable {
    fn call(&self, interpreter: &mut Interpreter, arguments: &[Value]) -> RuntimeResult<Value>;
    fn arity(&self) -> usize;
}

#[derive(Clone, Debug)]
pub enum CallableFn {
    Lox(LoxFunction),
    Native(NativeFn),
}

impl CallableFn {
    pub fn new_native(arity: usize, f: LoxFunctionPtr) -> Self {
        Self::Native(NativeFn::new(arity, f))
    }

    pub fn new_lox(
        declaration: &Function,
        closure: &Rc<RefCell<Environment>>,
        is_initializer: bool,
    ) -> Self {
        Self::Lox(LoxFunction::new(declaration, closure, is_initializer))
    }
}

impl LoxCallable for CallableFn {
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

impl Display for CallableFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lox(lox_fn) => write!(f, "{lox_fn}"),
            Self::Native(native_fn) => write!(f, "{native_fn}"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct NativeFn {
    arity: usize,
    f: LoxFunctionPtr,
}

impl NativeFn {
    pub fn new(arity: usize, f: LoxFunctionPtr) -> Self {
        Self { arity, f }
    }
}

impl LoxCallable for NativeFn {
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
