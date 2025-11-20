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
  args: Buf<TypeVar>,
  map: TypeMap,
  solver: TypeSolver,
  is_call: bool,
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

fn unify_impl(x: &mut ValType, y: ValType, todo: &mut Buf<(TypeVar, TypeVar)>) {
  use ValType::*;

  *x =
    match (*x, y) {
      (Abstract, t) | (t, Abstract) => t,
      (Bool, Bool) => Bool,
      (I64, I64) => I64,
      (Array(x), Array(y)) => {
        todo.put((x, y));
        Array(x)
      }
      (_, _) => TypeError,
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

  fn bound(&mut self, x: TypeVar, t: ValType) {
    unify_impl(&mut self.valtypes[x.0], t, &mut self.todo);
  }

  fn unify(&mut self, x: TypeVar, y: TypeVar) {
    if let (x, Some(y)) = self.valtypes.union(x.0, y.0) {
      unify_impl(x, y, &mut self.todo);
    }
  }

  fn propagate(&mut self) {
    while let Some((x, y)) = self.todo.pop_if_nonempty() {
      if let (x, Some(y)) = self.valtypes.union(x.0, y.0) {
        unify_impl(x, y, &mut self.todo);
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
      ValType::TypeError => ir1::ValType::TypeError, // ???
    }
  }
}

impl Env {
  fn new() -> Self {
    return Self {
      args: Buf::new(),
      map: TypeMap::new(),
      solver: TypeSolver::new(),
      is_call: false,
    }
  }
}

pub fn typecheck(code: &[Inst]) -> (TypeMap, TypeSolver) {
  let mut env = Env::new();

  // assign type variables for all relevant program points

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
      | Inst::Op2(..) =>
        env.map.put(InstType::Value(env.solver.fresh())),
      | Inst::DefLocal(..) =>
        env.map.put(InstType::Local(env.solver.fresh())),
      | Inst::GotoStaticError
      | Inst::Entry(..)
      | Inst::Label(..)
      | Inst::Put(..)
      | Inst::Goto(..)
      | Inst::Cond(..)
      | Inst::Ret
      | Inst::Call(..)
      | Inst::TailCall(..)
      | Inst::SetField(..)
      | Inst::SetIndex(..)
      | Inst::SetLocal(..) =>
        env.map.put(InstType::Nil),
    }
  }

  // apply initial type constraints

  for (i, &inst) in code.iter().enumerate() {
    let i = i as u32;
    match inst {
      Inst::Cond(x) =>
        env.solver.bound(env.map.value(x), ValType::Bool),
      Inst::ConstBool(_) =>
        env.solver.bound(env.map.value(i), ValType::Bool),
      Inst::ConstInt(_) =>
        env.solver.bound(env.map.value(i), ValType::I64),
      Inst::Index(x, y) => {
        let a = env.solver.fresh();
        env.solver.bound(env.map.value(x), ValType::Array(a));
        env.solver.bound(env.map.value(y), ValType::I64);
        env.solver.unify(a, env.map.value(i));
      }
      Inst::SetIndex(x, y, z) => {
        let a = env.solver.fresh();
        env.solver.bound(env.map.value(x), ValType::Array(a));
        env.solver.bound(env.map.value(y), ValType::I64);
        env.solver.unify(env.map.value(z), a);
      }
      Inst::DefLocal(x) =>
        env.solver.unify(env.map.value(x), env.map.local(i)),
      Inst::Local(x) =>
        env.solver.unify(env.map.local(x), env.map.value(i)),
      Inst::SetLocal(x, y) =>
        env.solver.unify(env.map.value(y), env.map.local(x)),
      Inst::Op1(f, x) => {
        let (a, b) =
          match f {
            | Op1::Neg => (ValType::I64, ValType::I64),
            | Op1::Not => (ValType::Bool, ValType::Bool),
          };
        env.solver.bound(env.map.value(x), a);
        env.solver.bound(env.map.value(i), b);
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
        env.solver.bound(env.map.value(x), a);
        env.solver.bound(env.map.value(y), b);
        env.solver.bound(env.map.value(i), c);
      }
      Inst::Entry(..) | Inst::Label(..) => {
        env.args.clear();
        env.is_call = false;
      }
      Inst::Put(x) =>
        env.args.put(env.map.value(x)),
      Inst::Goto(target) => {
        // TODO
        if ! env.is_call {
          let n = env.args.len();
          for i in 0 .. n {
            let Inst::Pop = code[(target + 1 + i) as usize] else { unreachable!() };
            env.solver.unify(env.args[i], env.map.value(target + 1 + i));
          }
        }
      }
      Inst::Call(..) => {
        env.args.clear();
        env.is_call = true; // TODO
      }
      | Inst::Pop
      | Inst::Const(..)
      | Inst::Field(..)
      | Inst::GotoStaticError
      | Inst::Ret
      | Inst::TailCall(..)
      | Inst::SetField(..) => {
      }
    }
  }

  // solve all type constraints

  env.solver.propagate();

  return (env.map, env.solver);
}
