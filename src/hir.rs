// high-level intermediate representation
//
// - bytecode
// - from a source file, without context
// - not type checked

use crate::arr::Arr;
use crate::symbol::Symbol;
use crate::operator::Op1;
use crate::operator::Op2;

type Arity = u32;
type Index = u32;
type Label = u32;
type Local = u32;
type Value = u32;

pub struct Module {
  pub code: Arr<Inst>,
  pub items: Arr<Item>,
}

#[derive(Debug)]
pub enum Item {
  Fun { name: Symbol, pos: u32, len: u32 }
}

// TODO: add type ascription

#[derive(Clone, Copy)]
pub enum Inst {
  GotoStaticError,
  Label(Arity),
  Get(Index),
  Put(Index, Value),
  Goto(Label),
  Cond(Value),
  Ret,
  Call(Value),
  TailCall(Value),
  Const(Symbol),
  ConstBool(bool),
  ConstInt(i64),
  Field(Value, Symbol),
  Index(Value, Value),
  Op1(Op1, Value),
  Op2(Op2, Value, Value),
  Local(Value),
  GetLocal(Local),
  SetField(Value, Symbol, Value),
  SetIndex(Value, Value, Value),
  SetLocal(Local, Value),
}

#[derive(Debug)]
pub enum ValType {
  Abstract,
  Array(Box<ValType>),
  Bool,
  F64,
  Fun(Arr<ValType>, Option<Arr<ValType>>),
  I64,
  TypeError, // ???
}

impl std::fmt::Display for Inst {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Label(n) => write!(f, "LABEL {}", n),
      Self::Get(i) => write!(f, "= GET {}", i),
      Self::Put(i, x) => write!(f, "PUT {} %{}", i, x),
      Self::Goto(x) => write!(f, "==> GOTO %{}", x),
      Self::GotoStaticError => write!(f, "==> GOTO-STATIC-ERROR"),
      Self::Cond(x) => write!(f, "COND %{}", x),
      Self::Ret => write!(f, "RET"),
      Self::Call(x) => write!(f, "CALL %{}", x),
      Self::TailCall(x) => write!(f, "TAIL-CALL %{}", x),
      Self::Const(s) => write!(f, "= CONST {}", s),
      Self::ConstBool(p) => write!(f, "= {}", p),
      Self::ConstInt(n) => write!(f, "= {}", n),
      Self::Field(x, s) => write!(f, "= %{} [ .{} ]", x, s),
      Self::Index(x, y) => write!(f, "= %{} [ %{} ]", x, y),
      Self::Op1(op, x) => write!(f, "= {} %{}", op, x),
      Self::Op2(op, x, y) => write!(f, "= %{} {} %{}", x, op, y),
      Self::Local(x) => write!(f, "= LOCAL %{}", x),
      Self::GetLocal(v) => write!(f, "= [ %{} ]", v),
      Self::SetField(x, s, y) => write!(f, "%{} [ .{} ] <- %{}", x, s, y),
      Self::SetIndex(x, y, z) => write!(f, "%{} [ %{} ] <- %{}", x, y, z),
      Self::SetLocal(v, x) => write!(f, "[ %{} ] <- %{}", v, x),
    }
  }
}

