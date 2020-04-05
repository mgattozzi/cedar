use crate::{
  chunk::{Chunk, OpCode},
  compiler::compile,
  native::{my_func, NativeFuncHolder},
  value::{Function, Value},
  CedarError,
};
use std::{
  borrow::{Borrow, Cow},
  collections::HashMap,
  fmt,
  rc::Rc,
};

pub struct VM {
  frames: Vec<CallFrame>,
  frame_count: usize,
  stack: Vec<Value>,
  heap: Vec<(Value, bool)>,
  globals: HashMap<Cow<'static, str>, Value>,
}

impl VM {
  pub fn new() -> Self {
    let mut new = Self {
      frames: Vec::new(),
      frame_count: 0,
      stack: Vec::new(),
      heap: Vec::new(),
      globals: HashMap::new(),
    };

    new.define_natives();

    new
  }

  fn define_natives(&mut self) {
    let my_func_p: fn(f64, f64) -> f64 = my_func;

    self.globals.insert(
      "native-fn".into(),
      Value::NativeFn(NativeFuncHolder {
        inner: Rc::new(my_func_p),
      }),
    );
  }

  pub fn interpret(&mut self, source: String) -> Result<(), CedarError> {
    let function = compile(source)?;
    //function.chunk.disassemble("MAIN");
    self.call(function.clone(), 0)?;
    self.stack.push(Value::Function(function));
    self.run()
  }

  fn ip(&mut self) -> &mut usize {
    &mut self.frames[self.frame_count - 1].ip
  }
  fn ip_(&self) -> usize {
    self.frames[self.frame_count - 1].ip
  }
  fn slots(&self) -> usize {
    self.frames[self.frame_count - 1].slots
  }

