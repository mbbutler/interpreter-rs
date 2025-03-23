use crate::{
    error::{InterpretError, InterpretResult},
    value::Value,
};

const STACK_MAX: usize = 256;

pub struct Stack {
    data: [Value; STACK_MAX],
    offset: usize,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            data: [0.0; STACK_MAX],
            offset: 0,
        }
    }

    pub fn push(&mut self, value: Value) -> InterpretResult<()> {
        if self.offset >= STACK_MAX {
            Err(InterpretError::RuntimeError(String::from(
                "Stack overflow.",
            )))
        } else {
            self.data[self.offset] = value;
            self.offset += 1;
            Ok(())
        }
    }

    pub fn pop(&mut self) -> InterpretResult<Value> {
        if self.offset <= 0 {
            Err(InterpretError::RuntimeError(String::from(
                "Stack underflow.",
            )))
        } else {
            self.offset -= 1;
            Ok(self.data[self.offset])
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Value> {
        self.data.iter().take(self.offset)
    }
}
