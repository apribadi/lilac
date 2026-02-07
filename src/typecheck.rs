//!
//!
//! linearized code -> typed code

use crate::arr::Arr;
use crate::buf::Buf;
use crate::iru::Inst;
use crate::iru;
use crate::operator::Op1;
use crate::operator::Op2;
use crate::prim::PrimOp1;
use crate::prim::PrimOp2;
use crate::prim::PrimType::Bool;
use crate::prim::PrimType::I64;
use crate::prim::PrimType;
use crate::symbol::Symbol;
use crate::typeid::TypeId;
use crate::unionfind::UnionFind;
use std::iter::zip;
use tangerine::map::HashMap;

pub enum ValueType {
  Array(Box<ValueType>),
  Fun(TupleType, TupleType),
  Prim(PrimType),
  Var(TypeId),
}

pub enum TupleType {
  Tuple(Arr<ValueType>),
  Var(TypeId),
}

pub enum SeqType {
  Nil,
  Cons(Box<(ValueType, SeqType)>),
}

pub struct TypeScheme(/* arity */ pub u32, pub ValueType);

pub enum TypeState {
  Array(TypeId),
  Error,
  Fresh,
  Fun(TypeId, TypeId),
  Prim(PrimType),
  Tuple(Arr<TypeId>),
  Var(TypeId),
}

struct Ctx {
  global_environment: HashMap<Symbol, TypeScheme>,
  letrec_environment: HashMap<Symbol, TypeId>,
  solver: Solver,
  block_args: Buf<TypeId>,
  block_outs: Buf<TypeId>,
  block_call_ret: Option<TypeId>,
}

pub struct Solver {
  union_find: UnionFind<TypeState>,
  to_unify: Buf<(TypeId, TypeId)>,
}

impl Solver {
  fn new() -> Self {
    return Self { union_find: UnionFind::new(), to_unify: Buf::new() };
  }

  fn unify(&mut self, x: TypeId, y: TypeId) {
    self.to_unify.push((x, y));
  }

  fn fresh(&mut self) -> TypeId {
    return TypeId(self.union_find.push(TypeState::Fresh));
  }

  fn construct_array(&mut self, x: TypeId) -> TypeId {
    return TypeId(self.union_find.push(TypeState::Array(x)));
  }

  fn construct_fun(&mut self, x: TypeId, y: TypeId) -> TypeId {
    return TypeId(self.union_find.push(TypeState::Fun(x, y)));
  }

  fn construct_prim(&mut self, t: PrimType) -> TypeId {
    return TypeId(self.union_find.push(TypeState::Prim(t)));
  }

  fn construct_tuple(&mut self, t: Arr<TypeId>) -> TypeId {
    return TypeId(self.union_find.push(TypeState::Tuple(t)));
  }

  fn constrain_prim(&mut self, x: TypeId, t: PrimType) {
    match &mut self.union_find[x.0] {
      state @ &mut TypeState::Fresh => {
        *state = TypeState::Prim(t);
      }
      &mut TypeState::Prim(u) if u == t => {
      }
      state => {
        *state = TypeState::Error;
      }
    }
  }

  fn constrain_array(&mut self, x: TypeId, a: TypeId) {
    match &mut self.union_find[x.0] {
      state @ &mut TypeState::Fresh => {
        *state = TypeState::Array(a);
      }
      &mut TypeState::Array(b) => {
        self.to_unify.push((a, b));
      }
      state => {
        *state = TypeState::Error;
      }
    }
  }

  fn constrain_fun(&mut self, x: TypeId, a: TypeId, b: TypeId) {
    match &mut self.union_find[x.0] {
      state @ &mut TypeState::Fresh => {
        *state = TypeState::Fun(a, b);
      }
      &mut TypeState::Fun(c, d) => {
        self.to_unify.push((a, c));
        self.to_unify.push((b, d));
      }
      state => {
        *state = TypeState::Error;
      }
    }
  }

