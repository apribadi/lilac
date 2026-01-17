//!
//!
//! linearized code -> typed code

use crate::arr::Arr;
use crate::buf::Buf;
use crate::uir::Inst;
use crate::uir;
use crate::operator::Op1;
use crate::operator::Op2;
use crate::prim::PrimOp1;
use crate::prim::PrimOp2;
use crate::prim::PrimType;
use crate::symbol::Symbol;
use crate::typevar::TypeVar;
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

  fn propagate(&mut self) {
    while let Some((x, y)) = self.to_unify.pop_if_nonempty() {
      if let (x, Some(y)) = self.union_find.union(x.0, y.0) {
        match (&*x, y) {
          (&TypeState::Fresh, t) => {
            *x = t;
          }
          (&_, TypeState::Fresh) => {
          }
          (&TypeState::ArrayType(a), TypeState::ArrayType(b)) => {
            self.to_unify.put((a, b));
          }
          (&TypeState::FunType(a, b), TypeState::FunType(c, d)) => {
            self.to_unify.put((a, c));
            self.to_unify.put((b, d));
          }
          (&TypeState::PrimType(u), TypeState::PrimType(v)) if u == v => {
          }
          (&TypeState::TupleType(ref u), TypeState::TupleType(ref v)) if u.len() == v.len() => {
            for (&a, &b) in zip(u.iter(), v.iter()) {
              self.to_unify.put((a, b));
            }
          }
          _ => {
            *x = TypeState::TypeError;
          }
        }
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
        let a = self.instantiate_value_type(bound_type_vars, a);
        self.array_type(a)
      }
      ValueType::Fun(ref a, ref b) => {
        let a = self.instantiate_tuple_type(bound_type_vars, a);
        let b = self.instantiate_tuple_type(bound_type_vars, b);
        self.fun_type(a, b)
      }
      ValueType::PrimType(t) => {
        self.prim_type(t)
      }
      ValueType::TypeVar(x) => {
        bound_type_vars[x.0]
      }
    }
  }

  fn instantiate_tuple_type(&mut self, bound_type_vars: &Arr<TypeVar>, t: &TupleType) -> TypeVar {
    match *t {
      TupleType::Tuple(ref a) => {
        let a = a.iter().map(|a| self.instantiate_value_type(bound_type_vars, a)).collect();
        self.tuple_type(a)
      }
      TupleType::TypeVar(a) => {
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
    let t = &mut self.union_find[t.0];

    match *t {
      TypeState::Fresh => {
        let i = *count;
        *count = i + 1;
        let a = TypeVar(i);
        *t = TypeState::TypeVar(a);
        Ok(ValueType::TypeVar(a))
      }
      TypeState::TypeVar(a) => {
        Ok(ValueType::TypeVar(a))
      }
      TypeState::ArrayType(a) => {
        let a = self.generalize_value_type(count, a)?;
        Ok(ValueType::Array(Box::new(a)))
      }
      TypeState::FunType(a, b) => {
        let a = self.generalize_tuple_type(count, a)?;
        let b = self.generalize_tuple_type(count, b)?;
        Ok(ValueType::Fun(a, b))
      }
      TypeState::PrimType(a) => {
        Ok(ValueType::PrimType(a))
      }
      TypeState::TupleType(..) | TypeState::TypeError => {
        Err(())
      }
    }
  }

  fn generalize_tuple_type(&mut self, count: &mut u32, t: TypeVar) -> Result<TupleType, ()> {
    let t = &mut self.union_find[t.0];

    match *t {
      TypeState::Fresh => {
        let i = *count;
        *count = i + 1;
        let a = TypeVar(i);
        *t = TypeState::TypeVar(a);
        Ok(TupleType::TypeVar(a))
      }
      TypeState::TypeVar(a) => {
        Ok(TupleType::TypeVar(a))
      }
      TypeState::TupleType(ref a) => {
        let a = a.clone(); // ???
        let a = a.iter().map(|b| self.generalize_value_type(count, *b)).collect::<Result<_, _>>()?;
        Ok(TupleType::Tuple(a))
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
        let a = a.iter().map(|b| self.resolve_value_type(*b)).collect::<Result<_, _>>()?;
        Ok(TupleType::Tuple(a))
      }
      _ =>
        Err(())
    }
  }
}

fn unify(x: TypeVar, y: TypeVar, solver: &mut Solver) {
  solver.unify(x, y);
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

pub fn typecheck(module: &uir::Module) -> (HashMap<Symbol, TypeScheme>, Solver) {
  let mut ctx = Ctx::new();

  // allocate a fresh type variable for each program point, starting from zero

  for _ in module.code.iter() {
    let _: TypeVar = ctx.solver.fresh();
  }

  // ?

  for f in module.decl.iter() {
    // typecheck a function

    let rettypevar = ctx.solver.fresh();
    let funtypevar = ctx.solver.fun_type(TypeVar(f.pos), rettypevar);
    ctx.letrec_environment.insert(f.name, funtypevar);

    // apply initial type constraints

    for i in f.pos .. f.pos + f.len {
      match module.code[i] {
        Inst::ConstBool(_) => {
          let a = ctx.solver.prim_type(PrimType::Bool);
          ctx.solver.unify(TypeVar(i), a);
        }
        Inst::ConstInt(_) => {
          let a = ctx.solver.prim_type(PrimType::I64);
          ctx.solver.unify(TypeVar(i), a);
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
          let a = ctx.solver.array_type(TypeVar(i));
          let b = ctx.solver.prim_type(PrimType::I64);
          ctx.solver.unify(TypeVar(x), a);
          ctx.solver.unify(TypeVar(y), b);
        }
        Inst::SetIndex(x, y, z) => {
          let a = ctx.solver.array_type(TypeVar(z));
          let b = ctx.solver.prim_type(PrimType::I64);
          ctx.solver.unify(TypeVar(x), a);
          ctx.solver.unify(TypeVar(y), b);
        }
        Inst::Op1(f, x) => {
          let f = lower_op1(f);
          let a = ctx.solver.prim_type(f.arg_type());
          let b = ctx.solver.prim_type(f.out_type());
          ctx.solver.unify(TypeVar(x), a);
          ctx.solver.unify(TypeVar(i), b);
        }
        Inst::Op2(f, x, y) => {
          let f = lower_op2(f);
          let a = ctx.solver.prim_type(f.arg_type().0);
          let b = ctx.solver.prim_type(f.arg_type().1);
          let c = ctx.solver.prim_type(f.out_type());
          ctx.solver.unify(TypeVar(x), a);
          ctx.solver.unify(TypeVar(y), b);
          ctx.solver.unify(TypeVar(i), c);
        }
        Inst::Label(n) => {
          ctx.block_args = (0 .. n).map(|_| ctx.solver.fresh()).collect();
          ctx.block_outs.clear();
          ctx.call_rettypevar = None;
          let a = ctx.solver.tuple_type(ctx.block_args.clone());
          ctx.solver.unify(TypeVar(i), a);
        }
        Inst::Get(k) => {
          ctx.solver.unify(TypeVar(i), ctx.block_args[k]);
        }
        Inst::Put(_, x) => {
          // TODO: check index
          ctx.block_outs.put(TypeVar(x));
        }
        Inst::Ret => {
          unify(rettypevar, ctx.solver.tuple_type(ctx.block_outs.drain().collect()), &mut ctx.solver);
        }
        Inst::Cond(x) => {
          unify(TypeVar(x), ctx.solver.prim_type(PrimType::Bool), &mut ctx.solver);
        }
        Inst::Goto(a) => {
          match ctx.call_rettypevar {
            None =>
              unify(TypeVar(a), ctx.solver.tuple_type(ctx.block_outs.iter().map(|x| *x).collect()), &mut ctx.solver),
            Some(call_ret) =>
              unify(TypeVar(a), call_ret, &mut ctx.solver),
          }
        }
        Inst::Call(f) => {
          let a = ctx.solver.fresh();
          let b = ctx.solver.fresh();
          unify(TypeVar(f), ctx.solver.fun_type(a, b), &mut ctx.solver);
          unify(a, ctx.solver.tuple_type(ctx.block_outs.drain().collect()), &mut ctx.solver);
          ctx.call_rettypevar = Some(b);
        }
        Inst::TailCall(f) => {
          let a = ctx.solver.fresh();
          let b = ctx.solver.fresh();
          unify(TypeVar(f), ctx.solver.fun_type(a, b), &mut ctx.solver);
          unify(a, ctx.solver.tuple_type(ctx.block_outs.drain().collect()), &mut ctx.solver);
          unify(rettypevar, b, &mut ctx.solver);
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
