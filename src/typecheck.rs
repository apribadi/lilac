//!
//!
//! linearized code -> typed code

use crate::buf::Buf;
use crate::ir1::Inst;
use crate::ir1::Item;
use crate::ir1::Module;
use crate::ir1::Op1;
use crate::ir1::Op2;
use crate::ir1;
use crate::union_find::UnionFind;
use std::iter::zip;
use std::mem::replace;

#[derive(Clone, Copy, Debug)]
pub struct TypeVar(u32);

#[derive(Clone)]
pub enum InstType {
  Entry(Box<[TypeVar]>, TypeVar),
  Label(Box<[TypeVar]>),
  Local(TypeVar),
  Nil,
  Value(TypeVar),
}

pub struct TypeMap {
  insts: Buf<InstType>,
}

#[derive(Clone, Debug)]
pub enum ValType {
  Array(TypeVar),
  Bool,
  Fun(Box<[TypeVar]>, TypeVar),
  I64,
}

type RetType = Box<[TypeVar]>;

pub enum TypeState {
  Abstract,
  TypeError,
  ValType(ValType),
  RetType(RetType),
}

pub struct TypeSolver {
  vars: UnionFind<TypeState>,
  todo: Buf<(TypeVar, TypeVar)>,
}

struct Env {
  args: Buf<TypeVar>,
  outs: Buf<TypeVar>,
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

  fn entry(&self, i: u32) -> (&[TypeVar], TypeVar) {
    let InstType::Entry(ref xs, y) = self.insts[i] else { unreachable!() };
    return (xs, y);
  }

  fn label(&self, i: u32) -> &[TypeVar] {
    let InstType::Label(ref xs) = self.insts[i] else { unreachable!() };
    return xs;
  }

  fn local(&self, i: u32) -> TypeVar {
    let InstType::Local(x) = self.insts[i] else { unreachable!() };
    return x;
  }

  fn value(&self, i: u32) -> TypeVar {
    let InstType::Value(x) = self.insts[i] else { unreachable!() };
    return x;
  }

  pub fn insts(&self) -> impl Iterator<Item = &InstType> {
    return self.insts.iter();
  }
}

fn unify_valtype(x: ValType, y: ValType, todo: &mut Buf<(TypeVar, TypeVar)>) -> TypeState {
  match (x, y) {
    (ValType::Bool, ValType::Bool) => {
      TypeState::ValType(ValType::Bool)
    }
    (ValType::I64, ValType::I64) => {
      TypeState::ValType(ValType::I64)
    }
    (ValType::Array(x), ValType::Array(y)) => {
      todo.put((x, y));
      TypeState::ValType(ValType::Array(x))
    }
    (ValType::Fun(xs, u), ValType::Fun(ys, v)) => {
      if xs.len() != ys.len() {
        TypeState::TypeError
      } else {
        for (x, y) in zip(xs.iter(), ys.iter()) {
          todo.put((*x, *y));
        }
        todo.put((u, v));
        TypeState::ValType(ValType::Fun(xs, u))
      }
    }
    (_, _) => {
      TypeState::TypeError
    }
  }
}

fn unify_rettype(xs: RetType, ys: RetType, todo: &mut Buf<(TypeVar, TypeVar)>) -> TypeState {
  if xs.len() != ys.len() {
    return TypeState::TypeError;
  }

  for (x, y) in zip(xs.iter(), ys.iter()) {
    todo.put((*x, *y));
  }

  return TypeState::RetType(xs);
}

impl TypeSolver {
  fn new() -> Self {
    return Self {
      vars: UnionFind::new(),
      todo: Buf::new(),
    };
  }

  fn fresh(&mut self) -> TypeVar {
    return TypeVar(self.vars.put(TypeState::Abstract));
  }

  fn bound(&mut self, x: TypeVar, t: ValType) {
    let x = &mut self.vars[x.0];

    *x =
      match replace(x, TypeState::Abstract) {
        TypeState::TypeError | TypeState::RetType(..) =>
          TypeState::TypeError,
        TypeState::Abstract =>
          TypeState::ValType(t),
        TypeState::ValType(x) =>
          unify_valtype(x, t, &mut self.todo),
      };
  }

  fn bound_ret(&mut self, x: TypeVar, t: RetType) {
    let x = &mut self.vars[x.0];

    *x =
      match replace(x, TypeState::Abstract) {
        TypeState::TypeError | TypeState::ValType(..) =>
          TypeState::TypeError,
        TypeState::Abstract =>
          TypeState::RetType(t),
        TypeState::RetType(x) =>
          unify_rettype(x, t, &mut self.todo),
      };
  }

  fn unify(&mut self, x: TypeVar, y: TypeVar) {
    if let (x, Some(y)) = self.vars.union(x.0, y.0) {
      *x =
        match (replace(x, TypeState::Abstract), y) {
          | (TypeState::TypeError, _)
          | (_, TypeState::TypeError)
          | (TypeState::ValType(..), TypeState::RetType(..))
          | (TypeState::RetType(..), TypeState::ValType(..)) =>
            TypeState::TypeError,
          | (TypeState::Abstract, t)
          | (t, TypeState::Abstract) =>
            t,
          (TypeState::ValType(x), TypeState::ValType(y)) =>
            unify_valtype(x, y, &mut self.todo),
          (TypeState::RetType(x), TypeState::RetType(y)) =>
            unify_rettype(x, y, &mut self.todo),
        };
    }
  }

