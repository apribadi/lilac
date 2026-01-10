//!
//!
//! linearized code -> typed code

use crate::arr::Arr;
use crate::buf::Buf;
use crate::hir::Inst;
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
pub struct TypeVar(pub u32);

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
pub enum Type {
  Array(Box<Type>),
  Bool,
  Fun(Arr<Type>, Arr<Type>),
  I64,
  Var(TypeVar),
}

#[derive(Clone, Debug)]
pub struct TypeScheme(/* arity */ u32, Type);

#[derive(Clone, Debug)]
pub enum ValueType {
  Array(TypeVar),
  Fun(TypeVar, TypeVar),
  PrimType(PrimType),
}

type TupleType = Arr<TypeVar>;

pub enum TypeState {
  Abstract,
  TupleType(TupleType),
  TypeError,
  ValueType(ValueType),
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

  fn label_tuple(&self, i: u32) -> &Arr<TypeVar> {
    let InstType::Label(ref xs) = self.insts[i] else { unreachable!() };
    return xs;
  }

  fn label(&self, i: u32) -> TypeVar {
    let _ = self;
    return TypeVar(i)
  }

  fn local(&self, i: u32) -> TypeVar {
    let _ = self;
    return TypeVar(i)
  }

  fn value(&self, i: u32) -> TypeVar {
    let _ = self;
    return TypeVar(i)
  }

  pub fn insts(&self) -> impl Iterator<Item = &InstType> {
    return self.insts.iter();
  }
}

fn unify_value_type(x: ValueType, y: ValueType, todo: &mut Buf<(TypeVar, TypeVar)>) -> TypeState {
  match (x, y) {
    (ValueType::PrimType(x), ValueType::PrimType(y)) if x == y => {
      TypeState::ValueType(ValueType::PrimType(x))
    }
    (ValueType::Array(x), ValueType::Array(y)) => {
      todo.put((x, y));
      TypeState::ValueType(ValueType::Array(x))
    }
    (ValueType::Fun(a, b), ValueType::Fun(c, d)) => {
      todo.put((a, c));
      todo.put((b, d));
      TypeState::ValueType(ValueType::Fun(a, c))
    }
    (_, _) => {
      TypeState::TypeError
    }
  }
}

