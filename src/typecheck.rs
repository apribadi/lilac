//!
//!
//! linearized code -> typed code

use crate::arr::Arr;
use crate::buf::Buf;
use crate::hir::Inst;
use crate::hir;
use crate::operator::Op1;
use crate::operator::Op2;
use crate::prim::PrimOp1;
use crate::prim::PrimOp2;
use crate::prim::PrimType;
use crate::symbol::Symbol;
use crate::typevar::TypeVar;
use crate::union_find::UnionFind;
use std::iter::zip;
use std::mem::replace;
use tangerine::map::HashMap;

#[derive(Clone, Debug)]
pub enum ValueType {
  Array(Box<ValueType>),
  Fun(Arr<ValueType>, Arr<ValueType>),
  I64,
  Bool,
  Var(TypeVar),
}

#[derive(Clone, Debug)]
pub struct TypeScheme(/* arity */ pub u32, pub ValueType);

#[derive(Clone, Debug)]
pub enum ValueTypeState {
  Array(TypeVar),
  Fun(TypeVar, TypeVar),
  PrimType(PrimType),
}

type TupleTypeState = Arr<TypeVar>;

#[derive(Debug)]
pub enum TypeState {
  Abstract,
  TupleType(TupleTypeState),
  TypeError,
  ValueType(ValueTypeState),
}

pub struct Solver {
  union_find: UnionFind<TypeState>,
  to_unify: Buf<(TypeVar, TypeVar)>,
}

struct Ctx {
  global_environment: HashMap<Symbol, TypeScheme>,
  solver_environment: HashMap<Symbol, TypeVar>,
  solver: Solver,
  block: u32,
  block_args: Arr<TypeVar>,
  block_outs: Buf<TypeVar>,
  call_rettypevar: Option<TypeVar>,
}

fn unify_value_type(
    x: ValueTypeState,
    y: ValueTypeState,
    to_unify: &mut Buf<(TypeVar, TypeVar)>
  ) -> TypeState
{
  match (x, y) {
    (ValueTypeState::Array(a), ValueTypeState::Array(b)) => {
      to_unify.put((a, b));
      TypeState::ValueType(ValueTypeState::Array(a))
    }
    (ValueTypeState::Fun(a, b), ValueTypeState::Fun(c, d)) => {
      to_unify.put((a, c));
      to_unify.put((b, d));
      TypeState::ValueType(ValueTypeState::Fun(a, b))
    }
    (ValueTypeState::PrimType(x), ValueTypeState::PrimType(y)) if x == y => {
      TypeState::ValueType(ValueTypeState::PrimType(x))
    }
    (_, _) => {
      TypeState::TypeError
    }
  }
}

fn unify_tuple_type(
    x: TupleTypeState,
    y: TupleTypeState,
    to_unify: &mut Buf<(TypeVar, TypeVar)>
  ) -> TypeState
{
  if x.len() != y.len() {
    return TypeState::TypeError;
  }

  for (x, y) in zip(x.iter(), y.iter()) {
    to_unify.put((*x, *y));
  }

  return TypeState::TupleType(x);
}

impl Solver {
  fn new() -> Self {
    return Self {
      union_find: UnionFind::new(),
      to_unify: Buf::new(),
    };
  }

  fn fresh(&mut self) -> TypeVar {
    return TypeVar(self.union_find.put(TypeState::Abstract));
  }

  fn constrain_value_type(&mut self, x: TypeVar, y: ValueTypeState) {
    let x = &mut self.union_find[x.0];

    *x =
      match replace(x, TypeState::Abstract) {
        TypeState::TypeError | TypeState::TupleType(..) =>
          TypeState::TypeError,
        TypeState::Abstract =>
          TypeState::ValueType(y),
        TypeState::ValueType(x) =>
          unify_value_type(x, y, &mut self.to_unify),
      };
  }

  fn constrain_tuple_type(&mut self, x: TypeVar, y: TupleTypeState) {
    let x = &mut self.union_find[x.0];

    *x =
      match replace(x, TypeState::Abstract) {
        TypeState::TypeError | TypeState::ValueType(..) =>
          TypeState::TypeError,
        TypeState::Abstract =>
          TypeState::TupleType(y),
        TypeState::TupleType(x) =>
          unify_tuple_type(x, y, &mut self.to_unify),
      };
  }

