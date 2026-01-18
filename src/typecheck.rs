//!
//!
//! linearized code -> typed code

use crate::arr::Arr;
use crate::buf::Buf;
use crate::operator::Op1;
use crate::operator::Op2;
use crate::prim::PrimOp1;
use crate::prim::PrimOp2;
use crate::prim::PrimType;
use crate::symbol::Symbol;
use crate::typevar::TypeVar;
use crate::uir::Inst;
use crate::uir;
use crate::union_find::UnionFind;
use std::iter::zip;
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

pub enum TypeState {
  ArrayType(TypeVar),
  Fresh,
  FunType(TypeVar, TypeVar),
  PrimType(PrimType),
  TupleType(Arr<TypeVar>),
  TypeError,
  TypeVar(TypeVar),
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

impl Solver {
  fn new() -> Self {
    return Self {
      union_find: UnionFind::new(),
      to_unify: Buf::new(),
    };
  }

  fn fresh(&mut self) -> TypeVar {
    return TypeVar(self.union_find.put(TypeState::Fresh));
  }

  fn prim_type(&mut self, t: PrimType) -> TypeVar {
    return TypeVar(self.union_find.put(TypeState::PrimType(t)));
  }

  fn array_type(&mut self, x: TypeVar) -> TypeVar {
    return TypeVar(self.union_find.put(TypeState::ArrayType(x)));
  }

  fn fun_type(&mut self, x: TypeVar, y: TypeVar) -> TypeVar {
    return TypeVar(self.union_find.put(TypeState::FunType(x, y)));
  }

  fn tuple_type(&mut self, t: Arr<TypeVar>) -> TypeVar {
    return TypeVar(self.union_find.put(TypeState::TupleType(t)));
  }

  fn unify(&mut self, x: TypeVar, y: TypeVar) {
    self.to_unify.put((x, y));
  }

  fn unify_prim_type(&mut self, x: TypeVar, t: PrimType) {
    match &mut self.union_find[x.0] {
      state @ &mut TypeState::Fresh => {
        *state = TypeState::PrimType(t);
      }
      &mut TypeState::PrimType(u) if u == t => {
      }
      state => {
        *state = TypeState::TypeError;
      }
    }
  }

  fn unify_array_type(&mut self, x: TypeVar, a: TypeVar) {
    match &mut self.union_find[x.0] {
      state @ &mut TypeState::Fresh => {
        *state = TypeState::ArrayType(a);
      }
      &mut TypeState::ArrayType(b) => {
        self.to_unify.put((a, b));
      }
      state => {
        *state = TypeState::TypeError;
      }
    }
  }

  fn unify_fun_type(&mut self, x: TypeVar, a: TypeVar, b: TypeVar) {
    match &mut self.union_find[x.0] {
      state @ &mut TypeState::Fresh => {
        *state = TypeState::FunType(a, b);
      }
      &mut TypeState::FunType(c, d) => {
        self.to_unify.put((a, c));
        self.to_unify.put((b, d));
      }
      state => {
        *state = TypeState::TypeError;
      }
    }
  }

  fn unify_tuple_type<'a, T>(&mut self, x: TypeVar, t: T)
  where
    T: IntoIterator,
    T::IntoIter: ExactSizeIterator<Item = &'a TypeVar>
  {
    let t = t.into_iter();
    match &mut self.union_find[x.0] {
      state @ &mut TypeState::Fresh => {
        *state = TypeState::TupleType(Arr::from(t.copied()));
      }
      &mut TypeState::TupleType(ref u) if t.len() == u.len() as usize => {
        for (&a, &b) in zip(t, u) {
          self.to_unify.put((a, b));
        }
      }
      state => {
        *state = TypeState::TypeError
      }
    }
  }

  fn propagate(&mut self) {
    while let Some((x, y)) = self.to_unify.pop_if_nonempty() {
      match self.union_find.union(x.0, y.0) {
        (_, None) => {
        }
        (state @ &mut TypeState::Fresh, Some(t)) => {
          *state = t;
        }
        (&mut _, Some(TypeState::Fresh)) => {
        }
        (&mut TypeState::ArrayType(a), Some(TypeState::ArrayType(b))) => {
          self.to_unify.put((a, b));
        }
        (&mut TypeState::FunType(a, b), Some(TypeState::FunType(c, d))) => {
          self.to_unify.put((a, c));
          self.to_unify.put((b, d));
        }
        (&mut TypeState::PrimType(u), Some(TypeState::PrimType(v))) if u == v => {
        }
        (&mut TypeState::TupleType(ref u), Some(TypeState::TupleType(ref v))) if u.len() == v.len() => {
          for (&a, &b) in zip(u, v) {
            self.to_unify.put((a, b));
          }
        }
        (state, _) => {
          *state = TypeState::TypeError;
        }
      }
    }
  }