  fn constrain_tuple<'a, T>(&mut self, x: TypeId, t: T)
  where
    T: IntoIterator<IntoIter: ExactSizeIterator<Item = &'a TypeId>>
  {
    let t = t.into_iter();
    match &mut self.union_find[x.0] {
      state @ &mut TypeState::Fresh => {
        *state = TypeState::Tuple(Arr::from(t.copied()));
      }
      &mut TypeState::Tuple(ref u) if t.len() == u.len() as usize => {
        for (&a, &b) in zip(t, u) {
          self.to_unify.push((a, b));
        }
      }
      state => {
        *state = TypeState::Error
      }
    }
  }

  fn propagate(&mut self) {
    while let Some((x, y)) = self.to_unify.pop_checked() {
      match self.union_find.union(x.0, y.0) {
        (&mut _, None) => {
        }
        (state @ &mut TypeState::Fresh, Some(t)) => {
          *state = t;
        }
        (&mut _, Some(TypeState::Fresh)) => {
        }
        (&mut TypeState::Array(a), Some(TypeState::Array(b))) => {
          self.to_unify.push((a, b));
        }
        (&mut TypeState::Fun(a, b), Some(TypeState::Fun(c, d))) => {
          self.to_unify.push((a, c));
          self.to_unify.push((b, d));
        }
        (&mut TypeState::Prim(u), Some(TypeState::Prim(v))) if u == v => {
        }
        (&mut TypeState::Tuple(ref u), Some(TypeState::Tuple(ref v))) if u.len() == v.len() => {
          for (&a, &b) in zip(u, v) {
            self.to_unify.push((a, b));
          }
        }
        (state, _) => {
          *state = TypeState::Error;
        }
      }
    }
  }

  fn instantiate(&mut self, t: &TypeScheme) -> TypeId {
    let bound_type_vars = Arr::new(t.0, |_| self.fresh());
    return self.instantiate_value_type(&bound_type_vars, &t.1);
  }

  fn instantiate_value_type(&mut self, bound_type_vars: &Arr<TypeId>, t: &ValueType) -> TypeId {
    match t {
      &ValueType::Array(ref a) => {
        let a = self.instantiate_value_type(bound_type_vars, a);
        self.construct_array(a)
      }
      &ValueType::Fun(ref a, ref b) => {
        let a = self.instantiate_tuple_type(bound_type_vars, a);
        let b = self.instantiate_tuple_type(bound_type_vars, b);
        self.construct_fun(a, b)
      }
      &ValueType::Prim(t) => {
        self.construct_prim(t)
      }
      &ValueType::Var(x) => {
        bound_type_vars[x.0]
      }
    }
  }

  fn instantiate_tuple_type(&mut self, bound_type_vars: &Arr<TypeId>, t: &TupleType) -> TypeId {
    match t {
      &TupleType::Tuple(ref u) => {
        let u = Arr::from(u.iter().map(|a| self.instantiate_value_type(bound_type_vars, a)));
        self.construct_tuple(u)
      }
      &TupleType::Var(a) => {
        bound_type_vars[a.0]
      }
    }
  }

  fn generalize(&mut self, t: TypeId) -> Result<TypeScheme, ()> {
    // TODO: we actually need to generalize multiple typevars at the same time,
    // from a strongly-connected-component of top-level items

    // TODO: to handle recursive types, replace type state with a black-hole
    // when we reach it, and restore the old state after traversing descendant
    // types.

    let mut count = 0;
    let t = self.generalize_value_type(&mut count, t)?;
    return Ok(TypeScheme(count, t));
  }