fn unify_tuple_type(xs: TupleType, ys: TupleType, todo: &mut Buf<(TypeVar, TypeVar)>) -> TypeState {
  if xs.len() != ys.len() {
    return TypeState::TypeError;
  }

  for (x, y) in zip(xs.iter(), ys.iter()) {
    todo.put((*x, *y));
  }

  return TypeState::TupleType(xs);
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

  fn bound_value_type(&mut self, x: TypeVar, t: ValueType) {
    let x = &mut self.vars[x.0];

    *x =
      match replace(x, TypeState::Abstract) {
        TypeState::TypeError | TypeState::TupleType(..) =>
          TypeState::TypeError,
        TypeState::Abstract =>
          TypeState::ValueType(t),
        TypeState::ValueType(x) =>
          unify_value_type(x, t, &mut self.todo),
      };
  }

  fn bound_tuple_type(&mut self, x: TypeVar, t: TupleType) {
    let x = &mut self.vars[x.0];

    *x =
      match replace(x, TypeState::Abstract) {
        TypeState::TypeError | TypeState::ValueType(..) =>
          TypeState::TypeError,
        TypeState::Abstract =>
          TypeState::TupleType(t),
        TypeState::TupleType(x) =>
          unify_tuple_type(x, t, &mut self.todo),
      };
  }

  fn unify(&mut self, x: TypeVar, y: TypeVar) {
    if let (x, Some(y)) = self.vars.union(x.0, y.0) {
      *x =
        match (replace(x, TypeState::Abstract), y) {
          (TypeState::Abstract, t) | (t, TypeState::Abstract) =>
            t,
          (TypeState::ValueType(x), TypeState::ValueType(y)) =>
            unify_value_type(x, y, &mut self.todo),
          (TypeState::TupleType(x), TypeState::TupleType(y)) =>
            unify_tuple_type(x, y, &mut self.todo),
          _ =>
            TypeState::TypeError,
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
        self.bound_value_type(a, ValueType::Array(b));
        return a;
      }
      Type::Bool => {
        let a = self.fresh();
        self.bound_value_type(a, ValueType::PrimType(PrimType::Bool));
        return a;
      }
      Type::Fun(ref x, ref y) => {
        let a = self.fresh();
        let b = self.fresh();
        let c = self.fresh();
        self.bound_value_type(a, ValueType::Fun(b, c));
        let x = Arr::new(x.iter().map(|x| self.instantiate_type(v, x)));
        self.bound_tuple_type(b, x);
        let y = Arr::new(y.iter().map(|y| self.instantiate_type(v, y)));
        self.bound_tuple_type(c, y);
        return a;
      }
      Type::I64 => {
        let a = self.fresh();
        self.bound_value_type(a, ValueType::PrimType(PrimType::Bool));
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
      TypeState::Abstract => hir::ValType::Abstract,
      TypeState::TypeError => hir::ValType::TypeError, // ???
      TypeState::TupleType(..) => hir::ValType::TypeError, // ???
      TypeState::ValueType(ref t) => {
        match *t {
          ValueType::Array(a) => hir::ValType::Array(Box::new(self.resolve(a))),
          ValueType::PrimType(PrimType::Bool) => hir::ValType::Bool,
          ValueType::PrimType(PrimType::I64) => hir::ValType::I64,
          ValueType::Fun(x, y) => hir::ValType::Fun(self.resolve_ret(x), self.resolve_ret(y)),
        }
      }
    }
  }

  pub fn resolve_ret(&self, x: TypeVar) -> Option<Arr<hir::ValType>> {
    // ???
    if let TypeState::TupleType(ref xs) = self.vars[x.0] {
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

pub fn typecheck(module: &hir::Module) -> (HashMap<Symbol, TypeScheme>, TypeSolver) {
  let mut ctx = Ctx::new();

  // assign type variables for all relevant program points

  for inst in module.code.iter() {
    let _ = ctx.solver.fresh();
    ctx.insts.put(InstType::Nil);
    /*
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
        ctx.insts.put(InstType::Value(i)),
      | Inst::Local(..) =>
        ctx.insts.put(InstType::Local(i)),
      | Inst::Label(_) => {
        ctx.insts.put(InstType::Nil);
      }
    }
    */
  }

  for f in module.decl.iter() {
    let funtypevar = ctx.solver.fresh();
    let rettypevar = ctx.solver.fresh();

    ctx.current_items.insert(f.name, funtypevar);

    // apply initial type constraints

    for i in f.pos .. f.pos + f.len {
      match module.code[i] {
        Inst::ConstBool(_) =>
          ctx.solver.bound_value_type(ctx.insts.value(i), ValueType::PrimType(PrimType::Bool)),
        Inst::ConstInt(_) =>
          ctx.solver.bound_value_type(ctx.insts.value(i), ValueType::PrimType(PrimType::I64)),
        Inst::Local(x) =>
          ctx.solver.unify(ctx.insts.value(x), ctx.insts.local(i)),
        Inst::GetLocal(v) =>
          ctx.solver.unify(ctx.insts.local(v), ctx.insts.value(i)),
        Inst::SetLocal(v, x) =>
          ctx.solver.unify(ctx.insts.value(x), ctx.insts.local(v)),
        Inst::Index(x, y) => {
          let a = ctx.solver.fresh();
          ctx.solver.bound_value_type(ctx.insts.value(x), ValueType::Array(a));
          ctx.solver.bound_value_type(ctx.insts.value(y), ValueType::PrimType(PrimType::I64));
          ctx.solver.unify(a, ctx.insts.value(i));
        }
        Inst::SetIndex(x, y, z) => {
          let a = ctx.solver.fresh();
          ctx.solver.bound_value_type(ctx.insts.value(x), ValueType::Array(a));
          ctx.solver.bound_value_type(ctx.insts.value(y), ValueType::PrimType(PrimType::I64));
          ctx.solver.unify(ctx.insts.value(z), a);
        }
        Inst::Op1(f, x) => {
          let f = lower_op1(f);
          let a = ValueType::PrimType(f.arg_type());
          let b = ValueType::PrimType(f.out_type());
          ctx.solver.bound_value_type(ctx.insts.value(x), a);
          ctx.solver.bound_value_type(ctx.insts.value(i), b);
        }
        Inst::Op2(f, x, y) => {
          let f = lower_op2(f);
          let a = ValueType::PrimType(f.arg_type().0);
          let b = ValueType::PrimType(f.arg_type().1);
          let c = ValueType::PrimType(f.out_type());
          ctx.solver.bound_value_type(ctx.insts.value(x), a);
          ctx.solver.bound_value_type(ctx.insts.value(y), b);
          ctx.solver.bound_value_type(ctx.insts.value(i), c);
        }
        Inst::Label(n) => {
          let a = Arr::new((0 .. n).map(|_| ctx.solver.fresh()));
          ctx.block = i;
          ctx.call_rettypevar = None;
          ctx.outs.clear();
          ctx.solver.bound_tuple_type(ctx.insts.label(i), a.clone());
          ctx.insts.insts[i] = InstType::Label(a);
        }
        Inst::Get(k) =>
          ctx.solver.unify(ctx.insts.value(i), ctx.insts.label_tuple(ctx.block)[k]),
        Inst::Put(_, x) =>
          ctx.outs.put(ctx.insts.value(x)),
        Inst::Ret =>
          ctx.solver.bound_tuple_type(rettypevar, ctx.outs.drain().collect()),
        Inst::Cond(x) =>
          ctx.solver.bound_value_type(ctx.insts.value(x), ValueType::PrimType(PrimType::Bool)),
        Inst::Goto(a) => {
          match ctx.call_rettypevar {
            None => {
              ctx.solver.bound_tuple_type(ctx.insts.label(a), ctx.outs.iter().map(|x| *x).collect());
            }
            Some(ret) => {
              ctx.solver.unify(ctx.insts.label(a), ret);
            }
          }
        }
        Inst::Call(f) => {
          let x = ctx.solver.fresh();
          let y = ctx.solver.fresh();
          ctx.solver.bound_value_type(ctx.insts.value(f), ValueType::Fun(x, y));
          ctx.solver.bound_tuple_type(x, ctx.outs.drain().collect());
          ctx.call_rettypevar = Some(y);
        }
        Inst::TailCall(f) => {
          let x = ctx.solver.fresh();
          let y = ctx.solver.fresh();
          ctx.solver.bound_value_type(ctx.insts.value(f), ValueType::Fun(x, y));
          ctx.solver.bound_tuple_type(x, ctx.outs.drain().collect());
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

    let argtypevar = ctx.insts.label(f.pos);
    ctx.solver.bound_value_type(funtypevar, ValueType::Fun(argtypevar, rettypevar));

    // solve all type constraints

    ctx.solver.propagate();

    // TODO: generalize

    ctx.current_items.clear();
    ctx.environment.insert(f.name, ctx.solver.generalize(funtypevar));
  }

  return (ctx.environment, ctx.solver);
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
