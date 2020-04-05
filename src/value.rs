use crate::{chunk::Chunk, native::NativeFuncHolder};
use std::{borrow::Cow, fmt};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
  Number(f64),
  Bool(bool),
  Byte(u8),
  Null,
  String(Cow<'static, str>),
  Heap(usize),
  Function(Function),
  NativeFn(NativeFuncHolder),
}

impl Value {
  pub fn into_byte(self) -> u8 {
    if let Value::Byte(b) = self {
      b
    } else {
      panic!()
    }
  }
  pub fn as_num(self) -> Option<f64> {
    if let Value::Number(n) = self {
      Some(n)
    } else {
      None
    }
  }
  pub fn as_bool(self) -> Option<bool> {
    if let Value::Bool(b) = self {
      Some(b)
    } else {
      None
    }
  }
  pub fn as_string(self) -> Option<Cow<'static, str>> {
    if let Value::String(s) = self {
      Some(s)
    } else {
      None
    }
  }
  pub fn as_function(self) -> Option<Function> {
    if let Value::Function(f) = self {
      Some(f)
    } else {
      None
    }
  }
}

impl fmt::Display for Value {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Value::Number(n) => write!(f, "{}", n),
      Value::Bool(b) => write!(f, "{}", b),
      Value::Byte(b) => write!(f, "{}", b),
      Value::Null => write!(f, "null"),
      Value::String(s) => write!(f, "{}", s),
      Value::Heap(h) => write!(f, "heap {}", h),
      Value::Function(func) => write!(f, "{}", func),
      Value::NativeFn(func) => write!(f, "{}", func),
    }
  }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Function {
  pub arity: usize,
  pub chunk: Chunk,
  pub name: Cow<'static, str>,
}

impl Function {
  pub fn new() -> Self {
    Self {
      name: "".into(),
      arity: 0,
      chunk: Chunk::new(),
    }
  }
}

impl fmt::Display for Function {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    if self.name.is_empty() {
      write!(f, "<script>")
    } else {
      write!(f, "<fn {}>", self.name)
    }
  }
}
