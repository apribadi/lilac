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
  typevars: UnionFind<ValType>,
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
    return Self { typevars: UnionFind::new() };
  }

  fn put(&mut self) -> TypeVar {
    return self.typevars.put(ValType::Abstract);
  }

  fn constrain(&mut self, x: TypeVar, y: ValType) {
    let x = &mut self.typevars[x];
    *x = unify(*x, y);
  }

  fn flow(&mut self, x: u32, y: u32) {
    if let (y, Some(x)) = self.typevars.union(y, x) {
      *y = unify(*y, x);
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

impl Env {
  fn new() -> Self {
    return Self {
      map: TypeMap::new(),
      solver: TypeSolver::new(),
    }
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
        env.solver.constrain(env.map.value(x), ValType::Bool),
      Inst::ConstBool(_) =>
        env.solver.constrain(env.map.value(i), ValType::Bool),
      Inst::ConstInt(_) =>
        env.solver.constrain(env.map.value(i), ValType::I64),
      Inst::DefLocal(x) =>
        env.solver.flow(env.map.value(x), env.map.local(i)),
      Inst::Local(x) =>
        env.solver.flow(env.map.local(x), env.map.value(i)),
      Inst::SetLocal(x, y) =>
        env.solver.flow(env.map.value(y), env.map.local(x)),
      _ => {
      }
    }
  }


  return (env.map.inst, env.solver.typevars);
}
