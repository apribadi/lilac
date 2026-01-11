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

pub enum ValueType {
  Array(Box<ValueType>),
  Fun(TupleType, TupleType),
  PrimType(PrimType),
  TypeVar(TypeVar),
}

pub enum TupleType {
  TypeVar(TypeVar),
  Tuple(Arr<ValueType>),
}

pub struct TypeScheme(/* arity */ pub u32, pub ValueType);

pub enum ValueTypeState {
  Array(TypeVar),
  Fun(TypeVar, TypeVar),
  PrimType(PrimType),
}

type TupleTypeState = Arr<TypeVar>;

pub enum TypeState {
  Abstract,
  TupleType(TupleTypeState),
  TypeError,
  TypeVar(TypeVar),
  ValueType(ValueTypeState),
}

pub struct Solver {
  union_find: UnionFind<TypeState>,
  to_unify: Buf<(TypeVar, TypeVar)>,
}

struct Ctx {
  global_environment: HashMap<Symbol, TypeScheme>,
  letrec_environment: HashMap<Symbol, TypeVar>,
  solver: Solver,
  block_args: Arr<TypeVar>,
  block_outs: Buf<TypeVar>,
  call_rettypevar: Option<TypeVar>,
}

fn update_value_type_state(
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

fn update_tuple_type_state(
    x: TupleTypeState,
    y: TupleTypeState,
    to_unify: &mut Buf<(TypeVar, TypeVar)>
  ) -> TypeState
{
  if x.len() != y.len() {
    return TypeState::TypeError;
  }

  for (&x, &y) in zip(x.iter(), y.iter()) {
    to_unify.put((x, y));
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

  fn prim_type(&mut self, t: PrimType) -> TypeVar {
    return TypeVar(self.union_find.put(TypeState::ValueType(ValueTypeState::PrimType(t))));
  }

  fn unify(&mut self, x: TypeVar, y: TypeVar) {
    self.to_unify.put((x, y));
  }

  fn unify_value_type(&mut self, x: TypeVar, t: ValueTypeState) {
    self.to_unify.put((x, TypeVar(self.union_find.put(TypeState::ValueType(t)))));
  }

  fn unify_tuple_type(&mut self, x: TypeVar, t: TupleTypeState) {
    self.to_unify.put((x, TypeVar(self.union_find.put(TypeState::TupleType(t)))));
  }

  fn propagate(&mut self) {
    while let Some((x, y)) = self.to_unify.pop_if_nonempty() {
      if let (x, Some(y)) = self.union_find.union(x.0, y.0) {
        *x =
          match (replace(x, TypeState::Abstract), y) {
            (TypeState::Abstract, t) | (t, TypeState::Abstract) =>
              t,
            (TypeState::ValueType(x), TypeState::ValueType(y)) =>
              update_value_type_state(x, y, &mut self.to_unify),
            (TypeState::TupleType(x), TypeState::TupleType(y)) =>
              update_tuple_type_state(x, y, &mut self.to_unify),
            _ =>
              TypeState::TypeError,
          };
      }
    }
  }

  fn instantiate(&mut self, t: &TypeScheme) -> TypeVar {
    let bound_type_vars = (0 .. t.0).map(|_| self.fresh()).collect();
    return self.instantiate_value_type(&bound_type_vars, &t.1);
  }

  fn instantiate_value_type(&mut self, bound_type_vars: &Arr<TypeVar>, t: &ValueType) -> TypeVar {
    match *t {
      ValueType::Array(ref a) => {
        let t = self.fresh();
        let a = self.instantiate_value_type(bound_type_vars, a);
        self.unify_value_type(t, ValueTypeState::Array(a));
        t
      }
      ValueType::Fun(ref a, ref b) => {
        let t = self.fresh();
        let a = self.instantiate_tuple_type(bound_type_vars, a);
        let b = self.instantiate_tuple_type(bound_type_vars, b);
        self.unify_value_type(t, ValueTypeState::Fun(a, b));
        t
      }
      ValueType::PrimType(a) => {
        self.prim_type(a)
      }
      ValueType::TypeVar(x) => {
        bound_type_vars[x.0]
      }
    }
  }

  fn instantiate_tuple_type(&mut self, bound_type_vars: &Arr<TypeVar>, t: &TupleType) -> TypeVar {
    match *t {
      TupleType::TypeVar(a) => {
        bound_type_vars[a.0]
      }
      TupleType::Tuple(ref a) => {
        let t = self.fresh();
        let a = a.iter().map(|a| self.instantiate_value_type(bound_type_vars, a)).collect();
        self.unify_tuple_type(t, a);
        t
      }
    }
  }

  fn generalize(&mut self, t: TypeVar) -> TypeScheme {
    // TODO: we actually need to generalize multiple typevars at the same time,
    // from a strongly-connected-component of top-level items

    // TODO: handle recursive types

    let mut count = 0;
    let t = self.generalize_value_type(&mut count, t);
    return TypeScheme(count, t);
  }

  fn generalize_value_type(&mut self, count: &mut u32, t: TypeVar) -> ValueType {
    let t = &mut self.union_find[t.0];

    match *t {
      TypeState::Abstract => {
        let i = *count;
        *count = i + 1;
        let a = TypeVar(i);
        *t = TypeState::TypeVar(a);
        ValueType::TypeVar(a)
      }
      TypeState::TypeVar(a) => {
        ValueType::TypeVar(a)
      }
      TypeState::TupleType(..) => {
        panic!()
      }
      TypeState::TypeError => {
        panic!()
      }
      TypeState::ValueType(ValueTypeState::Array(a)) => {
        let a = self.generalize_value_type(count, a);
        ValueType::Array(Box::new(a))
      }
      TypeState::ValueType(ValueTypeState::Fun(a, b)) => {
        let a = self.generalize_tuple_type(count, a);
        let b = self.generalize_tuple_type(count, b);
        ValueType::Fun(a, b)
      }
      TypeState::ValueType(ValueTypeState::PrimType(a)) => {
        ValueType::PrimType(a)
      }
    }
  }

  fn generalize_tuple_type(&mut self, count: &mut u32, t: TypeVar) -> TupleType {
    let t = &mut self.union_find[t.0];

    match *t {
      TypeState::Abstract => {
        let i = *count;
        *count = i + 1;
        let a = TypeVar(i);
        *t = TypeState::TypeVar(a);
        TupleType::TypeVar(a)
      }
      TypeState::TypeVar(a) => {
        TupleType::TypeVar(a)
      }
      TypeState::TypeError => {
        panic!()
      }
      TypeState::ValueType(..) => {
        panic!()
      }
      TypeState::TupleType(ref a) => {
        let a = a.clone(); // ??!!
        TupleType::Tuple(a.iter().map(|a| self.generalize_value_type(count, *a)).collect())
      }
    }
  }

  pub fn resolve_value_type(&self, t: TypeVar) -> ValueType {
    match self.union_find[t.0] {
      TypeState::TypeVar(a) =>
        ValueType::TypeVar(a),
      TypeState::Abstract =>
        ValueType::TypeVar(TypeVar(111)), // ???
      TypeState::TupleType(..) | TypeState::TypeError =>
        ValueType::TypeVar(TypeVar(999)), // ???
      TypeState::ValueType(ValueTypeState::Array(a)) =>
        ValueType::Array(Box::new(self.resolve_value_type(a))),
      TypeState::ValueType(ValueTypeState::Fun(a, b)) =>
        ValueType::Fun(self.resolve_tuple_type(a), self.resolve_tuple_type(b)),
      TypeState::ValueType(ValueTypeState::PrimType(a)) =>
        ValueType::PrimType(a),
    }
  }

  pub fn resolve_tuple_type(&self, t: TypeVar) -> TupleType {
    match self.union_find[t.0] {
      TypeState::TypeVar(a) =>
        TupleType::TypeVar(a),
      TypeState::TupleType(ref t) =>
        TupleType::Tuple(t.iter().map(|t| self.resolve_value_type(*t)).collect()),
      _ =>
        panic!(),
    }
  }
}

impl Ctx {
  fn new() -> Self {
    let mut ctx =
      Self {
        global_environment: HashMap::new(),
        letrec_environment: HashMap::new(),
        solver: Solver::new(),
        block_args: Arr::EMPTY,
        block_outs: Buf::new(),
        call_rettypevar: None,
      };

    ctx.global_environment.insert(
      Symbol::from_str("len"),
      TypeScheme(
        1,
        ValueType::Fun(
          TupleType::Tuple(Arr::new([ValueType::TypeVar(TypeVar(0))])),
          TupleType::Tuple(Arr::new([ValueType::PrimType(PrimType::I64)]))))
    );

    return ctx;
  }
}

pub fn typecheck(module: &hir::Module) -> (HashMap<Symbol, TypeScheme>, Solver) {
  let mut ctx = Ctx::new();

  // allocate a type variable for each program point

  for _ in module.code.iter() {
    let _ = ctx.solver.fresh();
  }

  // ?

  for f in module.decl.iter() {
    // typecheck a function

    let funtypevar = ctx.solver.fresh();
    let argtypevar = ctx.solver.fresh();
    let rettypevar = ctx.solver.fresh();
    ctx.solver.unify_value_type(funtypevar, ValueTypeState::Fun(argtypevar, rettypevar));
    ctx.solver.unify(argtypevar, TypeVar(f.pos));
    ctx.letrec_environment.insert(f.name, funtypevar);

    // apply initial type constraints

    for i in f.pos .. f.pos + f.len {
      match module.code[i] {
        Inst::ConstBool(_) =>
          ctx.solver.unify_value_type(TypeVar(i), ValueTypeState::PrimType(PrimType::Bool)),
        Inst::ConstInt(_) =>
          ctx.solver.unify_value_type(TypeVar(i), ValueTypeState::PrimType(PrimType::I64)),
        Inst::Local(x) =>
          ctx.solver.unify(TypeVar(x), TypeVar(i)),
        Inst::GetLocal(v) =>
          ctx.solver.unify(TypeVar(v), TypeVar(i)),
        Inst::SetLocal(v, x) =>
          ctx.solver.unify(TypeVar(x), TypeVar(v)),
        Inst::Index(x, y) => {
          let a = ctx.solver.fresh();
          ctx.solver.unify_value_type(TypeVar(x), ValueTypeState::Array(a));
          ctx.solver.unify_value_type(TypeVar(y), ValueTypeState::PrimType(PrimType::I64));
          ctx.solver.unify(a, TypeVar(i));
        }
        Inst::SetIndex(x, y, z) => {
          let a = ctx.solver.fresh();
          ctx.solver.unify_value_type(TypeVar(x), ValueTypeState::Array(a));
          ctx.solver.unify_value_type(TypeVar(y), ValueTypeState::PrimType(PrimType::I64));
          ctx.solver.unify(TypeVar(z), a);
        }
        Inst::Op1(f, x) => {
          let f = lower_op1(f);
          ctx.solver.unify_value_type(TypeVar(x), ValueTypeState::PrimType(f.arg_type()));
          ctx.solver.unify_value_type(TypeVar(i), ValueTypeState::PrimType(f.out_type()));
        }
        Inst::Op2(f, x, y) => {
          let f = lower_op2(f);
          ctx.solver.unify_value_type(TypeVar(x), ValueTypeState::PrimType(f.arg_type().0));
          ctx.solver.unify_value_type(TypeVar(y), ValueTypeState::PrimType(f.arg_type().1));
          ctx.solver.unify_value_type(TypeVar(i), ValueTypeState::PrimType(f.out_type()));
        }
        Inst::Label(n) => {
          ctx.block_args = (0 .. n).map(|_| ctx.solver.fresh()).collect();
          ctx.block_outs.clear();
          ctx.call_rettypevar = None;
          ctx.solver.unify_tuple_type(TypeVar(i), ctx.block_args.clone());
        }
        Inst::Get(k) =>
          ctx.solver.unify(TypeVar(i), ctx.block_args[k]),
        Inst::Put(_, x) =>
          ctx.block_outs.put(TypeVar(x)),
        Inst::Ret =>
          ctx.solver.unify_tuple_type(rettypevar, ctx.block_outs.drain().collect()),
        Inst::Cond(x) =>
          ctx.solver.unify_value_type(TypeVar(x), ValueTypeState::PrimType(PrimType::Bool)),
        Inst::Goto(a) => {
          match ctx.call_rettypevar {
            None =>
              ctx.solver.unify_tuple_type(TypeVar(a), ctx.block_outs.iter().map(|x| *x).collect()),
            Some(call_ret) =>
              ctx.solver.unify(TypeVar(a), call_ret),
          }
        }
        Inst::Call(f) => {
          let x = ctx.solver.fresh();
          let y = ctx.solver.fresh();
          ctx.solver.unify_value_type(TypeVar(f), ValueTypeState::Fun(x, y));
          ctx.solver.unify_tuple_type(x, ctx.block_outs.drain().collect());
          ctx.call_rettypevar = Some(y);
        }
        Inst::TailCall(f) => {
          let x = ctx.solver.fresh();
          let y = ctx.solver.fresh();
          ctx.solver.unify_value_type(TypeVar(f), ValueTypeState::Fun(x, y));
          ctx.solver.unify_tuple_type(x, ctx.block_outs.drain().collect());
          ctx.solver.unify(rettypevar, y);
        }
        Inst::Const(symbol) => {
          if let Some(&t) = ctx.letrec_environment.get(symbol) {
            ctx.solver.unify(TypeVar(i), t);
          } else if let Some(t) = ctx.global_environment.get(symbol) {
            let t = ctx.solver.instantiate(t);
            ctx.solver.unify(TypeVar(i), t);
          } else {
            // TODO: error unbound variable
            unimplemented!()
          }
        }
        Inst::Field(..) | Inst::GotoStaticError | Inst::SetField(..) => {
          unimplemented!()
        }
      }
    }

    // solve all type constraints

    ctx.solver.propagate();

    // generalize

    ctx.letrec_environment.clear();
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

impl std::fmt::Display for ValueType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match *self {
      Self::Array(ref a) =>
        write!(f, "Array[{}]", a)?,
      Self::Fun(ref a, ref b) =>
        write!(f, "Fun{} -> {}", a, b)?,
      Self::PrimType(a) =>
        write!(f, "{}", a)?,
      Self::TypeVar(a) =>
        write!(f, "'{}", a.0)?,
    }
    return Ok(());
  }
}

impl std::fmt::Display for TupleType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match *self {
      Self::TypeVar(a) =>
        write!(f, "'{}", a.0)?,
      Self::Tuple(ref a) => {
        write!(f, "(")?;
        for (i, a) in a.iter().enumerate() {
          if i != 0 {
            write!(f, ", ")?;
          }
          write!(f, "{}", a)?;
        }
        write!(f, ")")?;
      }
    }
    return Ok(());
  }
}

impl std::fmt::Display for TypeScheme {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if self.0 != 0 {
      write!(f, "forall")?;
      for i in 0 .. self.0 { write!(f, " '{}", i)?; }
      write!(f, " . ")?;
    }
    return self.1.fmt(f);
  }
}
