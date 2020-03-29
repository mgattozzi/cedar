#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Value {
  Number(f64),
  Bool(bool),
  Null,
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
}
