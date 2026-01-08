//!
//!
//! linearized code -> typed code

use crate::arr::Arr;
use crate::buf::Buf;
use crate::hir::Inst;
use crate::hir::Module;
use crate::prim::PrimType;
use crate::prim::PrimOp1;
use crate::prim::PrimOp2;
use crate::hir;
use crate::operator::Op1;
use crate::operator::Op2;
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
pub enum TypeCon {
  Array(TypeVar),
  Fun(Arr<TypeVar>, TypeVar),
  PrimType(PrimType),
}

#[derive(Clone, Debug)]
pub enum Type {
  Array(Box<Type>),
  Bool,
  Fun(Arr<Type>, Arr<Type>),
  I64,
  Var(TypeVar),
}

#[derive(Clone, Debug)]
pub struct TypeScheme(/* arity */ u32, Type);

type TypeSeq = Arr<TypeVar>;

pub enum TypeState {
  Abstract,
  TypeCon(TypeCon),
  TypeError,
  TypeGen(TypeVar),
  TypeSeq(TypeSeq),
}

pub struct TypeSolver {
  vars: UnionFind<TypeState>,
  todo: Buf<(TypeVar, TypeVar)>,
}

struct Ctx {
  environment: HashMap<Symbol, TypeScheme>,
  current_items: HashMap<Symbol, TypeVar>,
  insts: TypeMap,
  solver: TypeSolver,
  block: u32,
  outs: Buf<TypeVar>,
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

fn unify_valtype(x: TypeCon, y: TypeCon, todo: &mut Buf<(TypeVar, TypeVar)>) -> TypeState {
  match (x, y) {
    (TypeCon::PrimType(x), TypeCon::PrimType(y)) if x == y => {
      TypeState::TypeCon(TypeCon::PrimType(x))
    }
    (TypeCon::Array(x), TypeCon::Array(y)) => {
      todo.put((x, y));
      TypeState::TypeCon(TypeCon::Array(x))
    }
    (TypeCon::Fun(xs, u), TypeCon::Fun(ys, v)) => {
      if xs.len() != ys.len() {
        TypeState::TypeError
      } else {
        for (x, y) in zip(xs.iter(), ys.iter()) {
          todo.put((*x, *y));
        }
        todo.put((u, v));
        TypeState::TypeCon(TypeCon::Fun(xs, u))
      }
    }
    (_, _) => {
      TypeState::TypeError
    }
  }
}

fn unify_rettype(xs: TypeSeq, ys: TypeSeq, todo: &mut Buf<(TypeVar, TypeVar)>) -> TypeState {
  if xs.len() != ys.len() {
    return TypeState::TypeError;
  }

  for (x, y) in zip(xs.iter(), ys.iter()) {
    todo.put((*x, *y));
  }

  return TypeState::TypeSeq(xs);
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

  fn bound(&mut self, x: TypeVar, t: TypeCon) {
    let x = &mut self.vars[x.0];

    *x =
      match replace(x, TypeState::Abstract) {
        TypeState::TypeError | TypeState::TypeGen(..) | TypeState::TypeSeq(..) =>
          TypeState::TypeError,
        TypeState::Abstract =>
          TypeState::TypeCon(t),
        TypeState::TypeCon(x) =>
          unify_valtype(x, t, &mut self.todo),
      };
  }

  fn bound_ret(&mut self, x: TypeVar, t: TypeSeq) {
    let x = &mut self.vars[x.0];

    *x =
      match replace(x, TypeState::Abstract) {
        TypeState::TypeError | TypeState::TypeGen(..) | TypeState::TypeCon(..) =>
          TypeState::TypeError,
        TypeState::Abstract =>
          TypeState::TypeSeq(t),
        TypeState::TypeSeq(x) =>
          unify_rettype(x, t, &mut self.todo),
      };
  }

  fn unify(&mut self, x: TypeVar, y: TypeVar) {
    if let (x, Some(y)) = self.vars.union(x.0, y.0) {
      *x =
        match (replace(x, TypeState::Abstract), y) {
          | (TypeState::TypeError, _)
          | (_, TypeState::TypeError)
          | (TypeState::TypeGen(..), _)
          | (_, TypeState::TypeGen(..))
          | (TypeState::TypeCon(..), TypeState::TypeSeq(..))
          | (TypeState::TypeSeq(..), TypeState::TypeCon(..)) =>
            TypeState::TypeError,
          | (TypeState::Abstract, t)
          | (t, TypeState::Abstract) =>
            t,
          (TypeState::TypeCon(x), TypeState::TypeCon(y)) =>
            unify_valtype(x, y, &mut self.todo),
          (TypeState::TypeSeq(x), TypeState::TypeSeq(y)) =>
            unify_rettype(x, y, &mut self.todo),
        };
    }
  }

  fn propagate(&mut self) {
    while let Some((x, y)) = self.todo.pop_if_nonempty() {
      self.unify(x, y);
    }
  }

  fn instantiate(&mut self, t: &TypeScheme) -> TypeVar {
    let TypeScheme(n, ref t) = *t;
    let v = Arr::new((0 .. n).map(|_| self.fresh()));
    return self.instantiate_type(&v, t);
  }

  fn instantiate_type(&mut self, v: &Arr<TypeVar>, t: &Type) -> TypeVar {
    match *t {
      Type::Array(ref t) => {
        let a = self.fresh();
        let b = self.instantiate_type(v, t);
        self.bound(a, TypeCon::Array(b));
        return a;
      }
      Type::Bool => {
        let a = self.fresh();
        self.bound(a, TypeCon::PrimType(PrimType::Bool));
        return a;
      }
      Type::Fun(ref x, ref y) => {
        let a = self.fresh();
        let x = Arr::new(x.iter().map(|x| self.instantiate_type(v, x)));
        let b = self.fresh();
        let y = Arr::new(y.iter().map(|y| self.instantiate_type(v, y)));
        self.bound_ret(b, y);
        self.bound(a, TypeCon::Fun(x, b));
        return a;
      }
      Type::I64 => {
        let a = self.fresh();
        self.bound(a, TypeCon::PrimType(PrimType::Bool));
        return a;
      }
      Type::Var(TypeVar(i)) => {
        return v[i];
      }
    }
  }

  fn generalize(&mut self, t: TypeVar) -> TypeScheme {
    // TODO: we actually need to generalize multiple typevars at the same time,
    // from a strongly-connected-component of top-level items

    let mut i = 0u32;
    let _ = i;
    let _ = t;
    return TypeScheme(0, Type::Bool);
  }

  fn generalize_impl(&mut self, t: TypeVar, i: &mut u32) -> Type {
    let _ = self;
    let _ = t;
    let _ = i;
    unimplemented!()
  }

  pub fn resolve(&self, x: TypeVar) -> hir::ValType {
    // TODO: we should do an occurs check to prohibit recursive types.

    match self.vars[x.0] {
      TypeState::TypeGen(..) => unimplemented!(),
      TypeState::Abstract => hir::ValType::Abstract,
      TypeState::TypeError => hir::ValType::TypeError, // ???
      TypeState::TypeSeq(..) => hir::ValType::TypeError, // ???
      TypeState::TypeCon(ref t) => {
        match *t {
          TypeCon::Array(a) => hir::ValType::Array(Box::new(self.resolve(a))),
          TypeCon::PrimType(PrimType::Bool) => hir::ValType::Bool,
          TypeCon::PrimType(PrimType::I64) => hir::ValType::I64,
          TypeCon::Fun(ref xs, y) =>
            hir::ValType::Fun(
              xs.iter().map(|x| self.resolve(*x)).collect(),
              self.resolve_ret(y)),
        }
      }
    }
  }

  pub fn resolve_ret(&self, x: TypeVar) -> Option<Arr<hir::ValType>> {
    // ???
    if let TypeState::TypeSeq(ref xs) = self.vars[x.0] {
      return Some(xs.iter().map(|x| self.resolve(*x)).collect());
    } else {
      return None;
    }
  }
}

impl Ctx {
  fn new() -> Self {
    let mut ctx =
      Self {
        environment: HashMap::new(),
        current_items: HashMap::new(),
        insts: TypeMap::new(),
        solver: TypeSolver::new(),
        block: u32::MAX,
        outs: Buf::new(),
        call_rettypevar: None,
      };

    ctx.environment.insert(
      Symbol::from_str("len"),
      TypeScheme(1, Type::Fun(Arr::new([Type::Var(TypeVar(0))]), Arr::new([Type::I64])))
    );

    return ctx;
  }
}

pub fn typecheck(module: &Module) -> (HashMap<Symbol, TypeScheme>, Buf<InstType>, TypeSolver) {
  let mut ctx = Ctx::new();

  // assign type variables for all relevant program points

  for inst in module.code.iter() {
    match *inst {
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
        ctx.insts.put(InstType::Nil),
      | Inst::Get(..)
      | Inst::Const(..)
      | Inst::ConstBool(..)
      | Inst::ConstInt(..)
      | Inst::Field(..)
      | Inst::Index(..)
      | Inst::GetLocal(..)
      | Inst::Op1(..)
      | Inst::Op2(..) =>
        ctx.insts.put(InstType::Value(ctx.solver.fresh())),
      | Inst::Local(..) =>
        ctx.insts.put(InstType::Local(ctx.solver.fresh())),
      | Inst::Label(n) => {
        let xs = (0 .. n).map(|_| ctx.solver.fresh()).collect();
        ctx.insts.put(InstType::Label(xs));
      }
    }
  }

  for f in module.decl.iter() {
    let funtypevar = ctx.solver.fresh();
    let rettypevar = ctx.solver.fresh();
    ctx.solver.bound(funtypevar, TypeCon::Fun(ctx.insts.label(f.pos).clone(), rettypevar));
    ctx.current_items.insert(f.name, funtypevar);

    // apply initial type constraints

    for i in f.pos .. f.pos + f.len {
      match module.code[i] {
        Inst::ConstBool(_) =>
          ctx.solver.bound(ctx.insts.value(i), TypeCon::PrimType(PrimType::Bool)),
        Inst::ConstInt(_) =>
          ctx.solver.bound(ctx.insts.value(i), TypeCon::PrimType(PrimType::I64)),
        Inst::Local(x) =>
          ctx.solver.unify(ctx.insts.value(x), ctx.insts.local(i)),
        Inst::GetLocal(v) =>
          ctx.solver.unify(ctx.insts.local(v), ctx.insts.value(i)),
        Inst::SetLocal(v, x) =>
          ctx.solver.unify(ctx.insts.value(x), ctx.insts.local(v)),
        Inst::Index(x, y) => {
          let a = ctx.solver.fresh();
          ctx.solver.bound(ctx.insts.value(x), TypeCon::Array(a));
          ctx.solver.bound(ctx.insts.value(y), TypeCon::PrimType(PrimType::I64));
          ctx.solver.unify(a, ctx.insts.value(i));
        }
        Inst::SetIndex(x, y, z) => {
          let a = ctx.solver.fresh();
          ctx.solver.bound(ctx.insts.value(x), TypeCon::Array(a));
          ctx.solver.bound(ctx.insts.value(y), TypeCon::PrimType(PrimType::I64));
          ctx.solver.unify(ctx.insts.value(z), a);
        }
        Inst::Op1(f, x) => {
          let f = lower_op1(f);
          let a = TypeCon::PrimType(f.arg_type());
          let b = TypeCon::PrimType(f.out_type());
          ctx.solver.bound(ctx.insts.value(x), a);
          ctx.solver.bound(ctx.insts.value(i), b);
        }
        Inst::Op2(f, x, y) => {
          let f = lower_op2(f);
          let a = TypeCon::PrimType(f.arg_type()[0]);
          let b = TypeCon::PrimType(f.arg_type()[1]);
          let c = TypeCon::PrimType(f.out_type());
          ctx.solver.bound(ctx.insts.value(x), a);
          ctx.solver.bound(ctx.insts.value(y), b);
          ctx.solver.bound(ctx.insts.value(i), c);
        }
        Inst::Label(..) => {
          ctx.block = i;
          ctx.call_rettypevar = None;
          ctx.outs.clear();
        }
        Inst::Get(k) =>
          ctx.solver.unify(ctx.insts.value(i), ctx.insts.label(ctx.block)[k]),
        Inst::Put(_, x) =>
          ctx.outs.put(ctx.insts.value(x)),
        Inst::Ret =>
          ctx.solver.bound_ret(rettypevar, ctx.outs.drain().collect()),
        Inst::Cond(x) =>
          ctx.solver.bound(ctx.insts.value(x), TypeCon::PrimType(PrimType::Bool)),
        Inst::Goto(a) => {
          match ctx.call_rettypevar {
            None => {
              for (&x, &y) in zip(ctx.outs.iter(), ctx.insts.label(a).iter()) {
                ctx.solver.unify(x, y);
              }
            }
            Some(ret) =>  {
              ctx.solver.bound_ret(ret, ctx.insts.label(a).clone());
            }
          }
        }
        Inst::Call(f) => {
          let xs = ctx.outs.drain().collect();
          let y = ctx.solver.fresh();
          ctx.solver.bound(ctx.insts.value(f), TypeCon::Fun(xs, y));
          ctx.call_rettypevar = Some(y);
        }
        Inst::TailCall(f) => {
          let xs = ctx.outs.drain().collect();
          let y = ctx.solver.fresh();
          ctx.solver.bound(ctx.insts.value(f), TypeCon::Fun(xs, y));
          ctx.solver.unify(rettypevar, y);
        }
        Inst::Const(symbol) => {
          if let Some(&x) = ctx.current_items.get(symbol) {
            ctx.solver.unify(ctx.insts.value(i), x);
          } else if let Some(t) = ctx.environment.get(symbol) {
            let x = ctx.solver.instantiate(t);
            ctx.solver.unify(ctx.insts.value(i), x);
          } else {
            // TODO: error unbound variable
          }
        }
        | Inst::Field(..)
        | Inst::GotoStaticError
        | Inst::SetField(..) => {
        }
      }
    }

    // solve all type constraints

    ctx.solver.propagate();

    // TODO: generalize

    ctx.current_items.clear();
    ctx.environment.insert(f.name, ctx.solver.generalize(funtypevar));
  }

  return (ctx.environment, ctx.insts.insts, ctx.solver);
}

fn lower_op1(op: Op1) -> PrimOp1 {
  match op {
    Op1::Dec => PrimOp1::DecI64,
    Op1::Inc => PrimOp1::IncI64,
    Op1::Neg => PrimOp1::NegI64,
    Op1::Not => PrimOp1::NotBool,
  }
}

fn lower_op2(op: Op2) -> PrimOp2 {
  match op {
    Op2::Add => PrimOp2::AddI64,
    Op2::BitAnd => PrimOp2::BitAndI64,
    Op2::BitOr => PrimOp2::BitOrI64,
    Op2::BitXor => PrimOp2::BitXorI64,
    Op2::CmpEq => PrimOp2::CmpEqI64,
    Op2::CmpGe => PrimOp2::CmpGeI64,
    Op2::CmpGt => PrimOp2::CmpGtI64,
    Op2::CmpLe => PrimOp2::CmpLeI64,
    Op2::CmpLt => PrimOp2::CmpLtI64,
    Op2::CmpNe => PrimOp2::CmpNeI64,
    Op2::Div => PrimOp2::DivI64,
    Op2::Mul => PrimOp2::MulI64,
    Op2::Rem => PrimOp2::RemI64,
    Op2::Shl => PrimOp2::ShlI64,
    Op2::Shr => PrimOp2::ShrI64,
    Op2::Sub => PrimOp2::SubI64,
  }
}