  fn unify(&mut self, x: TypeVar, y: TypeVar) {
    if let (x, Some(y)) = self.union_find.union(x.0, y.0) {
      *x =
        match (replace(x, TypeState::Abstract), y) {
          (TypeState::Abstract, t) | (t, TypeState::Abstract) =>
            t,
          (TypeState::ValueType(x), TypeState::ValueType(y)) =>
            unify_value_type(x, y, &mut self.to_unify),
          (TypeState::TupleType(x), TypeState::TupleType(y)) =>
            unify_tuple_type(x, y, &mut self.to_unify),
          _ =>
            TypeState::TypeError,
        };
    }
  }

  fn propagate(&mut self) {
    while let Some((x, y)) = self.to_unify.pop_if_nonempty() {
      self.unify(x, y);
    }
  }

  fn instantiate(&mut self, t: &TypeScheme) -> TypeVar {
    let _ = self;
    let _ = t;
    return self.fresh();
  }

  fn generalize(&mut self, t: TypeVar) -> TypeScheme {
    // TODO: we actually need to generalize multiple typevars at the same time,
    // from a strongly-connected-component of top-level items

    let _ = self;
    let _ = t;
    return TypeScheme(0, ValueType::Bool);
  }

  pub fn resolve_value_type(&self, t: TypeVar) -> ValueType {
    match self.union_find[t.0] {
      TypeState::Abstract => ValueType::Var(TypeVar(111)), // ???
      TypeState::TupleType(..) => ValueType::Var(TypeVar(999)), // ???
      TypeState::TypeError => ValueType::Var(TypeVar(999)), // ???
      TypeState::ValueType(ValueTypeState::Array(a)) =>
        ValueType::Array(Box::new(self.resolve_value_type(a))),
      TypeState::ValueType(ValueTypeState::Fun(a, b)) =>
        ValueType::Fun(self.resolve_tuple_type(a), self.resolve_tuple_type(b)),
      TypeState::ValueType(ValueTypeState::PrimType(PrimType::Bool)) =>
        ValueType::Bool,
      TypeState::ValueType(ValueTypeState::PrimType(PrimType::I64)) =>
        ValueType::I64,
    }
  }

  pub fn resolve_tuple_type(&self, t: TypeVar) -> Arr<ValueType> {
    match self.union_find[t.0] {
      TypeState::Abstract => Arr::EMPTY,
      TypeState::ValueType(..) => Arr::EMPTY,
      TypeState::TypeError => Arr::EMPTY,
      TypeState::TupleType(ref t) =>
        Arr::new(t.iter().map(|t| self.resolve_value_type(*t))),
    }
  }
}

impl Ctx {
  fn new() -> Self {
    let mut ctx =
      Self {
        global_environment: HashMap::new(),
        solver_environment: HashMap::new(),
        solver: Solver::new(),
        block: u32::MAX,
        block_args: Arr::EMPTY,
        block_outs: Buf::new(),
        call_rettypevar: None,
      };

    ctx.global_environment.insert(
      Symbol::from_str("len"),
      TypeScheme(1, ValueType::Fun(Arr::new([ValueType::Var(TypeVar(0))]), Arr::new([ValueType::I64])))
    );

    return ctx;
  }
}

