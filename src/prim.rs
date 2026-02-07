#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum PrimType {
  Bool,
  I64,
}

use PrimType::*;

impl std::fmt::Display for PrimType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s =
      match self {
        &Self::Bool => "bool",
        &Self::I64 => "i64",
      };
    f.write_str(s)
  }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum PrimOp1 {
  DecI64,
  IncI64,
  NegI64,
  NotBool,
}

static OP1_TABLE: [(&'static str, PrimType, PrimType); 4] = [
  ("dec.i64", I64, I64),
  ("inc.i64", I64, I64),
  ("neg.i64", I64, I64),
  ("not.bool", Bool, Bool),
];

impl PrimOp1 {
  pub fn as_str(&self) -> &'static str {
    OP1_TABLE[*self as usize].0
  }

  pub fn arg_type(&self) -> PrimType {
    OP1_TABLE[*self as usize].1
  }

  pub fn out_type(&self) -> PrimType {
    OP1_TABLE[*self as usize].2
  }
}

impl std::fmt::Display for PrimOp1 {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.as_str())
  }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
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
  ("add.i64", (I64, I64), I64),
  ("bitand.i64", (I64, I64), I64),
  ("bitor.i64", (I64, I64), I64),
  ("bitxor.i64", (I64, I64), I64),
  ("cmpeq.i64", (I64, I64), Bool),
  ("cmpge.i64", (I64, I64), Bool),
  ("cmpgt.i64", (I64, I64), Bool),
  ("cmple.i64", (I64, I64), Bool),
  ("cmplt.i64", (I64, I64), Bool),
  ("cmpne.i64", (I64, I64), Bool),
  ("div.i64", (I64, I64), I64),
  ("mul.i64", (I64, I64), I64),
  ("rem.i64", (I64, I64), I64),
  ("shl.i64", (I64, I64), I64),
  ("shr.i64", (I64, I64), I64),
  ("sub.i64", (I64, I64), I64),
];

impl PrimOp2 {
  pub fn as_str(&self) -> &'static str {
    OP2_TABLE[*self as usize].0
  }

  pub fn arg_type(&self) -> (PrimType, PrimType) {
    OP2_TABLE[*self as usize].1
  }

  pub fn out_type(&self) -> PrimType {
    OP2_TABLE[*self as usize].2
  }
}

impl std::fmt::Display for PrimOp2 {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.as_str())
  }
}
