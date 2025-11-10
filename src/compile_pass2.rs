//!
//!
//! linearized code -> typed code

use crate::ir1;
use crate::buf::Buf;

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

enum Typing {
  Label,
  Nil,
  Value(u32),
}

enum ValueType {
  Bool,
  I64,
}

struct Env {
  typing: Buf<Typing>,
  value_type: Buf<ValueType>,
}


impl Env {
  fn new() -> Self {
    return Self {
      typing: Buf::new(),
      value_type: Buf::new(),
    };
  }
}
