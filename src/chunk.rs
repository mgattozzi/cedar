use crate::{value::Value, CedarError};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
  Return,
  Constant,
  Negate,
  Add,
  Subtract,
  Multiply,
  Divide,
}
impl From<u8> for OpCode {
  fn from(b: u8) -> Self {
    match b {
      0 => OpCode::Return,
      1 => OpCode::Constant,
      2 => OpCode::Negate,
      3 => OpCode::Add,
      4 => OpCode::Subtract,
      5 => OpCode::Multiply,
      6 => OpCode::Divide,
      _ => panic!("Invalid opcode: {}", b),
    }
  }
}
impl From<OpCode> for u8 {
  fn from(o: OpCode) -> Self {
    match o {
      OpCode::Return => 0,
      OpCode::Constant => 1,
      OpCode::Negate => 2,
      OpCode::Add => 3,
      OpCode::Subtract => 4,
      OpCode::Multiply => 5,
      OpCode::Divide => 6,
    }
  }
}

impl fmt::Display for OpCode {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let string = match self {
      OpCode::Return => "Return",
      OpCode::Constant => "Constant",
      OpCode::Negate => "Negate",
      OpCode::Add => "Add",
      OpCode::Subtract => "Subtract",
      OpCode::Multiply => "Multiply",
      OpCode::Divide => "Divide",
    };
    write!(f, "{}", string)
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
  pub code: Vec<u8>,
  pub constants: Vec<Value>,
  pub lines: Vec<usize>,
}

impl Chunk {
  pub fn new() -> Self {
    Self {
      code: Vec::new(),
      constants: Vec::new(),
      lines: Vec::new(),
    }
  }
  pub fn write_chunk(
    &mut self,
    byte: OpCode,
    value: Option<Value>,
    line: usize,
  ) -> Result<(), CedarError> {
    match byte {
      OpCode::Return
      | OpCode::Negate
      | OpCode::Add
      | OpCode::Subtract
      | OpCode::Multiply
      | OpCode::Divide => {
        self.write_byte(byte.into());
        self.lines.push(line);
        Ok(())
      }
      OpCode::Constant => self.add_constant(value.expect("Constant should have a value"), line),
    }
  }
  pub fn write_byte(&mut self, byte: u8) {
    self.code.push(byte);
  }

  fn add_constant(&mut self, value: Value, line: usize) -> Result<(), CedarError> {
    self.constants.push(value);
    if self.constants.len() > std::u8::MAX as usize {
      return Err(ChunkError::TooManyConst.into());
    }
    self.write_byte(OpCode::Constant.into());
    self.write_byte((self.constants.len() - 1) as u8);
    // TODO: Make this work for indexing better
    // push twice to keep length for indexing the same
    self.lines.push(line);
    self.lines.push(line);
    Ok(())
  }

  #[allow(dead_code)]
  pub fn dissasemble(&self, name: &str) {
    println!("== {} ==", name);
    let mut iterator = self.code.iter().enumerate();
    while let Some((i, instruction)) = iterator.next() {
      let op = OpCode::from(*instruction);
      match op {
        OpCode::Return
        | OpCode::Negate
        | OpCode::Add
        | OpCode::Subtract
        | OpCode::Multiply
        | OpCode::Divide => println!("{:04} {:4} {}", i, self.lines[i], op),
        OpCode::Constant => {
          if let Some((_, location)) = iterator.next() {
            print!("{:04} {:4} {} {:12} ", i, self.lines[i], op, location);
            match self.constants[*location as usize] {
              Value::Number(n) => println!("'{}'", n),
            }
          }
        }
      }
    }
  }
}

#[derive(Debug)]
pub enum ChunkError {
  TooManyConst,
}
impl fmt::Display for ChunkError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Too many constants used in code")
  }
}

impl std::error::Error for ChunkError {}
