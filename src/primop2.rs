#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrimOp2 {
  AddI64,
}

impl PrimOp2 {
  pub fn as_str(self) -> &'static str {
    match self {
      Self::AddI64 => "add.i64",
    }
  }
}

impl std::fmt::Display for PrimOp2 {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.as_str())
  }
}
