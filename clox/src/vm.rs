use std::sync::LazyLock;

use clox::binary_op;

use crate::{
    chunk::{Chunk, Op},
    compiler::Compiler,
    error::Result,
    stack::Stack,
    value::Value,
};

pub struct VM<'a> {
    chunk: &'a Chunk,
    ip: *const u8,
    stack: Stack,
}

static DEFAULT_CHUNK: LazyLock<Chunk> = LazyLock::new(Chunk::default);

impl<'a> VM<'a> {
    pub fn interpret(&mut self, input: &str) -> Result<()> {
        Compiler::compile(input)?;
        todo!()
    }

    fn new(chunk: &'a Chunk) -> Self {
        let ip = chunk.as_ptr();
        Self {
            chunk,
            ip,
            stack: Stack::new(),
        }
    }

    fn run(&mut self) -> Result<()> {
        loop {
            #[cfg(debug_assertions)]
            {
                print!("          ");
                for val in self.stack.iter() {
                    print!("[ {val} ]");
                }
                println!();
                self.chunk.disassemble_instruction(self.ip_offset());
            }

            let instruction: Op = self.read_op_code()?;
            match instruction {
                Op::Constant => {
                    let constant = self.read_constant();
                    self.push(constant)?;
                }
                Op::Add => binary_op!(self, +),
                Op::Subtract => binary_op!(self, -),
                Op::Multiply => binary_op!(self, *),
                Op::Divide => binary_op!(self, /),
                Op::Negate => {
                    let val = -self.pop()?;
                    self.push(val)?;
                }
                Op::Return => {
                    println!("{}", self.pop()?);
                    return Ok(());
                }
            }
        }
    }

    fn read_byte(&mut self) -> u8 {
        unsafe {
            let byte = *self.ip;
            self.ip = self.ip.add(1);
            byte
        }
    }

    fn read_op_code(&mut self) -> Result<Op> {
        Ok(self.read_byte().try_into()?)
    }

    fn read_constant(&mut self) -> Value {
        self.chunk.read_constant(self.read_byte())
    }

    fn push(&mut self, value: Value) -> Result<()> {
        self.stack.push(value)
    }

    fn pop(&mut self) -> Result<Value> {
        self.stack.pop()
    }

    fn ip_offset(&self) -> usize {
        unsafe { self.ip.offset_from(self.chunk.as_ptr()) as usize }
    }
}

impl Default for VM<'_> {
    fn default() -> Self {
        Self {
            chunk: &DEFAULT_CHUNK,
            ip: DEFAULT_CHUNK.as_ptr(),
            stack: Stack::default(),
        }
    }
}
