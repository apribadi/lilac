//!
//!
//! linearized code -> typed code

use crate::ir1;
use crate::buf::Buf;
use crate::union_find::UnionFind;
use std::iter::zip;

#[derive(Debug)]
pub enum Typing {
  Nil,
  ValType(u32),
}

#[derive(Clone, Copy, Debug)]
pub enum ValType {
  Abstract,
  Bool,
  I64,
  TypeError,
}

fn unify(x: ValType, y: ValType) -> ValType {
  match (x, y) {
    (ValType::Abstract, _) => y,
    (_, ValType::Abstract) => x,
    (ValType::Bool, ValType::Bool) => ValType::Bool,
    (ValType::I64, ValType::I64) => ValType::I64,
    (_, _) => ValType::TypeError,
  }
}

pub fn typecheck(code: &[ir1::Inst]) -> (Buf<Typing>, UnionFind<ValType>) {
  let mut typing = Buf::new();
  let mut valtypes = UnionFind::new();

  for inst in code.iter() {
    match inst {
      | ir1::Inst::Pop
      | ir1::Inst::Const(..)
      | ir1::Inst::ConstBool(..)
      | ir1::Inst::ConstInt(..)
      | ir1::Inst::Field(..)
      | ir1::Inst::Index(..)
      | ir1::Inst::Local(..)
      | ir1::Inst::Op1(..)
      | ir1::Inst::Op2(..) => {
        typing.put(Typing::ValType(valtypes.emit(ValType::Abstract)));
      }
      _ =>
        typing.put(Typing::Nil),
    }
  }

  for (inst, typing) in zip(code.iter(), typing.iter()) {
    match (inst, typing) {
      (ir1::Inst::ConstInt(_), Typing::ValType(x)) => {
        let x = &mut valtypes[*x];
        *x = unify(*x, ValType::I64);
      }
      _ => {
      }
    }
  }


  return (typing, valtypes);
}
