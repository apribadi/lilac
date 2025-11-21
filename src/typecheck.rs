//!
//!
//! linearized code -> typed code

use crate::ir1;
use crate::ir1::Op1;
use crate::ir1::Op2;
use crate::ir1::Inst;
use crate::buf::Buf;
use crate::union_find::UnionFind;
use std::iter::zip;
use std::mem::replace;

#[derive(Clone, Copy, Debug)]
pub struct TypeVar(u32);

#[derive(Clone, Copy, Debug)]
pub struct RetTypeVar(u32);

#[derive(Clone)]
pub enum InstType {
  Entry(Box<[TypeVar]>, RetTypeVar),
  Label(Box<[TypeVar]>),
  Local(TypeVar),
  Nil,
  Value(TypeVar),
}

enum Todo {
  ValType(TypeVar, TypeVar),
  RetType(RetTypeVar, RetTypeVar),
}

pub struct TypeMap {
  insts: Buf<InstType>,
}

pub struct TypeSolver {
  valtypes: UnionFind<ValType>,
  rettypes: UnionFind<RetType>,
  todo: Buf<Todo>,
}

#[derive(Clone, Debug)]
pub enum ValType {
  Abstract,
  Array(TypeVar),
  Bool,
  Fun(Box<[TypeVar]>, RetTypeVar),
  I64,
  TypeError,
}

#[derive(Clone, Debug)]
pub enum RetType {
  Abstract,
  Values(Box<[TypeVar]>),
  TypeError,
}

struct Env {
  args: Buf<TypeVar>,
  outs: Buf<TypeVar>,
  ret: RetTypeVar,
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