  fn run(&mut self) -> Result<(), CedarError> {
    loop {
      let op = self.read_instruction();
      *self.ip() += 1;
      match op {
        OpCode::Return => {
          // Collect garbage on exit not that it matters much
          self.collect_garbage();
          let result = self.pop();
          self.frames.pop();
          self.frame_count -= 1;
          if self.frame_count == 0 {
            self.pop();
            return Ok(());
          }
          self.push(result);
        }
        OpCode::Constant => {
          let constant = self.read_constant();
          self.push(constant);
        }
        OpCode::Negate => {
          let n = -self.pop().as_num().ok_or_else(|| {
            InterpreterResult::runtime_error("Operand must be a number", self.line())
          })?;
          self.push(Value::Number(n));
        }
        OpCode::Add => {
          let b = self.pop();
          let a = self.pop();
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
          let b = self.pop().as_num().ok_or_else(|| {
            InterpreterResult::runtime_error("Operand must be a number", self.line())
          })?;
          let a = self.pop().as_num().ok_or_else(|| {
            InterpreterResult::runtime_error("Operand must be a number", self.line())
          })?;
          self.push(Value::Number(a - b));
        }
        OpCode::Multiply => {
          let b = self.pop().as_num().ok_or_else(|| {
            InterpreterResult::runtime_error("Operand must be a number", self.line())
          })?;
          let a = self.pop().as_num().ok_or_else(|| {
            InterpreterResult::runtime_error("Operand must be a number", self.line())
          })?;
          self.push(Value::Number(a * b));
        }
        OpCode::Divide => {
          let b = self.pop().as_num().ok_or_else(|| {
            InterpreterResult::runtime_error("Operand must be a number", self.line())
          })?;
          let a = self.pop().as_num().ok_or_else(|| {
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
          let boolean = self.pop().as_bool().ok_or_else(|| {
            InterpreterResult::runtime_error("Operand must be a boolean", self.line())
          })?;
          self.push(Value::Bool(!boolean));
        }
        OpCode::Equal => {
          let b = self.pop();
          let a = self.pop();
          match (b, a) {
            (Value::Bool(b), Value::Bool(a)) => self.push(Value::Bool(a == b)),
            (Value::String(b), Value::String(a)) => self.push(Value::Bool(a == b)),
            (Value::Number(b), Value::Number(a)) => self.push(Value::Bool(a == b)),
            (Value::Null, Value::Null) => self.push(Value::Bool(true)),
            (_, Value::Null) => self.push(Value::Bool(false)),
            (Value::Null, _) => self.push(Value::Bool(false)),
            (_, _) => {
              return Err(
                InterpreterResult::runtime_error(
                  "Equality operator can only be used with 2 of the same type",
                  self.line(),
                )
                .into(),
              )
            }
          }
        }
        OpCode::NotEqual => {
          let b = self.pop();
          let a = self.pop();
          match (b, a) {
            (Value::Bool(b), Value::Bool(a)) => self.push(Value::Bool(a != b)),
            (Value::String(b), Value::String(a)) => self.push(Value::Bool(a != b)),
            (Value::Number(b), Value::Number(a)) => self.push(Value::Bool(a != b)),
            (Value::Null, Value::Null) => self.push(Value::Bool(false)),
            (_, _) => {
              return Err(
                InterpreterResult::runtime_error(
                  "Not equal operator can only be used with 2 of the same type",
                  self.line(),
                )
                .into(),
              )
            }
          }
        }
        OpCode::Greater => {
          let b = self.pop();
          let a = self.pop();
          match (b, a) {
            (Value::Bool(b), Value::Bool(a)) => self.push(Value::Bool(a > b)),
            (Value::String(b), Value::String(a)) => self.push(Value::Bool(a > b)),
            (Value::Number(b), Value::Number(a)) => self.push(Value::Bool(a > b)),
            (Value::Null, Value::Null) => self.push(Value::Bool(false)),
            (_, _) => {
              return Err(
                InterpreterResult::runtime_error(
                  "Greater than operator can only be used with 2 of the same type",
                  self.line(),
                )
                .into(),
              )
            }
          }
        }
        OpCode::GreaterOrEqual => {
          let b = self.pop();
          let a = self.pop();
          match (b, a) {
            (Value::Bool(b), Value::Bool(a)) => self.push(Value::Bool(a >= b)),
            (Value::String(b), Value::String(a)) => self.push(Value::Bool(a >= b)),
            (Value::Number(b), Value::Number(a)) => self.push(Value::Bool(a >= b)),
            (Value::Null, Value::Null) => self.push(Value::Bool(true)),
            (_, _) => {
              return Err(
                InterpreterResult::runtime_error(
                  "Greater than or equal operator can only be used with 2 of the same type",
                  self.line(),
                )
                .into(),
              )
            }
          }
        }
        OpCode::Less => {
          let b = self.pop();
          let a = self.pop();
          match (b, a) {
            (Value::Bool(b), Value::Bool(a)) => self.push(Value::Bool(a < b)),
            (Value::String(b), Value::String(a)) => self.push(Value::Bool(a < b)),
            (Value::Number(b), Value::Number(a)) => self.push(Value::Bool(a < b)),
            (Value::Null, Value::Null) => self.push(Value::Bool(false)),
            (_, _) => {
              return Err(
                InterpreterResult::runtime_error(
                  "Less than operator can only be used with 2 of the same type",
                  self.line(),
                )
                .into(),
              )
            }
          }
        }
        OpCode::LessOrEqual => {
          let b = self.pop();
          let a = self.pop();
          match (b, a) {
            (Value::Bool(b), Value::Bool(a)) => self.push(Value::Bool(a <= b)),
            (Value::String(b), Value::String(a)) => self.push(Value::Bool(a <= b)),
            (Value::Number(b), Value::Number(a)) => self.push(Value::Bool(a <= b)),
            (Value::Null, Value::Null) => self.push(Value::Bool(true)),
            (_, _) => {
              return Err(
                InterpreterResult::runtime_error(
                  "Less than or equal operator can only be used with 2 of the same type",
                  self.line(),
                )
                .into(),
              )
            }
          }
        }
        OpCode::Print => {
          println!("{}", self.pop());
        }
        OpCode::Pop => {
          self.pop();
        }
        OpCode::DefineGlobal => {
          let name = self.read_constant().as_string().ok_or_else(|| {
            InterpreterResult::runtime_error(
              "The identifier being used was not a string and is an internal runtime error",
              self.line(),
            )
          })?;
          let value = self.pop();
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
          let value = self.peek();
          if let None = self.globals.insert(name.clone(), value) {
            return Err(
              InterpreterResult::runtime_error(
                format!("Undefined variable '{}'", name),
                self.line(),
              )
              .into(),
            );
          }
        }
        OpCode::GetLocal => {
          let slot = self.read_byte() as usize;
          self.push(self.stack[slot + self.slots()].clone());
        }
        OpCode::SetLocal => {
          let slot = self.read_byte() as usize + self.slots();
          self.stack[slot] = self.peek();
        }
        OpCode::JumpIfFalse => {
          let offset = self.read_u16();
          if self.peek() == Value::Bool(false) {
            *self.ip() += offset as usize;
          }
        }
        OpCode::Jump => {
          *self.ip() += self.read_u16() as usize;
        }
        OpCode::Loop => {
          *self.ip() -= self.read_u16() as usize;
        }
        OpCode::Call => {
          let arg_count = self.read_byte();
          let callee = self.peek_n(arg_count as usize);
          self.call_value(callee, arg_count)?;
        }
      }
    }
  }
  fn read_byte(&mut self) -> u8 {
    let value = self.chunk().code[self.ip_()];
    *self.ip() += 1;
    value
  }
  fn read_u16(&mut self) -> u16 {
    *self.ip() += 2;
    let high = self.chunk().code[self.ip_() - 2] as u16;
    let low = self.chunk().code[self.ip_() - 1] as u16;
    (high << 8) | low
  }
  fn call_value(&mut self, callee: Value, mut arg_count: u8) -> Result<(), CedarError> {
    match callee {
      Value::Function(func) => self.call(func, arg_count),
      Value::NativeFn(func) => {
        let mut args = Vec::with_capacity(arg_count as usize);
        while arg_count != 0 {
          args.push(self.pop());
          arg_count -= 1;
        }
        let res = func.call(args).ok_or_else(|| {
          InterpreterResult::runtime_error("Native function call failed", self.line())
        })?;
        self.push(res);
        Ok(())
      }
      _ => Err(
        InterpreterResult::runtime_error("Can only call functions and classes", self.line()).into(),
      ),
    }
  }
  fn call(&mut self, function: Function, arg_count: u8) -> Result<(), CedarError> {
    if arg_count as usize > function.arity {
      return Err(
        InterpreterResult::runtime_error(
          format!(
            "Expected {} arguments but got {}",
            arg_count, function.arity
          ),
          self.line(),
        )
        .into(),
      );
    }
    self.frame_count += 1;
    Ok(self.frames.push(CallFrame {
      ip: 0,
      slots: if self.stack.len() == 0 {
        0
      } else {
        self.stack.len() - arg_count as usize
      },
      function,
    }))
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
    &self.frames[self.frame_count - 1].function.chunk
  }
  fn read_instruction(&self) -> OpCode {
    self.chunk().code[self.ip_()].into()
  }
  fn read_constant(&mut self) -> Value {
    let value = self.chunk().constants[self.chunk().code[self.ip_()] as usize].clone();
    *self.ip() += 1;
    value
  }
  fn push(&mut self, value: Value) {
    self.stack.push(value);
  }
  fn pop(&mut self) -> Value {
    let value = self.stack.pop().expect("Popped empty stack value");

    // This could be so so much better but I'm not really stuck
    // on a gc design or inneficiencies yet. Maybe I could use
    // Cow here idk
    if let Value::Heap(h) = value {
      self.heap[h].0.clone()
    } else {
      value
    }
  }
  fn peek(&mut self) -> Value {
    self
      .stack
      .iter()
      .rev()
      .next()
      .map(|v| v.clone())
      .expect("No value to peek on stack")
  }
  fn peek_n(&mut self, n: usize) -> Value {
    self
      .stack
      .iter()
      .rev()
      .nth(n)
      .map(|v| v.clone())
      .expect("No value to peek on stack")
  }
  fn line(&self) -> usize {
    self.chunk().lines[self.ip_() - 1]
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

#[derive(Debug)]
struct CallFrame {
  function: Function,
  ip: usize,
  // first index in stack it can point too.
  slots: usize,
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
