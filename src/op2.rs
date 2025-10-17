#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Op2 {
  Add,
  BitAnd,
  BitOr,
  BitXor,
  CmpEq,
  CmpGe,
  CmpGt,
  CmpLe,
  CmpLt,
  CmpNe,
  Div,
  Mul,
  Rem,
  Shl,
  Shr,
  Sub,
}

impl Op2 {
  pub fn as_str(self) -> &'static str {
    match self {
      Self::Add => "+",
      Self::BitAnd => "&",
      Self::BitOr => "|",
      Self::BitXor => "^",
      Self::CmpEq => "==",
      Self::CmpGe => ">=",
      Self::CmpGt => ">",
      Self::CmpLe => "<=",
      Self::CmpLt => "<",
      Self::CmpNe => "!=",
      Self::Div => "/",
      Self::Mul => "*",
      Self::Rem => "%",
      Self::Shl => "<<",
      Self::Shr => ">>",
      Self::Sub => "-",
    }
  }
}

impl std::fmt::Display for Op2 {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.as_str())
  }
}
