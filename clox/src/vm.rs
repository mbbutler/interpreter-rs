use clox::binary_op;

use crate::{
    chunk::{Chunk, Op},
    error::InterpretResult,
    stack::Stack,
    value::Value,
};

pub struct VM<'a> {
    chunk: &'a Chunk,
    ip: *const u8,
    stack: Stack,
}

impl<'a> VM<'a> {
    pub fn interpret(chunk: &Chunk) -> InterpretResult<()> {
        let mut vm = VM::new(chunk);
        vm.run()
    }

    fn new(chunk: &'a Chunk) -> Self {
        let ip = chunk.as_ptr();
        Self {
            chunk,
            ip,
            stack: Stack::new(),
        }
    }

    fn run(&mut self) -> InterpretResult<()> {
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
        let byte = unsafe {
            let byte = *self.ip;
            self.ip = self.ip.add(1);
            byte
        };
        byte
    }

    fn read_op_code(&mut self) -> InterpretResult<Op> {
        Ok(self.read_byte().try_into()?)
    }

    fn read_constant(&mut self) -> Value {
        self.chunk.read_constant(self.read_byte())
    }

    fn push(&mut self, value: Value) -> InterpretResult<()> {
        self.stack.push(value)
    }

    fn pop(&mut self) -> InterpretResult<Value> {
        self.stack.pop()
    }

    fn ip_offset(&self) -> usize {
        unsafe { self.ip.offset_from(self.chunk.as_ptr()) as usize }
    }
}
