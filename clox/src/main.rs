mod chunk;
mod op_code;
mod value;

use chunk::Chunk;
use op_code::OpCode;

fn main() {
    let mut chunk = Chunk::default();
    let constant = chunk.add_constant(1.2);
    chunk.write(OpCode::Constant.into(), 123);
    chunk.write(constant, 123);
    chunk.write(OpCode::Return.into(), 123);
    chunk.disassemble("test chunk");
}
