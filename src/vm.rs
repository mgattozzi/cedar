use crate::{
  chunk::{Chunk, OpCode},
  value::Value,
};
use std::fmt;

pub struct VM {
  chunk: Option<Chunk>,
  ip: usize,
  stack: Vec<Value>,
}

impl VM {
  pub fn new() -> Self {
    Self {
      chunk: None,
      ip: 0,
      stack: Vec::new(),
    }
  }

  pub fn interpret(&mut self, chunk: Chunk) -> Result<(), InterpretResult> {
    self.chunk = Some(chunk);
    self.ip = 0;
    self.run()
  }

  fn run(&mut self) -> Result<(), InterpretResult> {
    loop {
      let op = self.read_instruction();
      self.ip += 1;
      match op {
        OpCode::Return => {
          self.print_stack();
          return Ok(());
        }
        OpCode::Constant => {
          self.push(self.read_constant());
          // Offset the pointer to the next instruction, not the constant value
          self.ip += 1;
        }
        OpCode::Negate => {
          let n = -self
            .pop()
            .as_num()
            .ok_or_else(|| InterpretResult::RuntimeError)?;
          self.push(Value::Number(n));
        }
        OpCode::Add => {
          let b = self
            .pop()
            .as_num()
            .ok_or_else(|| InterpretResult::RuntimeError)?;
          let a = self
            .pop()
            .as_num()
            .ok_or_else(|| InterpretResult::RuntimeError)?;
          self.push(Value::Number(a + b));
        }
        OpCode::Subtract => {
          let b = self
            .pop()
            .as_num()
            .ok_or_else(|| InterpretResult::RuntimeError)?;
          let a = self
            .pop()
            .as_num()
            .ok_or_else(|| InterpretResult::RuntimeError)?;
          self.push(Value::Number(a - b));
        }
        OpCode::Multiply => {
          let b = self
            .pop()
            .as_num()
            .ok_or_else(|| InterpretResult::RuntimeError)?;
          let a = self
            .pop()
            .as_num()
            .ok_or_else(|| InterpretResult::RuntimeError)?;
          self.push(Value::Number(a * b));
        }
        OpCode::Divide => {
          let b = self
            .pop()
            .as_num()
            .ok_or_else(|| InterpretResult::RuntimeError)?;
          let a = self
            .pop()
            .as_num()
            .ok_or_else(|| InterpretResult::RuntimeError)?;
          self.push(Value::Number(a / b));
        }
      }
    }
  }
  fn chunk(&self) -> &Chunk {
    self.chunk.as_ref().unwrap()
  }
  fn read_instruction(&self) -> OpCode {
    self.chunk.as_ref().unwrap().code[self.ip].into()
  }
  fn read_constant(&self) -> Value {
    self.chunk().constants[self.chunk().code[self.ip] as usize].clone()
  }
  fn reset_stack(&mut self) {
    self.stack.clear();
  }
  fn push(&mut self, value: Value) {
    self.stack.push(value);
  }
  fn pop(&mut self) -> Value {
    self
      .stack
      .pop()
      .expect("VM tried to pop a value off an empty stack")
  }
  #[allow(dead_code)]
  pub fn print_stack(&self) {
    println!("--- Stack ---\n{:#?}", self.stack);
  }
}

#[derive(Debug, Copy, Clone)]
pub enum InterpretResult {
  CompileError,
  RuntimeError,
}

impl fmt::Display for InterpretResult {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      InterpretResult::CompileError => write!(f, "Compilation error"),
      InterpretResult::RuntimeError => write!(f, "Runtime error"),
    }
  }
}

impl std::error::Error for InterpretResult {}
