#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Value {
  Number(f64),
}

impl Value {
  #[allow(irrefutable_let_patterns)]
  pub fn as_num(self) -> Option<f64> {
    if let Value::Number(n) = self {
      Some(n)
    } else {
      None
    }
  }
}
