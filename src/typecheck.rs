//!
//!
//! linearized code -> typed code

use crate::arr::Arr;
use crate::buf::Buf;
use crate::ir1::Inst;
use crate::ir1::Item;
use crate::ir1::Module;
use crate::ir1::Op1;
use crate::ir1::Op2;
use crate::ir1;
use crate::symbol::Symbol;
use crate::union_find::UnionFind;
use std::iter::zip;
use std::mem::replace;
use tangerine::map::HashMap;

#[derive(Clone, Copy, Debug)]
pub struct TypeVar(u32);

#[derive(Clone)]
pub enum InstType {
  Label(Arr<TypeVar>),
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
  Fun(Arr<TypeVar>, TypeVar),
  I64,
}

type RetType = Arr<TypeVar>;

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
  items: HashMap<Symbol, TypeVar>,
  insts: TypeMap,
  block: u32,
  outs: Buf<TypeVar>,
  solver: TypeSolver,
  call_rettypevar: Option<TypeVar>,
}

impl TypeMap {
  fn new() -> Self {
    return Self { insts: Buf::new() };
  }

  fn put(&mut self, x: InstType) {
    self.insts.put(x);
  }

  fn label(&self, i: u32) -> &Arr<TypeVar> {
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

  pub fn resolve_ret(&self, x: TypeVar) -> Option<Arr<ir1::ValType>> {
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
      items: HashMap::new(),
      insts: TypeMap::new(),
      block: u32::MAX,
      outs: Buf::new(),
      solver: TypeSolver::new(),
      call_rettypevar: None,
    }
  }
}

pub fn typecheck(module: &Module) -> (HashMap<Symbol, TypeVar>, TypeMap, TypeSolver) {
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
        env.insts.put(InstType::Nil),
      | Inst::Get(..)
      | Inst::Const(..)
      | Inst::ConstBool(..)
      | Inst::ConstInt(..)
      | Inst::Field(..)
      | Inst::Index(..)
      | Inst::GetLocal(..)
      | Inst::Op1(..)
      | Inst::Op2(..) =>
        env.insts.put(InstType::Value(env.solver.fresh())),
      | Inst::Local(..) =>
        env.insts.put(InstType::Local(env.solver.fresh())),
      | Inst::Label(n) => {
        let xs = (0 .. n).map(|_| env.solver.fresh()).collect();
        env.insts.put(InstType::Label(xs));
      }
    }
  }

  for &Item::Fun { name, pos, len } in module.items.iter() {
    let funtypevar = env.solver.fresh();
    let rettypevar = env.solver.fresh();
    env.solver.bound(funtypevar, ValType::Fun(env.insts.label(pos).clone(), rettypevar));
    let _ = env.items.insert(name, funtypevar);

    // apply initial type constraints

    for i in pos .. pos + len {
      match module.code[i] {
        Inst::ConstBool(_) =>
          env.solver.bound(env.insts.value(i), ValType::Bool),
        Inst::ConstInt(_) =>
          env.solver.bound(env.insts.value(i), ValType::I64),
        Inst::Local(x) =>
          env.solver.unify(env.insts.value(x), env.insts.local(i)),
        Inst::GetLocal(v) =>
          env.solver.unify(env.insts.local(v), env.insts.value(i)),
        Inst::SetLocal(v, x) =>
          env.solver.unify(env.insts.value(x), env.insts.local(v)),
        Inst::Index(x, y) => {
          let a = env.solver.fresh();
          env.solver.bound(env.insts.value(x), ValType::Array(a));
          env.solver.bound(env.insts.value(y), ValType::I64);
          env.solver.unify(a, env.insts.value(i));
        }
        Inst::SetIndex(x, y, z) => {
          let a = env.solver.fresh();
          env.solver.bound(env.insts.value(x), ValType::Array(a));
          env.solver.bound(env.insts.value(y), ValType::I64);
          env.solver.unify(env.insts.value(z), a);
        }
        Inst::Op1(f, x) => {
          let (a, b) =
            match f {
              | Op1::Neg => (ValType::I64, ValType::I64),
              | Op1::Not => (ValType::Bool, ValType::Bool),
            };
          env.solver.bound(env.insts.value(x), a);
          env.solver.bound(env.insts.value(i), b);
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
          env.solver.bound(env.insts.value(x), a);
          env.solver.bound(env.insts.value(y), b);
          env.solver.bound(env.insts.value(i), c);
        }
        Inst::Label(..) => {
          env.block = i;
          env.call_rettypevar = None;
          env.outs.clear();
        }
        Inst::Get(k) =>
          env.solver.unify(env.insts.value(i), env.insts.label(env.block)[k]),
        Inst::Put(_, x) =>
          env.outs.put(env.insts.value(x)),
        Inst::Ret =>
          env.solver.bound_ret(rettypevar, env.outs.drain().collect()),
        Inst::Cond(x) =>
          env.solver.bound(env.insts.value(x), ValType::Bool),
        Inst::Goto(a) => {
          match env.call_rettypevar {
            None => {
              for (&x, &y) in zip(env.outs.iter(), env.insts.label(a).iter()) {
                env.solver.unify(x, y);
              }
            }
            Some(ret) =>  {
              env.solver.bound_ret(ret, env.insts.label(a).clone());
            }
          }
        }
        Inst::Call(f) => {
          let xs = env.outs.drain().collect();
          let y = env.solver.fresh();
          env.solver.bound(env.insts.value(f), ValType::Fun(xs, y));
          env.call_rettypevar = Some(y);
        }
        Inst::TailCall(f) => {
          let xs = env.outs.drain().collect();
          let y = env.solver.fresh();
          env.solver.bound(env.insts.value(f), ValType::Fun(xs, y));
          env.solver.unify(rettypevar, y);
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

  return (env.items, env.insts, env.solver);
}
