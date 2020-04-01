use crate::{
  chunk::{Chunk, OpCode},
  compiler::compile,
  value::Value,
  CedarError,
};
use std::{
  borrow::{Borrow, Cow},
  collections::HashMap,
  fmt,
};

pub struct VM {
  chunk: Option<Chunk>,
  ip: usize,
  stack: Vec<Value>,
  heap: Vec<(Value, bool)>,
  globals: HashMap<Cow<'static, str>, Value>,
}

impl VM {
  pub fn new() -> Self {
    Self {
      chunk: None,
      ip: 0,
      stack: Vec::new(),
      heap: Vec::new(),
      globals: HashMap::new(),
    }
  }

  pub fn interpret(&mut self, source: String) -> Result<(), CedarError> {
    let chunk = compile(source)?;
    // chunk.disassemble("MAIN");
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
          // Collect garbage on exit not that it matters much
          self.collect_garbage();
          // self.debug();
          return Ok(());
        }
        OpCode::Constant => {
          let constant = self.read_constant();
          self.push(constant);
        }
        OpCode::Negate => {
          let n = -self
            .pop()
            .ok_or_else(|| InterpreterResult::no_stack_value())?
            .as_num()
            .ok_or_else(|| {
              InterpreterResult::runtime_error("Operand must be a number", self.line())
            })?;
          self.push(Value::Number(n));
        }
        OpCode::Add => {
          let b = self
            .pop()
            .ok_or_else(|| InterpreterResult::no_stack_value())?;
          let a = self
            .pop()
            .ok_or_else(|| InterpreterResult::no_stack_value())?;
          match (b, a) {
            (Value::Number(b), Value::Number(a)) => self.push(Value::Number(a + b)),
            (Value::String(b), Value::String(a)) => {
              let mut string = a.clone();
              string.to_mut().push_str(b.borrow());
              self.heap.push((Value::String(string), false));
              self.push(Value::Heap(self.heap.len() - 1));
            }
            (Value::Number(b), Value::String(a)) => {
              let mut string = a.clone();
              string.to_mut().push_str(&b.to_string());
              self.heap.push((Value::String(string), false));
              self.push(Value::Heap(self.heap.len() - 1));
            }
            (Value::Bool(b), Value::String(a)) => {
              let mut string = a.clone();
              string.to_mut().push_str(&b.to_string());
              self.heap.push((Value::String(string), false));
              self.push(Value::Heap(self.heap.len() - 1));
            }
            (Value::Null, Value::String(a)) => {
              let mut string = a.clone();
              string.to_mut().push_str("null");
              self.heap.push((Value::String(string), false));
              self.push(Value::Heap(self.heap.len() - 1));
            }
            (_, Value::Number(_)) => {
              return Err(
                InterpreterResult::runtime_error("Second operand is not a number", self.line())
                  .into(),
              )
            }
            (Value::Number(_), _) => {
              return Err(
                InterpreterResult::runtime_error("First operand is not a number", self.line()).into(),
              )
            }
            (_, _) => {
              return Err(
                InterpreterResult::runtime_error(
                  "Addition operator can only be used with 2 number values or a String and another value",
                  self.line(),
                )
                .into(),
              )
            }
          }
        }
        OpCode::Subtract => {
          let b = self
            .pop()
            .ok_or_else(|| InterpreterResult::no_stack_value())?
            .as_num()
            .ok_or_else(|| {
              InterpreterResult::runtime_error("Operand must be a number", self.line())
            })?;
          let a = self
            .pop()
            .ok_or_else(|| InterpreterResult::no_stack_value())?
            .as_num()
            .ok_or_else(|| {
              InterpreterResult::runtime_error("Operand must be a number", self.line())
            })?;
          self.push(Value::Number(a - b));
        }
        OpCode::Multiply => {
          let b = self
            .pop()
            .ok_or_else(|| InterpreterResult::no_stack_value())?
            .as_num()
            .ok_or_else(|| {
              InterpreterResult::runtime_error("Operand must be a number", self.line())
            })?;
          let a = self
            .pop()
            .ok_or_else(|| InterpreterResult::no_stack_value())?
            .as_num()
            .ok_or_else(|| {
              InterpreterResult::runtime_error("Operand must be a number", self.line())
            })?;
          self.push(Value::Number(a * b));
        }
        OpCode::Divide => {
          let b = self
            .pop()
            .ok_or_else(|| InterpreterResult::no_stack_value())?
            .as_num()
            .ok_or_else(|| {
              InterpreterResult::runtime_error("Operand must be a number", self.line())
            })?;
          let a = self
            .pop()
            .ok_or_else(|| InterpreterResult::no_stack_value())?
            .as_num()
            .ok_or_else(|| {
              InterpreterResult::runtime_error("Operand must be a number", self.line())
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
          let boolean = self
            .pop()
            .ok_or_else(|| InterpreterResult::no_stack_value())?
            .as_bool()
            .ok_or_else(|| {
              InterpreterResult::runtime_error("Operand must be a boolean", self.line())
            })?;
          self.push(Value::Bool(!boolean));
        }
        OpCode::Equal => {
          let b = self
            .pop()
            .ok_or_else(|| InterpreterResult::no_stack_value())?;
          let a = self
            .pop()
            .ok_or_else(|| InterpreterResult::no_stack_value())?;
          match (b, a) {
            (Value::Bool(b), Value::Bool(a)) => self.push(Value::Bool(a == b)),
            (Value::String(b), Value::String(a)) => self.push(Value::Bool(a == b)),
            (_, Value::String(_)) => {
              return Err(
                InterpreterResult::runtime_error("Second operand is not a String", self.line())
                  .into(),
              )
            }
            (Value::String(_), _) => {
              return Err(
                InterpreterResult::runtime_error("First operand is not a String", self.line())
                  .into(),
              )
            }
            (_, Value::Bool(_)) => {
              return Err(
                InterpreterResult::runtime_error("Second operand is not a boolean", self.line())
                  .into(),
              )
            }
            (Value::Bool(_), _) => {
              return Err(
                InterpreterResult::runtime_error("First operand is not a boolean", self.line())
                  .into(),
              )
            }
            (_, _) => {
              return Err(
                InterpreterResult::runtime_error(
                  "Equality operator can only be used with 2 Strings or 2 boolean values",
                  self.line(),
                )
                .into(),
              )
            }
          }
        }
        OpCode::NotEqual => {
          let b = self
            .pop()
            .ok_or_else(|| InterpreterResult::no_stack_value())?
            .as_bool()
            .ok_or_else(|| {
              InterpreterResult::runtime_error("Operand must be a boolean", self.line())
            })?;
          let a = self
            .pop()
            .ok_or_else(|| InterpreterResult::no_stack_value())?
            .as_bool()
            .ok_or_else(|| {
              InterpreterResult::runtime_error("Operand must be a boolean", self.line())
            })?;
          self.push(Value::Bool(a != b));
        }
        OpCode::Greater => {
          let b = self
            .pop()
            .ok_or_else(|| InterpreterResult::no_stack_value())?
            .as_bool()
            .ok_or_else(|| {
              InterpreterResult::runtime_error("Operand must be a boolean", self.line())
            })?;
          let a = self
            .pop()
            .ok_or_else(|| InterpreterResult::no_stack_value())?
            .as_bool()
            .ok_or_else(|| {
              InterpreterResult::runtime_error("Operand must be a boolean", self.line())
            })?;
          self.push(Value::Bool(a > b));
        }
        OpCode::GreaterOrEqual => {
          let b = self
            .pop()
            .ok_or_else(|| InterpreterResult::no_stack_value())?
            .as_bool()
            .ok_or_else(|| {
              InterpreterResult::runtime_error("Operand must be a boolean", self.line())
            })?;
          let a = self
            .pop()
            .ok_or_else(|| InterpreterResult::no_stack_value())?
            .as_bool()
            .ok_or_else(|| {
              InterpreterResult::runtime_error("Operand must be a boolean", self.line())
            })?;
          self.push(Value::Bool(a >= b));
        }
        OpCode::Less => {
          let b = self
            .pop()
            .ok_or_else(|| InterpreterResult::no_stack_value())?
            .as_bool()
            .ok_or_else(|| {
              InterpreterResult::runtime_error("Operand must be a boolean", self.line())
            })?;
          let a = self
            .pop()
            .ok_or_else(|| InterpreterResult::no_stack_value())?
            .as_bool()
            .ok_or_else(|| {
              InterpreterResult::runtime_error("Operand must be a boolean", self.line())
            })?;
          self.push(Value::Bool(a < b));
        }
        OpCode::LessOrEqual => {
          let b = self
            .pop()
            .ok_or_else(|| InterpreterResult::no_stack_value())?
            .as_bool()
            .ok_or_else(|| {
              InterpreterResult::runtime_error("Operand must be a boolean", self.line())
            })?;
          let a = self
            .pop()
            .ok_or_else(|| InterpreterResult::no_stack_value())?
            .as_bool()
            .ok_or_else(|| {
              InterpreterResult::runtime_error("Operand must be a boolean", self.line())
            })?;
          self.push(Value::Bool(a <= b));
        }
        OpCode::Print => {
          println!(
            "{}",
            self
              .pop()
              .ok_or_else(|| InterpreterResult::no_stack_value())?
          );
        }
        OpCode::Pop => {
          // We don't care if we pop an empty stack here
          self.pop();
        }
        OpCode::DefineGlobal => {
          let name = self.read_constant().as_string().ok_or_else(|| {
            InterpreterResult::runtime_error(
              "The identifier being used was not a string and is an internal runtime error",
              self.line(),
            )
          })?;
          let value = self
            .pop()
            .ok_or_else(|| InterpreterResult::no_stack_value())?;
          self.globals.insert(name, value);
        }
        OpCode::GetGlobal => {
          let name = self.read_constant().as_string().ok_or_else(|| {
            InterpreterResult::runtime_error(
              "The identifier being used was not a string and is an internal runtime error",
              self.line(),
            )
          })?;
          self.push(
            self
              .globals
              .get(&name)
              .ok_or_else(|| {
                InterpreterResult::runtime_error(
                  format!("Undefined variable '{}'", name),
                  self.line(),
                )
              })?
              .clone(),
          );
        }
        OpCode::SetGlobal => {
          let name = self.read_constant().as_string().ok_or_else(|| {
            InterpreterResult::runtime_error(
              "The identifier being used was not a string and is an internal runtime error",
              self.line(),
            )
          })?;
          let value = self
            .pop()
            .ok_or_else(|| InterpreterResult::no_stack_value())?;
          self.globals.insert(name, value);
        }
      }
    }
  }
  // To do make this not hot garbage
  fn collect_garbage(&mut self) {
    for i in &self.stack {
      match i {
        Value::Heap(h) => {
          let mut pointer = *h;
          while {
            self.heap[pointer].1 = true;
            if let Value::Heap(inner) = self.heap[pointer].0 {
              pointer = inner;
              true
            } else {
              false
            }
          } {}
        }
        _ => (),
      }
    }
    // There's nothing to collect so return early
    if self.heap.len() == 0 {
      return;
    }
    let old_max_index = self.heap.len() - 1;
    self.heap.retain(|(_, alive)| *alive);
    let offset = old_max_index - (self.heap.len().checked_sub(1).unwrap_or(0));
    self.heap.iter_mut().for_each(|(h, alive)| {
      if let Value::Heap(inner) = h {
        *inner = *inner - offset;
      }
      *alive = false
    });
    self.stack.iter_mut().for_each(|h| {
      if let Value::Heap(inner) = h {
        *inner = *inner - offset;
      }
    });
  }
  fn chunk(&self) -> &Chunk {
    self.chunk.as_ref().unwrap()
  }
  fn read_instruction(&self) -> OpCode {
    self.chunk.as_ref().unwrap().code[self.ip].into()
  }
  fn read_constant(&mut self) -> Value {
    let value = self.chunk().constants[self.chunk().code[self.ip] as usize].clone();
    self.ip += 1;
    value
  }
  fn push(&mut self, value: Value) {
    self.stack.push(value);
  }
  fn pop(&mut self) -> Option<Value> {
    let value = self.stack.pop()?;

    // This could be so so much better but I'm not really stuck
    // on a gc design or inneficiencies yet. Maybe I could use
    // Cow here idk
    if let Value::Heap(h) = value {
      Some(self.heap[h].0.clone())
    } else {
      Some(value)
    }
  }
  fn line(&self) -> usize {
    self
      .chunk
      .as_ref()
      .map(|c| c.lines[self.ip - 1])
      .unwrap_or(0)
  }
  #[allow(dead_code)]
  pub fn debug(&self) {
    self.print_globals();
    self.print_stack();
    self.print_heap();
  }
  #[allow(dead_code)]
  pub fn print_globals(&self) {
    println!("--- Globals ---\n{:#?}", self.globals);
  }
  #[allow(dead_code)]
  pub fn print_stack(&self) {
    println!("--- Stack ---\n{:#?}", self.stack);
  }
  #[allow(dead_code)]
  pub fn print_heap(&self) {
    println!("--- Heap ---\n{:#?}", self.heap);
  }
}

#[derive(Debug, Clone)]
pub enum InterpreterResult {
  CompileError(Cow<'static, str>),
  RuntimeError(Cow<'static, str>, usize),
}

impl InterpreterResult {
  fn runtime_error<M: Into<Cow<'static, str>>>(message: M, line: usize) -> Self {
    Self::RuntimeError(message.into(), line)
  }
  fn no_stack_value() -> Self {
    Self::CompileError("Tried to pop a value off an empty stack".into())
  }
}

impl fmt::Display for InterpreterResult {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      InterpreterResult::CompileError(message) => {
        write!(f, "[ICE] Error in compilation: {}", message)
      }
      InterpreterResult::RuntimeError(message, line) => {
        write!(f, "[line {}] Error in script: {}", line, message)
      }
    }
  }
}

impl std::error::Error for InterpreterResult {}