  fn generalize_value_type(&mut self, count: &mut u32, t: TypeId) -> Result<ValueType, ()> {
    match &mut self.union_find[t.0] {
      state @ &mut TypeState::Fresh => {
        let a = TypeId(*count);
        *count += 1;
        *state = TypeState::Var(a);
        Ok(ValueType::Var(a))
      }
      &mut TypeState::Var(a) => {
        Ok(ValueType::Var(a))
      }
      &mut TypeState::Array(a) => {
        let a = self.generalize_value_type(count, a)?;
        Ok(ValueType::Array(Box::new(a)))
      }
      &mut TypeState::Fun(a, b) => {
        let a = self.generalize_tuple_type(count, a)?;
        let b = self.generalize_tuple_type(count, b)?;
        Ok(ValueType::Fun(a, b))
      }
      &mut TypeState::Prim(a) => {
        Ok(ValueType::Prim(a))
      }
      _ => {
        Err(())
      }
    }
  }

  fn generalize_tuple_type(&mut self, count: &mut u32, t: TypeId) -> Result<TupleType, ()> {
    match &mut self.union_find[t.0] {
      state @ &mut TypeState::Fresh => {
        let a = TypeId(*count);
        *count += 1;
        *state = TypeState::Var(a);
        Ok(TupleType::Var(a))
      }
      &mut TypeState::Var(a) => {
        Ok(TupleType::Var(a))
      }
      &mut TypeState::Tuple(ref u) => {
        let u = u.clone(); // ???
        let mut buf = Buf::new();
        for &a in &u { buf.push(self.generalize_value_type(count, a)?); }
        Ok(TupleType::Tuple(Arr::from(buf.drain())))
      }
      _ => {
        Err(())
      }
    }
  }

  pub fn resolve_value_type(&self, t: TypeId) -> Result<ValueType, ()> {
    match self.union_find[t.0] {
      TypeState::Var(a) =>
        Ok(ValueType::Var(a)),
      TypeState::Array(a) =>
        Ok(ValueType::Array(Box::new(self.resolve_value_type(a)?))),
      TypeState::Fun(a, b) =>
        Ok(ValueType::Fun(self.resolve_tuple_type(a)?, self.resolve_tuple_type(b)?)),
      TypeState::Prim(a) =>
        Ok(ValueType::Prim(a)),
      TypeState::Fresh =>
        Err(()),
      _ =>
        Err(()),
    }
  }

  pub fn resolve_tuple_type(&self, t: TypeId) -> Result<TupleType, ()> {
    match self.union_find[t.0] {
      TypeState::Var(a) =>
        Ok(TupleType::Var(a)),
      TypeState::Tuple(ref u) => {
        let mut buf = Buf::new();
        for &a in u { buf.push(self.resolve_value_type(a)?); }
        Ok(TupleType::Tuple(Arr::from(buf.drain())))
      }
      _ =>
        Err(())
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
        block_args: Buf::new(),
        block_outs: Buf::new(),
        block_call_ret: None,
      };

    ctx.global_environment.insert(
      Symbol::from_str("len"),
      TypeScheme(
        1,
        ValueType::Fun(
          TupleType::Tuple(Arr::from([ValueType::Var(TypeId(0))])),
          TupleType::Tuple(Arr::from([ValueType::Prim(I64)]))))
    );

    return ctx;
  }
}

