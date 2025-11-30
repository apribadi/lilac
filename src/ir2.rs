use crate::symbol::Symbol;

type Arity = u32;
type Index = u32;
type Label = u32;
type Local = u32;
type Value = u32;

#[derive(Clone, Copy)]
pub enum Inst {
  GotoStaticError,
  Label(Arity),
  Get(Index), // type?
  Put(Index, Value),
  Goto(Label),
  Cond(Value),
  Ret,
  Call(Value),
  TailCall(Value),
  Const(Symbol), // type?
  Local(Value),
  ConstBool(bool),
  ConstI64(i64),
  Op1(Op1, Value),
  Op2(Op2, Value, Value),
  GetArray(Value, Value),
  SetArray(Value, Value, Value),
  GetLocal(Local),
  SetLocal(Local, Value),
}

#[derive(Clone, Copy)]
pub enum Op1 {
  NegI64,
  NotBool,
}

#[derive(Clone, Copy)]
pub enum Op2 {
  AddF32,
  AddF64,
  AddI32,
  AddI64,
}

impl Op1 {
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::NegI64 => "neg.i64",
      Self::NotBool => "not.bool",
    }
  }
}

impl std::fmt::Display for Op1 {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.as_str())
  }
}

impl Op2 {
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::AddF32 => "add.f32",
      Self::AddF64 => "add.f64",
      Self::AddI32 => "add.i32",
      Self::AddI64 => "add.i64",
    }
  }
}

impl std::fmt::Display for Op2 {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.as_str())
  }
}
