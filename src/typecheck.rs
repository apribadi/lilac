//!
//!
//! linearized code -> typed code

use crate::ir1::Inst;
use crate::buf::Buf;
use crate::union_find::UnionFind;

#[derive(Clone, Copy)]
pub struct TypeVar(u32);

#[derive(Clone, Copy)]
pub enum InstType {
  Local(TypeVar),
  Nil,
  Value(TypeVar),
}

pub struct TypeMap {
  insts: Buf<InstType>,
}

pub struct TypeSolver {
  valtypes: UnionFind<ValType>,
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
    return Self { insts: Buf::new() };
  }

  fn put(&mut self, x: InstType) {
    self.insts.put(x);
  }

  fn local(&self, i: u32) -> TypeVar {
    let InstType::Local(x) = self.insts[i] else { unreachable!() };
    return x;
  }

  fn value(&self, i: u32) -> TypeVar {
    let InstType::Value(x) = self.insts[i] else { unreachable!() };
    return x;
  }

  pub fn insts(&self) -> impl Iterator<Item = InstType> {
    return self.insts.iter().map(|x| *x);
  }
}

impl TypeSolver {
  fn new() -> Self {
    return Self { valtypes: UnionFind::new() };
  }

  fn put(&mut self) -> TypeVar {
    return TypeVar(self.valtypes.put(ValType::Abstract));
  }

  fn flow_tv(&mut self, x: ValType, y: TypeVar) {
    let y = &mut self.valtypes[y.0];
    unify(y, x);
  }

  fn flow_vt(&mut self, x: TypeVar, y: ValType) {
    let x = &mut self.valtypes[x.0];
    unify(x, y);
  }

  fn flow_vv(&mut self, x: TypeVar, y: TypeVar) {
    if let (y, Some(x)) = self.valtypes.union(y.0, x.0) {
      unify(y, x);
    }
  }

  pub fn valtype(&self, x: TypeVar) -> ValType {
    return self.valtypes[x.0];
  }
}

fn unify(x: &mut ValType, y: ValType) {
  *x =
    match (*x, y) {
      (ValType::Abstract, _) => y,
      (_, ValType::Abstract) => *x,
      (ValType::Bool, ValType::Bool) => ValType::Bool,
      (ValType::I64, ValType::I64) => ValType::I64,
      (_, _) => ValType::TypeError,
    };
}

impl Env {
  fn new() -> Self {
    return Self {
      map: TypeMap::new(),
      solver: TypeSolver::new(),
    }
  }
}

pub fn typecheck(code: &[Inst]) -> (TypeMap, TypeSolver) {
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
        env.solver.flow_vt(env.map.value(x), ValType::Bool),
      Inst::ConstBool(_) =>
        env.solver.flow_tv(ValType::Bool, env.map.value(i)),
      Inst::ConstInt(_) =>
        env.solver.flow_tv(ValType::I64, env.map.value(i)),
      Inst::DefLocal(x) =>
        env.solver.flow_vv(env.map.value(x), env.map.local(i)),
      Inst::Local(x) =>
        env.solver.flow_vv(env.map.local(x), env.map.value(i)),
      Inst::SetLocal(x, y) =>
        env.solver.flow_vv(env.map.value(y), env.map.local(x)),
      _ => {
      }
    }
  }

  return (env.map, env.solver);
}
