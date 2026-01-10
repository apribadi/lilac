#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum PrimType {
  Bool,
  I64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum PrimOp1 {
  DecI64,
  IncI64,
  NegI64,
  NotBool,
}

static OP1_TABLE: [(&'static str, PrimType, PrimType); 4] = [
  ("dec.i64", PrimType::I64, PrimType::I64),
  ("inc.i64", PrimType::I64, PrimType::I64),
  ("neg.i64", PrimType::I64, PrimType::I64),
  ("not.bool", PrimType::Bool, PrimType::Bool),
];

impl PrimOp1 {
  pub fn as_str(&self) -> &'static str {
    return OP1_TABLE[*self as usize].0;
  }

  pub fn arg_type(&self) -> PrimType {
    return OP1_TABLE[*self as usize].1;
  }

  pub fn out_type(&self) -> PrimType {
    return OP1_TABLE[*self as usize].2;
  }
}

impl std::fmt::Display for PrimOp1 {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.as_str())
  }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum PrimOp2 {
  AddI64,
  BitAndI64,
  BitOrI64,
  BitXorI64,
  CmpEqI64,
  CmpGeI64,
  CmpGtI64,
  CmpLeI64,
  CmpLtI64,
  CmpNeI64,
  DivI64,
  MulI64,
  RemI64,
  ShlI64,
  ShrI64,
  SubI64,
}

static OP2_TABLE: [(&'static str, (PrimType, PrimType), PrimType); 16] = [
  ("add.i64", (PrimType::I64, PrimType::I64), PrimType::I64),
  ("bitand.i64", (PrimType::I64, PrimType::I64), PrimType::I64),
  ("bitor.i64", (PrimType::I64, PrimType::I64), PrimType::I64),
  ("bitxor.i64", (PrimType::I64, PrimType::I64), PrimType::I64),
  ("cmpeq.i64", (PrimType::I64, PrimType::I64), PrimType::Bool),
  ("cmpge.i64", (PrimType::I64, PrimType::I64), PrimType::Bool),
  ("cmpgt.i64", (PrimType::I64, PrimType::I64), PrimType::Bool),
  ("cmple.i64", (PrimType::I64, PrimType::I64), PrimType::Bool),
  ("cmplt.i64", (PrimType::I64, PrimType::I64), PrimType::Bool),
  ("cmpne.i64", (PrimType::I64, PrimType::I64), PrimType::Bool),
  ("div.i64", (PrimType::I64, PrimType::I64), PrimType::Bool),
  ("mul.i64", (PrimType::I64, PrimType::I64), PrimType::Bool),
  ("rem.i64", (PrimType::I64, PrimType::I64), PrimType::I64),
  ("shl.i64", (PrimType::I64, PrimType::I64), PrimType::I64),
  ("shr.i64", (PrimType::I64, PrimType::I64), PrimType::I64),
  ("sub.i64", (PrimType::I64, PrimType::I64), PrimType::I64),
];

impl PrimOp2 {
  pub fn as_str(&self) -> &'static str {
    return OP2_TABLE[*self as usize].0;
  }

  pub fn arg_type(&self) -> (PrimType, PrimType) {
    return OP2_TABLE[*self as usize].1;
  }

  pub fn out_type(&self) -> PrimType {
    return OP2_TABLE[*self as usize].2;
  }
}

impl std::fmt::Display for PrimOp2 {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.as_str())
  }
}
