mod chunk;
mod error;
mod stack;
mod value;
mod vm;

use chunk::{Chunk, Op};
use vm::VM;

fn main() {
    let mut chunk = Chunk::default();
    let constant = chunk.add_constant(1.2);
    chunk.write(Op::Constant.into(), 123);
    chunk.write(constant, 123);

    let constant = chunk.add_constant(3.4);
    chunk.write(Op::Constant.into(), 123);
    chunk.write(constant, 123);

    chunk.write(Op::Add.into(), 123);

    let constant = chunk.add_constant(5.6);
    chunk.write(Op::Constant.into(), 123);
    chunk.write(constant, 123);

    chunk.write(Op::Divide.into(), 123);
    chunk.write(Op::Negate.into(), 123);

    chunk.write(Op::Return.into(), 123);

    if let Err(err) = VM::interpret(&chunk) {
        eprintln!("Error: {err}");
    }
}
