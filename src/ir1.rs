// intermediate representation
//
// - bytecode
// - from a source file, without context
// - not type checked

use crate::symbol::Symbol;
use crate::op1::Op1;
use crate::op2::Op2;

type Label = u32;

type Value = u32;

type Local = u32;

// TODO: add type ascription

pub enum Inst {
  GotoStaticError,
  Entry(u32),
  Label(u32),
  Pop,
  Put(Value),
  Goto(Label),
  Cond(Value),
  Ret,
  Call(Value),
  TailCall(Value),
  Const(Symbol),
  ConstBool(bool),
  ConstInt(i64),
  DefLocal(Value),
  Field(Value, Symbol),
  Index(Value, Value),
  Local(Local),
  Op1(Op1, Value),
  Op2(Op2, Value, Value),
  SetField(Value, Symbol, Value),
  SetIndex(Value, Value, Value),
  SetLocal(Local, Value),
}

impl std::fmt::Display for Inst {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Entry(n) => write!(f, "ENTRY {}", n),
      Self::Label(n) => write!(f, "LABEL {}", n),
      Self::Pop => write!(f, "= POP"),
      Self::Put(x) => write!(f, "PUT %{}", x),
      Self::Goto(x) => write!(f, "==> GOTO %{}", x),
      Self::GotoStaticError => write!(f, "==> GOTO-STATIC-ERROR"),
      Self::Cond(x) => write!(f, "COND %{}", x),
      Self::Ret => write!(f, "RET"),
      Self::Call(x) => write!(f, "CALL %{}", x),
      Self::TailCall(x) => write!(f, "TAIL-CALL %{}", x),
      Self::Const(s) => write!(f, "= CONST {}", s),
      Self::ConstBool(p) => write!(f, "= {}", p),
      Self::ConstInt(n) => write!(f, "= {}", n),
      Self::DefLocal(x) => write!(f, "= DEF-LOCAL %{}", x),
      Self::Field(x, s) => write!(f, "= %{} [ .{} ]", x, s),
      Self::Index(x, y) => write!(f, "= %{} [ %{} ]", x, y),
      Self::Local(v) => write!(f, "= [ %{} ]", v),
      Self::Op1(op, x) => write!(f, "= {} %{}", op, x),
      Self::Op2(op, x, y) => write!(f, "= %{} {} %{}", x, op, y),
      Self::SetField(x, s, y) => write!(f, "%{} [ .{} ] <- %{}", x, s, y),
      Self::SetIndex(x, y, z) => write!(f, "%{} [ %{} ] <- %{}", x, y, z),
      Self::SetLocal(v, x) => write!(f, "[ %{} ] <- %{}", v, x),
    }
  }
}