  fn entry(&self, i: u32) -> (&[TypeVar], RetTypeVar) {
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

fn unify_impl(x: &mut ValType, y: ValType, todo: &mut Buf<Todo>) {
  use ValType::*;

  *x =
    match (replace(x, Abstract), y) {
      (Abstract, t) => t,
      (t, Abstract) => t,
      (Bool, Bool) => Bool,
      (I64, I64) => I64,
      (Array(x), Array(y)) => {
        todo.put(Todo::ValType(x, y));
        Array(x)
      }
      (Fun(xs, u), Fun(ys, v)) => {
        if xs.len() != ys.len() {
          TypeError
        } else {
          for (x, y) in zip(xs.iter(), ys.iter()) {
            todo.put(Todo::ValType(*x, *y));
          }
          todo.put(Todo::RetType(u, v));
          Fun(xs, u)
        }
      }
      (_, _) => TypeError,
    };
}

fn unify_ret_impl(x: &mut RetType, y: RetType, todo: &mut Buf<Todo>) {
  *x =
    match (replace(x, RetType::Abstract), y) {
      (RetType::Abstract, t) | (t, RetType::Abstract) =>
        t,
      (RetType::TypeError, _) | (_, RetType::TypeError) =>
        RetType::TypeError,
      (RetType::Values(xs), RetType::Values(ys)) => {
        if xs.len() != ys.len() {
          RetType::TypeError
        } else {
          for (&x, &y) in zip(xs.iter(), ys.iter()) {
            todo.put(Todo::ValType(x, y));
          }
          RetType::Values(xs)
        }
      }
    };
}

impl TypeSolver {
  fn new() -> Self {
    return Self {
      valtypes: UnionFind::new(),
      rettypes: UnionFind::new(),
      todo: Buf::new(),
    };
  }

  fn fresh(&mut self) -> TypeVar {
    return TypeVar(self.valtypes.put(ValType::Abstract));
  }

  fn fresh_ret(&mut self) -> RetTypeVar {
    return RetTypeVar(self.rettypes.put(RetType::Abstract));
  }

  fn bound(&mut self, x: TypeVar, t: ValType) {
    unify_impl(&mut self.valtypes[x.0], t, &mut self.todo);
  }

  fn bound_ret(&mut self, x: RetTypeVar, t: Box<[TypeVar]>) {
    unify_ret_impl(&mut self.rettypes[x.0], RetType::Values(t), &mut self.todo);
  }

  fn unify(&mut self, x: TypeVar, y: TypeVar) {
    if let (x, Some(y)) = self.valtypes.union(x.0, y.0) {
      unify_impl(x, y, &mut self.todo);
    }
  }

  fn propagate(&mut self) {
    while let Some(todo) = self.todo.pop_if_nonempty() {
      match todo {
        Todo::ValType(x, y) => {
          if let (x, Some(y)) = self.valtypes.union(x.0, y.0) {
            unify_impl(x, y, &mut self.todo);
          }
        }
        Todo::RetType(x, y) => {
          if let (x, Some(y)) = self.rettypes.union(x.0, y.0) {
            unimplemented!()
          }
        }
      }
    }
  }

  pub fn resolve(&self, x: TypeVar) -> ir1::ValType {
    // recursive types?

    match &self.valtypes[x.0] {
      ValType::Abstract => ir1::ValType::Abstract,
      ValType::Array(a) => ir1::ValType::Array(Box::new(self.resolve(*a))),
      ValType::Bool => ir1::ValType::Bool,
      ValType::I64 => ir1::ValType::I64,
      ValType::Fun(xs, y) =>
        ir1::ValType::Fun(
          xs.iter().map(|x| self.resolve(*x)).collect(),
          self.resolve_ret(*y)),
      ValType::TypeError => ir1::ValType::TypeError, // ???
    }
  }

  pub fn resolve_ret(&self, x: RetTypeVar) -> Option<Box<[ir1::ValType]>> {
    // ???
    match &self.rettypes[x.0] {
      RetType::Abstract => None,
      RetType::TypeError => None,
      RetType::Values(xs) =>
        Some(xs.iter().map(|x| self.resolve(*x)).collect()),
    }
  }
}

impl Env {
  fn new() -> Self {
    return Self {
      args: Buf::new(),
      outs: Buf::new(),
      ret: RetTypeVar(u32::MAX),
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
      | Inst::Entry(n) => {
        let xs = (0 .. n).map(|_| env.solver.fresh()).collect();
        let y = env.solver.fresh_ret();
        env.map.put(InstType::Entry(xs, y));
      }
      | Inst::Label(n) => {
        let xs = (0 .. n).map(|_| env.solver.fresh()).collect();
        env.map.put(InstType::Label(xs));
      }
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
    }
  }

  // apply initial type constraints

  for (i, &inst) in code.iter().enumerate() {
    let i = i as u32;
    match inst {
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
      Inst::Entry(..) => {
        env.is_call = false;
        env.args.clear();
        env.outs.clear();
        let (args, ret) = env.map.entry(i);
        for &arg in args.iter().rev() { env.args.put(arg); }
        env.ret = ret;
      }
      Inst::Label(..) => {
        env.is_call = false;
        env.args.clear();
        env.outs.clear();
        let args = env.map.label(i);
        for &arg in args.iter().rev() { env.args.put(arg); }
      }
      Inst::Pop => {
        env.solver.unify(env.map.value(i), env.args.pop());
      }
      Inst::Put(x) => {
        env.outs.put(env.map.value(x));
      }
      Inst::Ret => {
        env.solver.bound_ret(env.ret, env.outs.drain().collect());
      }
      Inst::Cond(x) => {
        env.solver.bound(env.map.value(x), ValType::Bool);
      }
      Inst::Goto(a) => {
        // TODO - handle call continuations
        if ! env.is_call {
          for (&x, &y) in zip(env.outs.iter(), env.map.label(a).iter()) {
            env.solver.unify(x, y);
          }
        }
      }
      Inst::Call(f) => {
        let xs = env.outs.drain().collect();
        let y = env.solver.fresh_ret(); // TODO
        env.solver.bound(env.map.value(f), ValType::Fun(xs, y));
        env.is_call = true; // TODO
      }
      Inst::TailCall(f) => {
        let xs = env.outs.drain().collect();
        let y = env.solver.fresh_ret(); // TODO
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

  return (env.map, env.solver);
}
