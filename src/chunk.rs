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
  Null,
  True,
  False,
  Not,
  Equal,
  NotEqual,
  Greater,
  GreaterOrEqual,
  Less,
  LessOrEqual,
  Print,
  Pop,
  DefineGlobal,
  GetGlobal,
  SetGlobal,
  GetLocal,
  SetLocal,
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
      7 => OpCode::Null,
      8 => OpCode::True,
      9 => OpCode::False,
      10 => OpCode::Not,
      11 => OpCode::Equal,
      12 => OpCode::NotEqual,
      13 => OpCode::Greater,
      14 => OpCode::GreaterOrEqual,
      15 => OpCode::Less,
      16 => OpCode::LessOrEqual,
      17 => OpCode::Print,
      18 => OpCode::Pop,
      19 => OpCode::DefineGlobal,
      20 => OpCode::GetGlobal,
      21 => OpCode::SetGlobal,
      22 => OpCode::GetLocal,
      23 => OpCode::SetLocal,
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
      OpCode::Null => 7,
      OpCode::True => 8,
      OpCode::False => 9,
      OpCode::Not => 10,
      OpCode::Equal => 11,
      OpCode::NotEqual => 12,
      OpCode::Greater => 13,
      OpCode::GreaterOrEqual => 14,
      OpCode::Less => 15,
      OpCode::LessOrEqual => 16,
      OpCode::Print => 17,
      OpCode::Pop => 18,
      OpCode::DefineGlobal => 19,
      OpCode::GetGlobal => 20,
      OpCode::SetGlobal => 21,
      OpCode::GetLocal => 22,
      OpCode::SetLocal => 23,
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
      OpCode::Null => "null",
      OpCode::True => "true",
      OpCode::False => "false",
      OpCode::Not => "Not",
      OpCode::Equal => "Equal",
      OpCode::NotEqual => "NotEqual",
      OpCode::Greater => "Greater",
      OpCode::GreaterOrEqual => "GreaterOrEqual",
      OpCode::Less => "Less",
      OpCode::LessOrEqual => "LessOrEqual",
      OpCode::Print => "Print",
      OpCode::Pop => "Pop",
      OpCode::DefineGlobal => "DefineGlobal",
      OpCode::GetGlobal => "GetGlobal",
      OpCode::SetGlobal => "SetGlobal",
      OpCode::GetLocal => "GetLocal",
      OpCode::SetLocal => "SetLocal",
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
      | OpCode::Not
      | OpCode::Null
      | OpCode::True
      | OpCode::False
      | OpCode::Equal
      | OpCode::NotEqual
      | OpCode::Greater
      | OpCode::GreaterOrEqual
      | OpCode::Less
      | OpCode::LessOrEqual
      | OpCode::Print
      | OpCode::Pop
      | OpCode::Divide => {
        self.write_byte(byte.into());
        self.lines.push(line);
        Ok(())
      }
      OpCode::Constant => self.add_constant(value.expect("Constant should have a value"), line),
      OpCode::DefineGlobal => {
        self.add_global(value.expect("Global variable should have a value"), line)
      }
      OpCode::GetGlobal => self.add_get_global(
        value.expect("Global variable ref should have a value"),
        line,
      ),
      OpCode::SetGlobal => self.add_set_global(
        value.expect("Global variable ref should have a value"),
        line,
      ),
      OpCode::GetLocal => {
        self.add_get_local(value.expect("Local variable ref should have a value"), line)
      }
      OpCode::SetLocal => {
        self.add_set_local(value.expect("Local variable ref should have a value"), line)
      }
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

  fn add_global(&mut self, value: Value, line: usize) -> Result<(), CedarError> {
    self.constants.push(value);
    if self.constants.len() > std::u8::MAX as usize {
      return Err(ChunkError::TooManyConst.into());
    }
    self.write_byte(OpCode::DefineGlobal.into());
    self.write_byte((self.constants.len() - 1) as u8);
    // TODO: Make this work for indexing better
    // push twice to keep length for indexing the same
    self.lines.push(line);
    self.lines.push(line);
    Ok(())
  }

  fn add_get_global(&mut self, value: Value, line: usize) -> Result<(), CedarError> {
    match self
      .constants
      .iter()
      .enumerate()
      .find_map(|(i, c)| if *c == value { Some(i) } else { None })
    {
      None => {
        self.constants.push(value);
        if self.constants.len() > std::u8::MAX as usize {
          return Err(ChunkError::TooManyConst.into());
        }
        self.write_byte(OpCode::GetGlobal.into());
        self.write_byte((self.constants.len() - 1) as u8);
      }
      Some(b) => {
        self.write_byte(OpCode::GetGlobal.into());
        self.write_byte(b as u8);
      }
    }
    // TODO: Make this work for indexing better
    // push twice to keep length for indexing the same
    self.lines.push(line);
    self.lines.push(line);
    Ok(())
  }

  fn add_set_global(&mut self, value: Value, line: usize) -> Result<(), CedarError> {
    match self
      .constants
      .iter()
      .enumerate()
      .find_map(|(i, c)| if *c == value { Some(i) } else { None })
    {
      None => {
        self.constants.push(value);
        if self.constants.len() > std::u8::MAX as usize {
          return Err(ChunkError::TooManyConst.into());
        }
        self.write_byte(OpCode::SetGlobal.into());
        self.write_byte((self.constants.len() - 1) as u8);
      }
      Some(b) => {
        self.write_byte(OpCode::SetGlobal.into());
        self.write_byte(b as u8);
      }
    }
    // TODO: Make this work for indexing better
    // push twice to keep length for indexing the same
    self.lines.push(line);
    self.lines.push(line);
    Ok(())
  }
  fn add_get_local(&mut self, value: Value, line: usize) -> Result<(), CedarError> {
    self.write_byte(OpCode::GetLocal.into());
    let value = value.into_byte();
    // TODO: Make this work for indexing better
    // push twice to keep length for indexing the same
    self.lines.push(line);
    self.lines.push(line);
    // We checked for truncation here
    Ok(self.write_byte(value as u8))
  }
  fn add_set_local(&mut self, value: Value, line: usize) -> Result<(), CedarError> {
    self.write_byte(OpCode::SetLocal.into());
    let value = value.into_byte();
    // TODO: Make this work for indexing better
    // push twice to keep length for indexing the same
    self.lines.push(line);
    self.lines.push(line);
    // We checked for truncation here
    Ok(self.write_byte(value as u8))
  }
  #[allow(dead_code)]
  pub fn disassemble(&self, name: &str) {
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
        | OpCode::Divide
        | OpCode::Not
        | OpCode::Null
        | OpCode::False
        | OpCode::Equal
        | OpCode::NotEqual
        | OpCode::Greater
        | OpCode::GreaterOrEqual
        | OpCode::Less
        | OpCode::LessOrEqual
        | OpCode::Print
        | OpCode::Pop
        | OpCode::True => println!("{:04} {:4} {}", i, self.lines[i], op),
        OpCode::Constant => {
          if let Some((_, location)) = iterator.next() {
            print!("{:04} {:4} {}{:12} ", i, self.lines[i], op, location);
            match &self.constants[*location as usize] {
              Value::Number(n) => println!("'{}'", n),
              Value::Bool(b) => println!("'{}'", b),
              Value::Byte(b) => println!("'{}'", b),
              Value::String(s) => println!("'{}'", s),
              Value::Heap(h) => println!("'heap {}'", h),
              Value::Null => println!("'null'"),
            }
          }
        }
        OpCode::DefineGlobal => {
          if let Some((_, location)) = iterator.next() {
            print!("{:04} {:4} {}{:8} ", i, self.lines[i], op, location);
            match &self.constants[*location as usize] {
              Value::Number(n) => println!("'{}'", n),
              Value::Bool(b) => println!("'{}'", b),
              Value::Byte(b) => println!("'{}'", b),
              Value::String(s) => println!("'{}'", s),
              Value::Heap(h) => println!("'heap {}'", h),
              Value::Null => println!("'null'"),
            }
          }
        }
        OpCode::GetGlobal => {
          if let Some((_, location)) = iterator.next() {
            print!("{:04} {:4} {}{:11} ", i, self.lines[i], op, location);
            match &self.constants[*location as usize] {
              Value::Number(n) => println!("'{}'", n),
              Value::Bool(b) => println!("'{}'", b),
              Value::Byte(b) => println!("'{}'", b),
              Value::String(s) => println!("'{}'", s),
              Value::Heap(h) => println!("'heap {}'", h),
              Value::Null => println!("'null'"),
            }
          }
        }
        OpCode::SetGlobal => {
          if let Some((_, location)) = iterator.next() {
            print!("{:04} {:4} {}{:11} ", i, self.lines[i], op, location);
            match &self.constants[*location as usize] {
              Value::Number(n) => println!("'{}'", n),
              Value::Bool(b) => println!("'{}'", b),
              Value::Byte(b) => println!("'{}'", b),
              Value::String(s) => println!("'{}'", s),
              Value::Heap(h) => println!("'heap {}'", h),
              Value::Null => println!("'null'"),
            }
          }
        }
        OpCode::GetLocal => {
          if let Some((_, location)) = iterator.next() {
            println!("{:04} {:4} {}{:12} ", i, self.lines[i], op, location);
          }
        }
        OpCode::SetLocal => {
          if let Some((_, location)) = iterator.next() {
            println!("{:04} {:4} {}{:12} ", i, self.lines[i], op, location);
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
    match self {
      ChunkError::TooManyConst => write!(f, "Too many constants used in code"),
    }
  }
}

impl std::error::Error for ChunkError {}