  fn instantiate(&mut self, t: &TypeScheme) -> TypeVar {
    let bound_type_vars = Arr::new(t.0, |_| self.fresh());
    return self.instantiate_value_type(&bound_type_vars, &t.1);
  }

  fn instantiate_value_type(&mut self, bound_type_vars: &Arr<TypeVar>, t: &ValueType) -> TypeVar {
    match t {
      &ValueType::Array(ref a) => {
        let a = self.instantiate_value_type(bound_type_vars, a);
        self.array_type(a)
      }
      &ValueType::Fun(ref a, ref b) => {
        let a = self.instantiate_tuple_type(bound_type_vars, a);
        let b = self.instantiate_tuple_type(bound_type_vars, b);
        self.fun_type(a, b)
      }
      &ValueType::PrimType(t) => {
        self.prim_type(t)
      }
      &ValueType::TypeVar(x) => {
        bound_type_vars[x.0]
      }
    }
  }

  fn instantiate_tuple_type(&mut self, bound_type_vars: &Arr<TypeVar>, t: &TupleType) -> TypeVar {
    match t {
      &TupleType::Tuple(ref u) => {
        let u = Arr::from(u.iter().map(|a| self.instantiate_value_type(bound_type_vars, a)));
        self.tuple_type(u)
      }
      &TupleType::TypeVar(a) => {
        bound_type_vars[a.0]
      }
    }
  }

  fn generalize(&mut self, t: TypeVar) -> Result<TypeScheme, ()> {
    // TODO: we actually need to generalize multiple typevars at the same time,
    // from a strongly-connected-component of top-level items

    // TODO: to handle recursive types, replace type state with a black-hole
    // when we reach it, and restore the old state after traversing descendant
    // types.

    let mut count = 0;
    let t = self.generalize_value_type(&mut count, t)?;
    return Ok(TypeScheme(count, t));
  }

  fn generalize_value_type(&mut self, count: &mut u32, t: TypeVar) -> Result<ValueType, ()> {
    match &mut self.union_find[t.0] {
      state @ &mut TypeState::Fresh => {
        let a = TypeVar(*count);
        *count += 1;
        *state = TypeState::TypeVar(a);
        Ok(ValueType::TypeVar(a))
      }
      &mut TypeState::TypeVar(a) => {
        Ok(ValueType::TypeVar(a))
      }
      &mut TypeState::ArrayType(a) => {
        let a = self.generalize_value_type(count, a)?;
        Ok(ValueType::Array(Box::new(a)))
      }
      &mut TypeState::FunType(a, b) => {
        let a = self.generalize_tuple_type(count, a)?;
        let b = self.generalize_tuple_type(count, b)?;
        Ok(ValueType::Fun(a, b))
      }
      &mut TypeState::PrimType(a) => {
        Ok(ValueType::PrimType(a))
      }
      &mut TypeState::TupleType(..) | &mut TypeState::TypeError => {
        Err(())
      }
    }
  }

  fn generalize_tuple_type(&mut self, count: &mut u32, t: TypeVar) -> Result<TupleType, ()> {
    match &mut self.union_find[t.0] {
      state @ &mut TypeState::Fresh => {
        let a = TypeVar(*count);
        *count += 1;
        *state = TypeState::TypeVar(a);
        Ok(TupleType::TypeVar(a))
      }
      &mut TypeState::TypeVar(a) => {
        Ok(TupleType::TypeVar(a))
      }
      &mut TypeState::TupleType(ref a) => {
        let a = a.clone(); // ???
        let mut a = a.iter().map(|b| self.generalize_value_type(count, *b)).collect::<Result<Buf<_>, _>>()?;
        Ok(TupleType::Tuple(Arr::from(a.drain())))
      }
      _ => {
        Err(())
      }
    }
  }

  pub fn resolve_value_type(&self, t: TypeVar) -> Result<ValueType, ()> {
    match self.union_find[t.0] {
      TypeState::TypeVar(a) =>
        Ok(ValueType::TypeVar(a)),
      TypeState::ArrayType(a) =>
        Ok(ValueType::Array(Box::new(self.resolve_value_type(a)?))),
      TypeState::FunType(a, b) =>
        Ok(ValueType::Fun(self.resolve_tuple_type(a)?, self.resolve_tuple_type(b)?)),
      TypeState::PrimType(a) =>
        Ok(ValueType::PrimType(a)),
      TypeState::Fresh =>
        Err(()),
      TypeState::TupleType(..) | TypeState::TypeError =>
        Err(()),
    }
  }

