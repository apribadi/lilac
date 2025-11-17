//!
//!
//! linearized code -> typed code

use crate::ir1;
use crate::ir1::Op1;
use crate::ir1::Op2;
use crate::ir1::Inst;
use crate::buf::Buf;
use crate::union_find::UnionFind;

#[derive(Clone, Copy, Debug)]
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
  todo: Buf<(TypeVar, TypeVar)>,
}

#[derive(Clone, Copy, Debug)]
pub enum ValType {
  Abstract,
  Array(TypeVar),
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

fn unify(x: &mut ValType, y: ValType, todo: &mut Buf<(TypeVar, TypeVar)>) {
  *x =
    match (*x, y) {
      (ValType::Abstract, y) => y,
      (x, ValType::Abstract) => x,
      (ValType::Bool, ValType::Bool) => ValType::Bool,
      (ValType::I64, ValType::I64) => ValType::I64,
      (ValType::Array(x), ValType::Array(y)) => {
        todo.put((x, y));
        ValType::Array(x)
      }
      (_, _) => ValType::TypeError,
    };
}

impl TypeSolver {
  fn new() -> Self {
    return Self {
      valtypes: UnionFind::new(),
      todo: Buf::new(),
    };
  }

  fn fresh(&mut self) -> TypeVar {
    return TypeVar(self.valtypes.put(ValType::Abstract));
  }

  fn flow_tv(&mut self, x: ValType, y: TypeVar) {
    unify(&mut self.valtypes[y.0], x, &mut self.todo);
  }

  fn flow_vt(&mut self, x: TypeVar, y: ValType) {
    unify(&mut self.valtypes[x.0], y, &mut self.todo);
  }

  fn flow_vv(&mut self, x: TypeVar, y: TypeVar) {
    if let (y, Some(x)) = self.valtypes.union(y.0, x.0) {
      unify(y, x, &mut self.todo);
    }
  }

  fn propagate(&mut self) {
    while ! self.todo.is_empty() {
      let (x, y) = self.todo.pop();
      if let (x, Some(y)) = self.valtypes.union(x.0, y.0) {
        unify(x, y, &mut self.todo);
      }
    }
  }

  pub fn resolve(&self, x: TypeVar) -> ir1::ValType {
    // recursive types?

    match self.valtypes[x.0] {
      ValType::Abstract => ir1::ValType::Abstract,
      ValType::Array(a) => ir1::ValType::Array(Box::new(self.resolve(a))),
      ValType::Bool => ir1::ValType::Bool,
      ValType::I64 => ir1::ValType::I64,
      ValType::TypeError => unimplemented!(),
    }
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
        env.map.put(InstType::Value(env.solver.fresh()));
      }
      | Inst::DefLocal(..) => {
        env.map.put(InstType::Local(env.solver.fresh()));
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
      Inst::Index(x, y) => {
        let a = env.solver.fresh();
        env.solver.flow_vt(env.map.value(x), ValType::Array(a));
        env.solver.flow_vt(env.map.value(y), ValType::I64);
        env.solver.flow_vv(a, env.map.value(i));
      }
      Inst::SetIndex(x, y, z) => {
        let a = env.solver.fresh();
        env.solver.flow_vt(env.map.value(x), ValType::Array(a));
        env.solver.flow_vt(env.map.value(y), ValType::I64);
        env.solver.flow_vv(env.map.value(z), a);
      }
      Inst::DefLocal(x) =>
        env.solver.flow_vv(env.map.value(x), env.map.local(i)),
      Inst::Local(x) =>
        env.solver.flow_vv(env.map.local(x), env.map.value(i)),
      Inst::SetLocal(x, y) =>
        env.solver.flow_vv(env.map.value(y), env.map.local(x)),
      Inst::Op1(f, x) => {
        let (a, b) =
          match f {
            | Op1::Neg => (ValType::I64, ValType::I64),
            | Op1::Not => (ValType::Bool, ValType::Bool),
          };
        env.solver.flow_vt(env.map.value(x), a);
        env.solver.flow_tv(b, env.map.value(i));
      }
      Inst::Op2(f, x, y) => {
        let (a, b, c) =
          match f {
            | Op2::Add
            | Op2::Sub
            | Op2::BitAnd
            | Op2::BitOr
            | Op2::BitXor
            | Op2::Div
            | Op2::Mul
            | Op2::Rem
              => (ValType::I64, ValType::I64, ValType::I64),
            | Op2::Shl
            | Op2::Shr
              => (ValType::I64, ValType::I64, ValType::I64),
            | Op2::CmpEq
            | Op2::CmpNe
            | Op2::CmpGe
            | Op2::CmpGt
            | Op2::CmpLe
            | Op2::CmpLt
              => (ValType::I64, ValType::I64, ValType::Bool),
          };
        env.solver.flow_vt(env.map.value(x), a);
        env.solver.flow_vt(env.map.value(y), b);
        env.solver.flow_tv(c, env.map.value(i));
      }
      _ => {
      }
    }
  }

  env.solver.propagate();

  return (env.map, env.solver);
}
