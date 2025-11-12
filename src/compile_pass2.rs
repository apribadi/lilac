//!
//!
//! linearized code -> typed code

use crate::ir1;
use crate::buf::Buf;

pub enum Typing {
  Label,
  Nil,
  Value(u32),
}

pub struct ValueType(u32);

pub enum ValueTypeNode {
  Abstract,
  Bool,
  I64,
  Indirect(u32),
}

struct Env {
  typing: Buf<Typing>,
  value_type: Buf<ValueTypeNode>,
}

impl Env {
  fn new() -> Self {
    return Self {
      typing: Buf::new(),
      value_type: Buf::new(),
    };
  }
}

pub fn compile(code: &[ir1::Inst]) {
  let mut env = Env::new();

  for &inst in code.iter() {
    match inst {
      ir1::Inst::Label(..) =>
        env.typing.put(Typing::Label),
      _ => {
      }
    }
  }
}
