mod chunk;
mod value;
mod vm;

use chunk::*;
use value::*;
use vm::*;

fn main() {
  let mut chunk = Chunk::new();
  chunk.write_chunk(OpCode::Constant, Some(Value::Number(1.2)), 0);
  chunk.write_chunk(OpCode::Constant, Some(Value::Number(3.4)), 0);
  chunk.write_chunk(OpCode::Add, None, 0);
  chunk.write_chunk(OpCode::Constant, Some(Value::Number(5.6)), 0);
  chunk.write_chunk(OpCode::Divide, None, 0);
  chunk.write_chunk(OpCode::Negate, None, 0);
  chunk.write_chunk(OpCode::Return, None, 0);
  let mut vm = VM::new();
  if let Err(e) = vm.interpret(chunk) {
    eprintln!("{}", e);
  }
}