  fn propagate(&mut self) {
    while let Some((x, y)) = self.todo.pop_if_nonempty() {
      self.unify(x, y);
    }
  }

  pub fn resolve(&self, x: TypeVar) -> ir1::ValType {
    // recursive types?

    match &self.vars[x.0] {
      TypeState::Abstract => ir1::ValType::Abstract,
      TypeState::TypeError => ir1::ValType::TypeError, // ???
      TypeState::RetType(..) => ir1::ValType::TypeError, // ???
      TypeState::ValType(t) => {
        match t {
          ValType::Array(a) => ir1::ValType::Array(Box::new(self.resolve(*a))),
          ValType::Bool => ir1::ValType::Bool,
          ValType::I64 => ir1::ValType::I64,
          ValType::Fun(xs, y) =>
            ir1::ValType::Fun(
              xs.iter().map(|x| self.resolve(*x)).collect(),
              self.resolve_ret(*y)),
        }
      }
    }
  }

  pub fn resolve_ret(&self, x: TypeVar) -> Option<Box<[ir1::ValType]>> {
    // ???
    if let TypeState::RetType(xs) = &self.vars[x.0] {
      return Some(xs.iter().map(|x| self.resolve(*x)).collect());
    } else {
      return None;
    }
  }
}

impl Env {
  fn new() -> Self {
    return Self {
      args: Buf::new(),
      outs: Buf::new(),
      map: TypeMap::new(),
      solver: TypeSolver::new(),
      is_call: false,
    }
  }
}

pub fn typecheck(module: &Module) -> (TypeMap, TypeSolver) {
  let mut env = Env::new();

  // assign type variables for all relevant program points

  for &inst in module.code.iter() {
    match inst {
      | Inst::GotoStaticError
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
      | Inst::Pop
      | Inst::Const(..)
      | Inst::ConstBool(..)
      | Inst::ConstInt(..)
      | Inst::Field(..)
      | Inst::Index(..)
      | Inst::GetLocal(..)
      | Inst::Op1(..)
      | Inst::Op2(..) =>
        env.map.put(InstType::Value(env.solver.fresh())),
      | Inst::Local(..) =>
        env.map.put(InstType::Local(env.solver.fresh())),
      | Inst::Entry(n) => {
        let xs = (0 .. n).map(|_| env.solver.fresh()).collect();
        let y = env.solver.fresh();
        env.map.put(InstType::Entry(xs, y));
      }
      | Inst::Label(n) => {
        let xs = (0 .. n).map(|_| env.solver.fresh()).collect();
        env.map.put(InstType::Label(xs));
      }
    }
  }

  for &Item::Fun { pos, len } in module.items.iter() {
    let mut rettypevar = TypeVar(u32::MAX);
    // apply initial type constraints

    for i in pos .. pos + len {
      match module.code[i as usize] {
        Inst::ConstBool(_) =>
          env.solver.bound(env.map.value(i), ValType::Bool),
        Inst::ConstInt(_) =>
          env.solver.bound(env.map.value(i), ValType::I64),
        Inst::Local(x) =>
          env.solver.unify(env.map.value(x), env.map.local(i)),
        Inst::GetLocal(x) =>
          env.solver.unify(env.map.local(x), env.map.value(i)),
        Inst::SetLocal(x, y) =>
          env.solver.unify(env.map.value(y), env.map.local(x)),
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
        Inst::Entry(..) => {
          env.is_call = false;
          env.args.clear();
          env.outs.clear();
          let (args, ret) = env.map.entry(i);
          for &arg in args.iter().rev() { env.args.put(arg); }
          rettypevar = ret;
        }
        Inst::Label(..) => {
          env.is_call = false;
          env.args.clear();
          env.outs.clear();
          for &arg in env.map.label(i).iter().rev() { env.args.put(arg); }
        }
        Inst::Pop =>
          env.solver.unify(env.map.value(i), env.args.pop()),
        Inst::Put(x) =>
          env.outs.put(env.map.value(x)),
        Inst::Ret =>
          env.solver.bound_ret(rettypevar, env.outs.drain().collect()),
        Inst::Cond(x) =>
          env.solver.bound(env.map.value(x), ValType::Bool),
        Inst::Goto(a) => {
          if env.is_call {
            // TODO - handle call continuations
            //
            // unify function RetTypeVar with label argument types

          } else {
            for (&x, &y) in zip(env.outs.iter(), env.map.label(a).iter()) {
              env.solver.unify(x, y);
            }
          }
        }
        Inst::Call(f) => {
          let xs = env.outs.drain().collect();
          let y = env.solver.fresh(); // TODO
          env.solver.bound(env.map.value(f), ValType::Fun(xs, y));
          env.is_call = true; // TODO
        }
        Inst::TailCall(f) => {
          let xs = env.outs.drain().collect();
          let y = env.solver.fresh(); // TODO
          env.solver.bound(env.map.value(f), ValType::Fun(xs, y));
          env.is_call = true; // TODO
        }
        | Inst::Const(..)
        | Inst::Field(..)
        | Inst::GotoStaticError
        | Inst::SetField(..) => {
        }
      }
    }

    // solve all type constraints

    env.solver.propagate();
  }

  return (env.map, env.solver);
}
