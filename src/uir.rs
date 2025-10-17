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
  Index(Value, Value),
  Op1(Op1, Value),
  Op2(Op2, Value, Value),
  ConstBool(bool),
  Integer(i64),
  Undefined,
}

impl std::fmt::Display for Inst {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Label => write!(f, "LABEL"),
      Self::Pop => write!(f, "= POP"),
      Self::Put(x) => write!(f, "PUT %{}", x),
      Self::Jump(x) => write!(f, "JUMP =>{}", x),
      Self::Cond(x, a, b) => write!(f, "COND %{} =>{} =>{}", x, a, b),
      Self::Ret => write!(f, "RET"),
      Self::Index(x, y) => write!(f, "= %{}[%{}]", x, y),
      Self::Op1(op, x) => write!(f, "= {} %{}", op, x),
      Self::Op2(op, x, y) => write!(f, "= %{} {} %{}", x, op, y),
      Self::ConstBool(p) => write!(f, "= {}", p),
      Self::Integer(n) => write!(f, "= #{}", n),
      _ => unimplemented!()
    }
  }
}
