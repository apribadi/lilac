// untyped intermediate representation - bytecode
//
//

// TODO: consider operator application in tail position

use crate::op1::Op1;
use crate::op2::Op2;
use crate::symbol::Symbol;

type Label = u32;

type Value = u32;

pub enum Inst {
  Label,
  Pop,
  Put(Value),
  Jump(Label),
  Cond(Value, /* false */ Label, /* true */ Label),
  Ret,
  Call(Value, Label),
  TailCall(Value),
  ConstBool(bool),
  ConstInt(i64),
  Field(Value, Symbol),
  Global(Symbol),
  Index(Value, Value),
  Local(Value),
  Op1(Op1, Value),
  Op2(Op2, Value, Value),
  SetField(Value, Symbol, Value),
  SetIndex(Value, Value, Value),
  SetLocal(Value, Value),
  Var(Value), // InitLocal?
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
      Self::Call(x, a) => write!(f, "CALL %{} =>{}", x, a),
      Self::TailCall(x) => write!(f, "TAILCALL %{}", x),
      Self::ConstBool(p) => write!(f, "= {}", p),
      Self::ConstInt(n) => write!(f, "= {}", n),
      Self::Field(x, s) => write!(f, "= %{} .{}", x, s),
      Self::Global(s) => write!(f, "= {}", s),
      Self::Index(x, y) => write!(f, "= %{} [ %{} ]", x, y),
      Self::Local(v) => write!(f, "= [ %{} ]", v),
      Self::Op1(op, x) => write!(f, "= {} %{}", op, x),
      Self::Op2(op, x, y) => write!(f, "= %{} {} %{}", x, op, y),
      Self::SetField(x, s, y) => write!(f, "%{} .{} <- %{}", x, s, y),
      Self::SetIndex(x, i, y) => write!(f, "%{} [ %{} ] <- %{}", x, i, y),
      Self::SetLocal(v, x) => write!(f, "[ %{} ] <- %{}", v, x),
      Self::Var(x) => write!(f, "= VAR %{}", x),
      Self::Undefined => write!(f, "= UNDEFINED"),
    }
  }
}