pub fn typecheck(module: &hir::Module) -> (HashMap<Symbol, TypeScheme>, Solver) {
  let mut ctx = Ctx::new();

  // allocate type variables for all program points

  for _ in module.code.iter() {
    let _ = ctx.solver.fresh();
  }

  // ?

  for f in module.decl.iter() {
    let funtypevar = ctx.solver.fresh();
    let argtypevar = ctx.solver.fresh();
    let rettypevar = ctx.solver.fresh();
    ctx.solver.constrain_value_type(funtypevar, ValueTypeState::Fun(argtypevar, rettypevar));
    ctx.solver_environment.insert(f.name, funtypevar);

    // apply initial type constraints

    for i in f.pos .. f.pos + f.len {
      match module.code[i] {
        Inst::ConstBool(_) =>
          ctx.solver.constrain_value_type(TypeVar(i), ValueTypeState::PrimType(PrimType::Bool)),
        Inst::ConstInt(_) =>
          ctx.solver.constrain_value_type(TypeVar(i), ValueTypeState::PrimType(PrimType::I64)),
        Inst::Local(x) =>
          ctx.solver.unify(TypeVar(x), TypeVar(i)),
        Inst::GetLocal(v) =>
          ctx.solver.unify(TypeVar(v), TypeVar(i)),
        Inst::SetLocal(v, x) =>
          ctx.solver.unify(TypeVar(x), TypeVar(v)),
        Inst::Index(x, y) => {
          let a = ctx.solver.fresh();
          ctx.solver.constrain_value_type(TypeVar(x), ValueTypeState::Array(a));
          ctx.solver.constrain_value_type(TypeVar(y), ValueTypeState::PrimType(PrimType::I64));
          ctx.solver.unify(a, TypeVar(i));
        }
        Inst::SetIndex(x, y, z) => {
          let a = ctx.solver.fresh();
          ctx.solver.constrain_value_type(TypeVar(x), ValueTypeState::Array(a));
          ctx.solver.constrain_value_type(TypeVar(y), ValueTypeState::PrimType(PrimType::I64));
          ctx.solver.unify(TypeVar(z), a);
        }
        Inst::Op1(f, x) => {
          let f = lower_op1(f);
          let a = ValueTypeState::PrimType(f.arg_type());
          let b = ValueTypeState::PrimType(f.out_type());
          ctx.solver.constrain_value_type(TypeVar(x), a);
          ctx.solver.constrain_value_type(TypeVar(i), b);
        }
        Inst::Op2(f, x, y) => {
          let f = lower_op2(f);
          let a = ValueTypeState::PrimType(f.arg_type().0);
          let b = ValueTypeState::PrimType(f.arg_type().1);
          let c = ValueTypeState::PrimType(f.out_type());
          ctx.solver.constrain_value_type(TypeVar(x), a);
          ctx.solver.constrain_value_type(TypeVar(y), b);
          ctx.solver.constrain_value_type(TypeVar(i), c);
        }
        Inst::Label(n) => {
          let a = Arr::new((0 .. n).map(|_| ctx.solver.fresh()));
          ctx.solver.constrain_tuple_type(TypeVar(i), a.clone());
          ctx.block = i;
          ctx.block_args = a;
          ctx.block_outs.clear();
          ctx.call_rettypevar = None;
        }
        Inst::Get(k) =>
          ctx.solver.unify(TypeVar(i), ctx.block_args[k]),
        Inst::Put(_, x) =>
          ctx.block_outs.put(TypeVar(x)),
        Inst::Ret =>
          ctx.solver.constrain_tuple_type(rettypevar, ctx.block_outs.drain().collect()),
        Inst::Cond(x) =>
          ctx.solver.constrain_value_type(TypeVar(x), ValueTypeState::PrimType(PrimType::Bool)),
        Inst::Goto(a) => {
          match ctx.call_rettypevar {
            None => {
              ctx.solver.constrain_tuple_type(TypeVar(a), ctx.block_outs.iter().map(|x| *x).collect());
            }
            Some(ret) => {
              ctx.solver.unify(TypeVar(a), ret);
            }
          }
        }
        Inst::Call(f) => {
          let x = ctx.solver.fresh();
          let y = ctx.solver.fresh();
          ctx.solver.constrain_value_type(TypeVar(f), ValueTypeState::Fun(x, y));
          ctx.solver.constrain_tuple_type(x, ctx.block_outs.drain().collect());
          ctx.call_rettypevar = Some(y);
        }
        Inst::TailCall(f) => {
          let x = ctx.solver.fresh();
          let y = ctx.solver.fresh();
          ctx.solver.constrain_value_type(TypeVar(f), ValueTypeState::Fun(x, y));
          ctx.solver.constrain_tuple_type(x, ctx.block_outs.drain().collect());
          ctx.solver.unify(rettypevar, y);
        }
        Inst::Const(symbol) => {
          if let Some(&x) = ctx.solver_environment.get(symbol) {
            ctx.solver.unify(TypeVar(i), x);
          } else if let Some(t) = ctx.global_environment.get(symbol) {
            let x = ctx.solver.instantiate(t);
            ctx.solver.unify(TypeVar(i), x);
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

    ctx.solver_environment.clear();
    ctx.global_environment.insert(f.name, ctx.solver.generalize(funtypevar));
  }

  return (ctx.global_environment, ctx.solver);
}

// TODO: operator overloading

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
