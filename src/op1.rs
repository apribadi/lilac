#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Op1 {
  Neg,
  Not,
}

impl Op1 {
  pub fn as_str(self) -> &'static str {
    match self {
      Self::Neg => "-",
      Self::Not => "!",
    }
  }
}

impl std::fmt::Display for Op1 {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.as_str())
  }
}
