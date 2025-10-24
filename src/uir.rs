// untyped intermediate representation - bytecode

use crate::symbol::Symbol;

pub enum Op1 {
  Decr,
  Incr,
  Neg,
  Not,
}

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

type Label = u32;

type Value = u32;

// TODO: add type ascription

pub enum Inst {
  GotoStaticError,
  Label(u32),
  Pop,
  Put(Value),
  Goto(Label),
  Cond(Value),
  Ret,
  Call(Value),
  TailCall(Value),
  ConstBool(bool),
  ConstInt(i64),
  DefLocal(Value),
  Field(Value, Symbol),
  Global(Symbol),
  Index(Value, Value),
  Local(Value),
  Op1(Op1, Value),
  Op2(Op2, Value, Value),
  SetField(Value, Symbol, Value),
  SetIndex(Value, Value, Value),
  SetLocal(Value, Value),
}

impl std::fmt::Display for Inst {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Label(n) => write!(f, "LABEL {}", n),
      Self::Pop => write!(f, "= POP"),
      Self::Put(x) => write!(f, "PUT %{}", x),
      Self::Goto(x) => write!(f, "==> GOTO %{}", x),
      Self::GotoStaticError => write!(f, "==> GOTO-STATIC-ERROR"),
      Self::Cond(x) => write!(f, "COND %{}", x),
      Self::Ret => write!(f, "RET"),
      Self::Call(x) => write!(f, "CALL %{}", x),
      Self::TailCall(x) => write!(f, "TAIL-CALL %{}", x),
      Self::ConstBool(p) => write!(f, "= {}", p),
      Self::ConstInt(n) => write!(f, "= {}", n),
      Self::DefLocal(x) => write!(f, "= DEF-LOCAL %{}", x),
      Self::Field(x, s) => write!(f, "= %{} [ .{} ]", x, s),
      Self::Global(s) => write!(f, "= {}", s),
      Self::Index(x, y) => write!(f, "= %{} [ %{} ]", x, y),
      Self::Local(v) => write!(f, "= [ %{} ]", v),
      Self::Op1(op, x) => write!(f, "= {} %{}", op, x),
      Self::Op2(op, x, y) => write!(f, "= %{} {} %{}", x, op, y),
      Self::SetField(x, s, y) => write!(f, "%{} [ .{} ] <- %{}", x, s, y),
      Self::SetIndex(x, i, y) => write!(f, "%{} [ %{} ] <- %{}", x, i, y),
      Self::SetLocal(v, x) => write!(f, "[ %{} ] <- %{}", v, x),
    }
  }
}

impl Op1 {
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::Decr => "--",
      Self::Incr => "++",
      Self::Neg => "-",
      Self::Not => "!",
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
