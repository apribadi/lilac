// untyped bytecode

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