  pub fn resolve_tuple_type(&self, t: TypeVar) -> Result<TupleType, ()> {
    match self.union_find[t.0] {
      TypeState::TypeVar(a) =>
        Ok(TupleType::TypeVar(a)),
      TypeState::TupleType(ref a) => {
        let mut a = a.iter().map(|b| self.resolve_value_type(*b)).collect::<Result<Buf<_>, _>>()?;
        Ok(TupleType::Tuple(Arr::from(a.drain())))
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
        block_args: Arr::EMPTY,
        block_outs: Buf::new(),
        call_rettypevar: None,
      };

    ctx.global_environment.insert(
      Symbol::from_str("len"),
      TypeScheme(
        1,
        ValueType::Fun(
          TupleType::Tuple(Arr::from([ValueType::TypeVar(TypeVar(0))])),
          TupleType::Tuple(Arr::from([ValueType::PrimType(PrimType::I64)]))))
    );

    return ctx;
  }
}

pub fn typecheck(module: &uir::Module) -> (HashMap<Symbol, TypeScheme>, Solver) {
  let mut ctx = Ctx::new();

  // allocate a fresh type variable for each program point, starting from zero

  for _ in &module.code {
    let _: TypeVar = ctx.solver.fresh();
  }

  // ?

  for f in &module.decl {
    // typecheck a function

    let rettypevar = ctx.solver.fresh();
    let funtypevar = ctx.solver.fun_type(TypeVar(f.pos), rettypevar);
    ctx.letrec_environment.insert(f.name, funtypevar);

    // apply initial type constraints

    for i in f.pos .. f.pos + f.len {
      match module.code[i] {
        Inst::ConstBool(_) => {
          ctx.solver.unify_prim_type(TypeVar(i), PrimType::Bool);
        }
        Inst::ConstInt(_) => {
          ctx.solver.unify_prim_type(TypeVar(i), PrimType::I64);
        }
        Inst::Local(x) => {
          ctx.solver.unify(TypeVar(i), TypeVar(x));
        }
        Inst::GetLocal(v) => {
          ctx.solver.unify(TypeVar(i), TypeVar(v));
        }
        Inst::SetLocal(v, x) => {
          ctx.solver.unify(TypeVar(v), TypeVar(x));
        }
        Inst::Index(x, y) => {
          ctx.solver.unify_array_type(TypeVar(x), TypeVar(i));
          ctx.solver.unify_prim_type(TypeVar(y), PrimType::I64);
        }
        Inst::SetIndex(x, y, z) => {
          ctx.solver.unify_array_type(TypeVar(x), TypeVar(z));
          ctx.solver.unify_prim_type(TypeVar(y), PrimType::I64);
        }
        Inst::Op1(f, x) => {
          let f = lower_op1(f);
          ctx.solver.unify_prim_type(TypeVar(x), f.arg_type());
          ctx.solver.unify_prim_type(TypeVar(i), f.out_type());
        }
        Inst::Op2(f, x, y) => {
          let f = lower_op2(f);
          ctx.solver.unify_prim_type(TypeVar(x), f.arg_type().0);
          ctx.solver.unify_prim_type(TypeVar(y), f.arg_type().1);
          ctx.solver.unify_prim_type(TypeVar(i), f.out_type());
        }
        Inst::Label(n) => {
          ctx.block_args = Arr::new(n, |_| ctx.solver.fresh());
          ctx.block_outs.clear();
          ctx.call_rettypevar = None;
          ctx.solver.unify_tuple_type(TypeVar(i), &ctx.block_args);
        }
        Inst::Get(k) => {
          ctx.solver.unify(TypeVar(i), ctx.block_args[k]);
        }
        Inst::Put(_, x) => {
          // TODO: check index
          ctx.block_outs.put(TypeVar(x));
        }
        Inst::Ret => {
          ctx.solver.unify_tuple_type(rettypevar, &ctx.block_outs);
        }
        Inst::Cond(x) => {
          ctx.solver.unify_prim_type(TypeVar(x), PrimType::Bool);
        }
        Inst::Goto(a) => {
          match ctx.call_rettypevar {
            None => {
              ctx.solver.unify_tuple_type(TypeVar(a), &ctx.block_outs);
            }
            Some(call_ret) => {
              ctx.solver.unify(TypeVar(a), call_ret);
            }
          }
        }
        Inst::Call(f) => {
          let a = ctx.solver.fresh();
          let b = ctx.solver.fresh();
          ctx.solver.unify_fun_type(TypeVar(f), a, b);
          ctx.solver.unify_tuple_type(a, &ctx.block_outs);
          ctx.call_rettypevar = Some(b);
        }
        Inst::TailCall(f) => {
          let a = ctx.solver.fresh();
          let b = ctx.solver.fresh();
          ctx.solver.unify_fun_type(TypeVar(f), a, b);
          ctx.solver.unify_tuple_type(a, &ctx.block_outs);
          ctx.solver.unify(rettypevar, b);
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
