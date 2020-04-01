use std::{borrow::Cow, fmt};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
  Number(f64),
  Bool(bool),
  Null,
  String(Cow<'static, str>),
  Heap(usize),
}

impl Value {
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
}

impl fmt::Display for Value {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Value::Number(n) => write!(f, "{}", n),
      Value::Bool(b) => write!(f, "{}", b),
      Value::Null => write!(f, "null"),
      Value::String(s) => write!(f, "{}", s),
      Value::Heap(h) => write!(f, "heap {}", h),
    }
  }
}
