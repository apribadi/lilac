//!
//!
//! linearized code -> typed code

use crate::ir1::Inst;
use crate::buf::Buf;
use crate::union_find::UnionFind;

type TypeVar = u32;

pub enum InstType {
  Local(TypeVar),
  Nil,
  Value(TypeVar),
}

struct TypeMap {
  inst: Buf<InstType>,
}

struct TypeSolver {
  vars: UnionFind<ValType>,
}

#[derive(Clone, Copy, Debug)]
pub enum ValType {
  Abstract,
  Bool,
  I64,
  TypeError,
}

struct Env {
  map: TypeMap,
  solver: TypeSolver,
}

impl TypeMap {
  fn new() -> Self {
    return Self { inst: Buf::new() };
  }

  fn put(&mut self, x: InstType) {
    self.inst.put(x);
  }

  fn local(&self, i: u32) -> TypeVar {
    let InstType::Local(x) = self.inst[i] else { unreachable!() };
    return x;
  }

  fn value(&self, i: u32) -> TypeVar {
    let InstType::Value(x) = self.inst[i] else { unreachable!() };
    return x;
  }
}

impl TypeSolver {
  fn new() -> Self {
    return Self { vars: UnionFind::new() };
  }

  fn put(&mut self) -> TypeVar {
    return self.vars.put(ValType::Abstract);
  }
}

impl Env {
  fn new() -> Self {
    return Self {
      map: TypeMap::new(),
      solver: TypeSolver::new(),
    }
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
  let x = &mut env.solver.vars[env.map.value(x)];
  *x = unify(*x, y);
}

fn flow(x: u32, y: u32, vars: &mut UnionFind<ValType>) {
  if let (x, Some(y)) = vars.union(x, y) {
    *x = unify(*x, y);
  }
}

pub fn typecheck(code: &[Inst]) -> (Buf<InstType>, UnionFind<ValType>) {
  let mut env = Env::new();

  for &inst in code.iter() {
    match inst {
      | Inst::Pop
      | Inst::Const(..)
      | Inst::ConstBool(..)
      | Inst::ConstInt(..)
      | Inst::Field(..)
      | Inst::Index(..)
      | Inst::Local(..)
      | Inst::Op1(..)
      | Inst::Op2(..) => {
        env.map.put(InstType::Value(env.solver.put()));
      }
      | Inst::DefLocal(..) => {
        env.map.put(InstType::Local(env.solver.put()));
      }
      _ =>
        env.map.put(InstType::Nil),
    }
  }

  for (i, &inst) in code.iter().enumerate() {
    let i = i as u32;
    match inst {
      Inst::Cond(x) =>
        constrain(&mut env, x, ValType::Bool),
      Inst::ConstBool(_) =>
        constrain(&mut env, i, ValType::Bool),
      Inst::ConstInt(_) =>
        constrain(&mut env, i, ValType::I64),
      Inst::DefLocal(x) => {
        let x = env.map.value(x);
        let y = env.map.local(i);
        flow(x, y, &mut env.solver.vars);
      }
      Inst::Local(x) => {
        let x = env.map.value(x);
        let y = env.map.local(i);
        flow(x, y, &mut env.solver.vars);
      }
      _ => {
      }
    }
  }


  return (env.map.inst, env.solver.vars);
}
