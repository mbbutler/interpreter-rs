use crate::{op_code::OpCode, value::Value};

#[derive(Default)]
pub struct Chunk {
    code: Vec<u8>,
    lines: Vec<usize>,
    constants: Vec<Value>,
}

impl Chunk {
    pub fn write(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, value: Value) -> u8 {
        self.constants.push(value);
        (self.constants.len() - 1) as u8
    }

    pub fn free(&mut self) {
        self.code.clear();
        self.code.shrink_to_fit();
        self.lines.clear();
        self.lines.shrink_to_fit();
        self.constants.clear();
        self.constants.shrink_to_fit();
    }

    pub fn disassemble(&self, name: &str) {
        println!("== {name} ==");
        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset);
        }
    }

    fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{offset:0>4} ");
        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            print!("   | ");
        } else {
            print!("{:>4} ", self.lines[offset]);
        }
        let instruction = self.code[offset];
        match OpCode::try_from(instruction) {
            Ok(opcode) => match opcode {
                OpCode::Constant => self.constant_instruction("OP_CONSTANT", offset),
                OpCode::Return => self.simple_instruction("OP_RETURN", offset),
            },
            Err(_) => {
                println!("Unknown opcode value: {instruction}");
                offset + 1
            }
        }
    }

    fn constant_instruction(&self, name: &str, offset: usize) -> usize {
        let constant = self.code[offset + 1] as usize;
        println!("{name:<16} {constant:>4} '{}'", self.constants[constant]);
        offset + 2
    }

    fn simple_instruction(&self, name: &str, offset: usize) -> usize {
        println!("{name}");
        offset + 1
    }
}