pub fn typecheck(module: &iru::Module) -> (HashMap<Symbol, TypeScheme>, Solver) {
  let mut ctx = Ctx::new();

  // allocate a fresh type variable for each program point, starting from zero

  for _ in &module.code {
    let _: TypeId = ctx.solver.fresh();
  }

  // ?

  for f in &module.decl {
    // typecheck a function

    let rettypevar = ctx.solver.fresh();
    let funtypevar = ctx.solver.construct_fun(TypeId(f.pos), rettypevar);
    ctx.letrec_environment.insert(f.name, funtypevar);

    // apply initial type constraints

    for i in f.pos .. f.pos + f.len {
      match module.code[i] {
        Inst::ConstBool(_) => {
          ctx.solver.constrain_prim(TypeId(i), Bool);
        }
        Inst::ConstInt(_) => {
          ctx.solver.constrain_prim(TypeId(i), I64);
        }
        Inst::Local(x) => {
          ctx.solver.unify(TypeId(i), TypeId(x));
        }
        Inst::GetLocal(v) => {
          ctx.solver.unify(TypeId(i), TypeId(v));
        }
        Inst::SetLocal(v, x) => {
          ctx.solver.unify(TypeId(v), TypeId(x));
        }
        Inst::Index(x, y) => {
          ctx.solver.constrain_array(TypeId(x), TypeId(i));
          ctx.solver.constrain_prim(TypeId(y), I64);
        }
        Inst::SetIndex(x, y, z) => {
          ctx.solver.constrain_array(TypeId(x), TypeId(z));
          ctx.solver.constrain_prim(TypeId(y), I64);
        }
        Inst::Op1(f, x) => {
          let f = lower_op1(f);
          ctx.solver.constrain_prim(TypeId(x), f.arg_type());
          ctx.solver.constrain_prim(TypeId(i), f.out_type());
        }
        Inst::Op2(f, x, y) => {
          let f = lower_op2(f);
          ctx.solver.constrain_prim(TypeId(x), f.arg_type().0);
          ctx.solver.constrain_prim(TypeId(y), f.arg_type().1);
          ctx.solver.constrain_prim(TypeId(i), f.out_type());
        }
        Inst::Label(n) => {
          ctx.block_args.clear();
          ctx.block_outs.clear();
          ctx.block_call_ret = None;
          for _ in 0 .. n { ctx.block_args.push(ctx.solver.fresh()); }
          ctx.solver.constrain_tuple(TypeId(i), &ctx.block_args);
        }
        Inst::Get(k) => {
          ctx.solver.unify(TypeId(i), ctx.block_args[k]);
        }
        Inst::Put(i, x) => {
          assert!(ctx.block_outs.len() == i);
          ctx.block_outs.push(TypeId(x));
        }
        Inst::Ret => {
          ctx.solver.constrain_tuple(rettypevar, &ctx.block_outs);
        }
        Inst::Cond(x) => {
          ctx.solver.constrain_prim(TypeId(x), Bool);
        }
        Inst::Goto(a) => {
          match ctx.block_call_ret {
            None => {
              ctx.solver.constrain_tuple(TypeId(a), &ctx.block_outs);
            }
            Some(call_ret) => {
              ctx.solver.unify(TypeId(a), call_ret);
            }
          }
        }
        Inst::Call(f) => {
          let a = ctx.solver.fresh();
          let b = ctx.solver.fresh();
          ctx.solver.constrain_fun(TypeId(f), a, b);
          ctx.solver.constrain_tuple(a, &ctx.block_outs);
          ctx.block_call_ret = Some(b);
        }
        Inst::TailCall(f) => {
          let a = ctx.solver.fresh();
          let b = ctx.solver.fresh();
          ctx.solver.constrain_fun(TypeId(f), a, b);
          ctx.solver.constrain_tuple(a, &ctx.block_outs);
          ctx.solver.unify(rettypevar, b);
        }
        Inst::Const(symbol) => {
          if let Some(&t) = ctx.letrec_environment.get(symbol) {
            ctx.solver.unify(TypeId(i), t);
          } else if let Some(t) = ctx.global_environment.get(symbol) {
            let t = ctx.solver.instantiate(t);
            ctx.solver.unify(TypeId(i), t);
          } else {
            // TODO: error unbound variable
            unimplemented!()
          }
        }
        Inst::Field(..) | Inst::GotoStaticError | Inst::SetField(..) => {
          // TODO: unsupported instructions
          unimplemented!()
        }
      }
    }

    // solve all type constraints

    ctx.solver.propagate();

    // generalize

    // TODO: handle type error on failed generalization

    ctx.global_environment.insert(f.name, ctx.solver.generalize(funtypevar).unwrap());
    ctx.letrec_environment.clear();
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
      Self::Prim(a) =>
        write!(f, "{}", a)?,
      Self::Var(a) =>
        write!(f, "'{}", a.0)?,
    }
    return Ok(());
  }
}

impl std::fmt::Display for TupleType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match *self {
      Self::Var(a) =>
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
