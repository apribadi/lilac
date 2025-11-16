//!
//!
//! linearized code -> typed code

use crate::ir1;
use crate::buf::Buf;
use crate::union_find::UnionFind;

#[derive(Debug)]
pub enum Typing {
  Nil,
  Val(u32),
  Var(u32),
}

#[derive(Clone, Copy, Debug)]
pub enum ValType {
  Abstract,
  Bool,
  I64,
  TypeError,
}

struct Env {
  typing: Buf<Typing>,
  valtypes: UnionFind<ValType>,
}

impl Env {
  fn valtype(&self, x: u32) -> u32 {
    let Typing::Val(x) = self.typing[x] else { panic!() };
    return x;
  }

  fn vartype(&self, x: u32) -> u32 {
    let Typing::Var(x) = self.typing[x] else { panic!() };
    return x;
  }
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

fn constrain(env: &mut Env, x: u32, y: ValType) {
  let Typing::Val(x) = env.typing[x] else { unreachable!() };
  let x = &mut env.valtypes[x];
  *x = unify(*x, y);
}

fn flow(x: u32, y: u32, valtypes: &mut UnionFind<ValType>) {
  if let (x, Some(y)) = valtypes.union(x, y) {
    *x = unify(*x, y);
  }
}

pub fn typecheck(code: &[ir1::Inst]) -> (Buf<Typing>, UnionFind<ValType>) {
  let mut env =
    Env {
      typing: Buf::new(),
      valtypes: UnionFind::new(),
    };

  for &inst in code.iter() {
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
        env.typing.put(Typing::Val(env.valtypes.emit(ValType::Abstract)));
      }
      | ir1::Inst::DefLocal(..) => {
        env.typing.put(Typing::Var(env.valtypes.emit(ValType::Abstract)));
      }
      _ =>
        env.typing.put(Typing::Nil),
    }
  }

  for (i, &inst) in code.iter().enumerate() {
    let i = i as u32;
    match inst {
      ir1::Inst::Cond(x) =>
        constrain(&mut env, x, ValType::Bool),
      ir1::Inst::ConstBool(_) =>
        constrain(&mut env, i, ValType::Bool),
      ir1::Inst::ConstInt(_) =>
        constrain(&mut env, i, ValType::I64),
      ir1::Inst::DefLocal(x) => {
        let x = env.valtype(x);
        let y = env.vartype(i);
        flow(x, y, &mut env.valtypes);
      }
      ir1::Inst::Local(x) => {
        let x = env.vartype(x);
        let y = env.valtype(i);
        flow(x, y, &mut env.valtypes);
      }
      _ => {
      }
    }
  }


  return (env.typing, env.valtypes);
}
