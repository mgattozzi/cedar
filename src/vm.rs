use crate::{
  chunk::{Chunk, OpCode},
  compiler::compile,
  value::Value,
  CedarError,
};
use std::{borrow::Cow, fmt};

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

  pub fn interpret(&mut self, source: String) -> Result<(), CedarError> {
    let chunk = compile(source)?;
    chunk.dissasemble("MAIN");
    self.chunk = Some(chunk);
    self.ip = 0;
    self.run()
  }

  fn run(&mut self) -> Result<(), CedarError> {
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
          let n = -self.pop().as_num().ok_or_else(|| {
            InterpretResult::runtime_error("Operand must be a number", self.line())
          })?;
          self.push(Value::Number(n));
        }
        OpCode::Add => {
          let b = self.pop().as_num().ok_or_else(|| {
            InterpretResult::runtime_error("Operand must be a number", self.line())
          })?;
          let a = self.pop().as_num().ok_or_else(|| {
            InterpretResult::runtime_error("Operand must be a number", self.line())
          })?;
          self.push(Value::Number(a + b));
        }
        OpCode::Subtract => {
          let b = self.pop().as_num().ok_or_else(|| {
            InterpretResult::runtime_error("Operand must be a number", self.line())
          })?;
          let a = self.pop().as_num().ok_or_else(|| {
            InterpretResult::runtime_error("Operand must be a number", self.line())
          })?;
          self.push(Value::Number(a - b));
        }
        OpCode::Multiply => {
          let b = self.pop().as_num().ok_or_else(|| {
            InterpretResult::runtime_error("Operand must be a number", self.line())
          })?;
          let a = self.pop().as_num().ok_or_else(|| {
            InterpretResult::runtime_error("Operand must be a number", self.line())
          })?;
          self.push(Value::Number(a * b));
        }
        OpCode::Divide => {
          let b = self.pop().as_num().ok_or_else(|| {
            InterpretResult::runtime_error("Operand must be a number", self.line())
          })?;
          let a = self.pop().as_num().ok_or_else(|| {
            InterpretResult::runtime_error("Operand must be a number", self.line())
          })?;
          self.push(Value::Number(a / b));
        }
        OpCode::False => {
          self.push(Value::Bool(false));
        }
        OpCode::True => {
          self.push(Value::Bool(true));
        }
        OpCode::Null => {
          self.push(Value::Null);
        }
        OpCode::Not => {
          let boolean = self.pop().as_bool().ok_or_else(|| {
            InterpretResult::runtime_error("Operand must be a boolean", self.line())
          })?;
          self.push(Value::Bool(!boolean));
        }
        OpCode::Equal => {
          let b = self.pop().as_bool().ok_or_else(|| {
            InterpretResult::runtime_error("Operand must be a boolean", self.line())
          })?;
          let a = self.pop().as_bool().ok_or_else(|| {
            InterpretResult::runtime_error("Operand must be a boolean", self.line())
          })?;
          self.push(Value::Bool(a == b));
        }
        OpCode::NotEqual => {
          let b = self.pop().as_bool().ok_or_else(|| {
            InterpretResult::runtime_error("Operand must be a boolean", self.line())
          })?;
          let a = self.pop().as_bool().ok_or_else(|| {
            InterpretResult::runtime_error("Operand must be a boolean", self.line())
          })?;
          self.push(Value::Bool(a != b));
        }
        OpCode::Greater => {
          let b = self.pop().as_bool().ok_or_else(|| {
            InterpretResult::runtime_error("Operand must be a boolean", self.line())
          })?;
          let a = self.pop().as_bool().ok_or_else(|| {
            InterpretResult::runtime_error("Operand must be a boolean", self.line())
          })?;
          self.push(Value::Bool(a > b));
        }
        OpCode::GreaterOrEqual => {
          let b = self.pop().as_bool().ok_or_else(|| {
            InterpretResult::runtime_error("Operand must be a boolean", self.line())
          })?;
          let a = self.pop().as_bool().ok_or_else(|| {
            InterpretResult::runtime_error("Operand must be a boolean", self.line())
          })?;
          self.push(Value::Bool(a >= b));
        }
        OpCode::Less => {
          let b = self.pop().as_bool().ok_or_else(|| {
            InterpretResult::runtime_error("Operand must be a boolean", self.line())
          })?;
          let a = self.pop().as_bool().ok_or_else(|| {
            InterpretResult::runtime_error("Operand must be a boolean", self.line())
          })?;
          self.push(Value::Bool(a < b));
        }
        OpCode::LessOrEqual => {
          let b = self.pop().as_bool().ok_or_else(|| {
            InterpretResult::runtime_error("Operand must be a boolean", self.line())
          })?;
          let a = self.pop().as_bool().ok_or_else(|| {
            InterpretResult::runtime_error("Operand must be a boolean", self.line())
          })?;
          self.push(Value::Bool(a <= b));
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
  fn push(&mut self, value: Value) {
    self.stack.push(value);
  }
  fn pop(&mut self) -> Value {
    self
      .stack
      .pop()
      .expect("VM tried to pop a value off an empty stack")
  }
  fn line(&self) -> usize {
    self
      .chunk
      .as_ref()
      .map(|c| c.lines[self.ip - 1])
      .unwrap_or(0)
  }
  #[allow(dead_code)]
  pub fn print_stack(&self) {
    println!("--- Stack ---\n{:#?}", self.stack);
  }
}

#[derive(Debug, Clone)]
pub enum InterpretResult {
  CompileError,
  RuntimeError(Cow<'static, str>, usize),
}

impl InterpretResult {
  fn runtime_error<M: Into<Cow<'static, str>>>(message: M, line: usize) -> Self {
    Self::RuntimeError(message.into(), line)
  }
}

impl fmt::Display for InterpretResult {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      InterpretResult::CompileError => write!(f, "Compilation error"),
      InterpretResult::RuntimeError(message, line) => {
        write!(f, "[line {}] Error in script: {}", line, message)
      }
    }
  }
}

impl std::error::Error for InterpretResult {}
