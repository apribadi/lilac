// untyped intermediate representation - bytecode
//
//

use crate::op1::Op1;
use crate::op2::Op2;

pub struct Symbol(pub u64);

type Label = u32;

type Value = u32;

pub enum Inst {
  Label,
  Pop,
  Put(Value),
  Jump(Label),
  Cond(Value, /* false */ Label, /* true */ Label),
  Ret,
  CallK1(Value, Label),
  CallTail(Value),
  Op1(Op1, Value),
  Op2(Op2, Value, Value),
  Integer(i64),
}

impl std::fmt::Display for Inst {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Label => write!(f, "LABEL"),
      Self::Pop => write!(f, "= POP"),
      Self::Put(x) => write!(f, "PUT %{}", x),
      Self::Ret => write!(f, "RET"),
      Self::Op2(op, x, y) => write!(f, "= %{} {} %{}", x, op, y),
      Self::Integer(n) => write!(f, "= INTEGER {}", n),
      _ => unimplemented!()
    }
  }
}
