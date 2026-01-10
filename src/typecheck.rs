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
  Fun(TupleType, TupleType),
  I64,
  Bool,
  TypeVar(TypeVar),
}

#[derive(Clone, Debug)]
pub enum TupleType {
  TypeVar(TypeVar),
  Tuple(Arr<ValueType>),
}

#[derive(Clone, Debug)]
pub struct TypeScheme(/* arity */ pub u32, pub ValueType);

#[derive(Clone, Debug)]
pub enum ValueTypeNode {
  Array(TypeVar),
  Fun(TypeVar, TypeVar),
  PrimType(PrimType),
}

type TupleTypeNode = Arr<TypeVar>;

#[derive(Debug)]
pub enum TypeNode {
  Abstract,
  TypeVar(TypeVar),
  TupleType(TupleTypeNode),
  TypeError,
  ValueType(ValueTypeNode),
}

pub struct Solver {
  union_find: UnionFind<TypeNode>,
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
    x: ValueTypeNode,
    y: ValueTypeNode,
    to_unify: &mut Buf<(TypeVar, TypeVar)>
  ) -> TypeNode
{
  match (x, y) {
    (ValueTypeNode::Array(a), ValueTypeNode::Array(b)) => {
      to_unify.put((a, b));
      TypeNode::ValueType(ValueTypeNode::Array(a))
    }
    (ValueTypeNode::Fun(a, b), ValueTypeNode::Fun(c, d)) => {
      to_unify.put((a, c));
      to_unify.put((b, d));
      TypeNode::ValueType(ValueTypeNode::Fun(a, b))
    }
    (ValueTypeNode::PrimType(x), ValueTypeNode::PrimType(y)) if x == y => {
      TypeNode::ValueType(ValueTypeNode::PrimType(x))
    }
    (_, _) => {
      TypeNode::TypeError
    }
  }
}

fn unify_tuple_type(
    x: TupleTypeNode,
    y: TupleTypeNode,
    to_unify: &mut Buf<(TypeVar, TypeVar)>
  ) -> TypeNode
{
  if x.len() != y.len() {
    return TypeNode::TypeError;
  }

  for (x, y) in zip(x.iter(), y.iter()) {
    to_unify.put((*x, *y));
  }

  return TypeNode::TupleType(x);
}

impl Solver {
  fn new() -> Self {
    return Self {
      union_find: UnionFind::new(),
      to_unify: Buf::new(),
    };
  }

  fn fresh(&mut self) -> TypeVar {
    return TypeVar(self.union_find.put(TypeNode::Abstract));
  }

  fn value_type(&mut self, x: TypeVar, y: ValueTypeNode) {
    let x = &mut self.union_find[x.0];

    *x =
      match replace(x, TypeNode::Abstract) {
        TypeNode::Abstract =>
          TypeNode::ValueType(y),
        TypeNode::ValueType(x) =>
          unify_value_type(x, y, &mut self.to_unify),
        _ =>
          TypeNode::TypeError,
      };
  }

  fn tuple_type(&mut self, x: TypeVar, y: TupleTypeNode) {
    let x = &mut self.union_find[x.0];

    *x =
      match replace(x, TypeNode::Abstract) {
        TypeNode::Abstract =>
          TypeNode::TupleType(y),
        TypeNode::TupleType(x) =>
          unify_tuple_type(x, y, &mut self.to_unify),
          _ =>
          TypeNode::TypeError,
      };
  }

  fn unify(&mut self, x: TypeVar, y: TypeVar) {
    if let (x, Some(y)) = self.union_find.union(x.0, y.0) {
      *x =
        match (replace(x, TypeNode::Abstract), y) {
          (TypeNode::Abstract, t) | (t, TypeNode::Abstract) =>
            t,
          (TypeNode::ValueType(x), TypeNode::ValueType(y)) =>
            unify_value_type(x, y, &mut self.to_unify),
          (TypeNode::TupleType(x), TypeNode::TupleType(y)) =>
            unify_tuple_type(x, y, &mut self.to_unify),
          _ =>
            TypeNode::TypeError,
        };
    }
  }

  fn propagate(&mut self) {
    while let Some((x, y)) = self.to_unify.pop_if_nonempty() {
      self.unify(x, y);
    }
  }

  fn instantiate(&mut self, t: &TypeScheme) -> TypeVar {
    let n = t.0;
    let t = &t.1;
    let bound_type_vars = Arr::new((0 .. n).map(|_| self.fresh()));
    return self.instantiate_value_type(&bound_type_vars, t);
  }

  fn instantiate_value_type(&mut self, bound_type_vars: &Arr<TypeVar>, t: &ValueType) -> TypeVar {
    match *t {
      ValueType::Array(ref a) => {
        let t = self.fresh();
        let a = self.instantiate_value_type(bound_type_vars, a);
        self.value_type(t, ValueTypeNode::Array(a));
        t
      }
      ValueType::Fun(ref a, ref b) => {
        let t = self.fresh();
        let a = self.instantiate_tuple_type(bound_type_vars, a);
        let b = self.instantiate_tuple_type(bound_type_vars, b);
        self.value_type(t, ValueTypeNode::Fun(a, b));
        t
      }
      ValueType::I64 => {
        let t = self.fresh();
        self.value_type(t, ValueTypeNode::PrimType(PrimType::I64));
        t
      }
      ValueType::Bool => {
        let t = self.fresh();
        self.value_type(t, ValueTypeNode::PrimType(PrimType::Bool));
        t
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
        let a = Arr::new(a.iter().map(|a| self.instantiate_value_type(bound_type_vars, a)));
        self.tuple_type(t, a);
        t
      }
    }
  }

  fn generalize(&mut self, t: TypeVar) -> TypeScheme {
    // TODO: we actually need to generalize multiple typevars at the same time,
    // from a strongly-connected-component of top-level items

    let mut count = 0;
    let t = self.generalize_value_type(&mut count, t);
    return TypeScheme(count, t);
  }

  fn generalize_value_type(&mut self, count: &mut u32, t: TypeVar) -> ValueType {
    let t = &mut self.union_find[t.0];

    match *t {
      TypeNode::Abstract => {
        let i = *count;
        *count = i + 1;
        let a = TypeVar(i);
        *t = TypeNode::TypeVar(a);
        ValueType::TypeVar(a)
      }
      TypeNode::TypeVar(a) => {
        ValueType::TypeVar(a)
      }
      TypeNode::TupleType(..) => {
        panic!()
      }
      TypeNode::TypeError => {
        panic!()
      }
      TypeNode::ValueType(ValueTypeNode::Array(a)) => {
        let a = self.generalize_value_type(count, a);
        ValueType::Array(Box::new(a))
      }
      TypeNode::ValueType(ValueTypeNode::Fun(a, b)) => {
        let a = self.generalize_tuple_type(count, a);
        let b = self.generalize_tuple_type(count, b);
        ValueType::Fun(a, b)
      }
      TypeNode::ValueType(ValueTypeNode::PrimType(PrimType::Bool)) => {
        ValueType::Bool
      }
      TypeNode::ValueType(ValueTypeNode::PrimType(PrimType::I64)) => {
        ValueType::I64
      }
    }
  }

  fn generalize_tuple_type(&mut self, count: &mut u32, t: TypeVar) -> TupleType {
    let t = &mut self.union_find[t.0];

    match *t {
      TypeNode::Abstract => {
        let i = *count;
        *count = i + 1;
        let a = TypeVar(i);
        *t = TypeNode::TypeVar(a);
        TupleType::TypeVar(a)
      }
      TypeNode::TypeVar(a) => {
        TupleType::TypeVar(a)
      }
      TypeNode::TypeError => {
        panic!()
      }
      TypeNode::ValueType(..) => {
        panic!()
      }
      TypeNode::TupleType(ref a) => {
        let a = a.clone(); // ??!!
        TupleType::Tuple(Arr::new(a.iter().map(|a| self.generalize_value_type(count, *a))))
      }
    }
  }

  pub fn resolve_value_type(&self, t: TypeVar) -> ValueType {
    match self.union_find[t.0] {
      TypeNode::TypeVar(a) =>
        ValueType::TypeVar(a),
      TypeNode::Abstract =>
        ValueType::TypeVar(TypeVar(111)), // ???
      TypeNode::TupleType(..) | TypeNode::TypeError =>
        ValueType::TypeVar(TypeVar(999)), // ???
      TypeNode::ValueType(ValueTypeNode::Array(a)) =>
        ValueType::Array(Box::new(self.resolve_value_type(a))),
      TypeNode::ValueType(ValueTypeNode::Fun(a, b)) =>
        ValueType::Fun(self.resolve_tuple_type(a), self.resolve_tuple_type(b)),
      TypeNode::ValueType(ValueTypeNode::PrimType(PrimType::Bool)) =>
        ValueType::Bool,
      TypeNode::ValueType(ValueTypeNode::PrimType(PrimType::I64)) =>
        ValueType::I64,
    }
  }

  pub fn resolve_tuple_type(&self, t: TypeVar) -> TupleType {
    match self.union_find[t.0] {
      TypeNode::TypeVar(a) =>
        TupleType::TypeVar(a),
      TypeNode::TupleType(ref t) =>
        TupleType::Tuple(Arr::new(t.iter().map(|t| self.resolve_value_type(*t)))),
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
        solver_environment: HashMap::new(),
        solver: Solver::new(),
        block: u32::MAX,
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
          TupleType::Tuple(Arr::new([ValueType::I64]))))
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
    ctx.solver.value_type(funtypevar, ValueTypeNode::Fun(argtypevar, rettypevar));
    ctx.solver.unify(argtypevar, TypeVar(f.pos));
    ctx.solver_environment.insert(f.name, funtypevar);

    // apply initial type constraints

    for i in f.pos .. f.pos + f.len {
      match module.code[i] {
        Inst::ConstBool(_) =>
          ctx.solver.value_type(TypeVar(i), ValueTypeNode::PrimType(PrimType::Bool)),
        Inst::ConstInt(_) =>
          ctx.solver.value_type(TypeVar(i), ValueTypeNode::PrimType(PrimType::I64)),
        Inst::Local(x) =>
          ctx.solver.unify(TypeVar(x), TypeVar(i)),
        Inst::GetLocal(v) =>
          ctx.solver.unify(TypeVar(v), TypeVar(i)),
        Inst::SetLocal(v, x) =>
          ctx.solver.unify(TypeVar(x), TypeVar(v)),
        Inst::Index(x, y) => {
          let a = ctx.solver.fresh();
          ctx.solver.value_type(TypeVar(x), ValueTypeNode::Array(a));
          ctx.solver.value_type(TypeVar(y), ValueTypeNode::PrimType(PrimType::I64));
          ctx.solver.unify(a, TypeVar(i));
        }
        Inst::SetIndex(x, y, z) => {
          let a = ctx.solver.fresh();
          ctx.solver.value_type(TypeVar(x), ValueTypeNode::Array(a));
          ctx.solver.value_type(TypeVar(y), ValueTypeNode::PrimType(PrimType::I64));
          ctx.solver.unify(TypeVar(z), a);
        }
        Inst::Op1(f, x) => {
          let f = lower_op1(f);
          let a = ValueTypeNode::PrimType(f.arg_type());
          let b = ValueTypeNode::PrimType(f.out_type());
          ctx.solver.value_type(TypeVar(x), a);
          ctx.solver.value_type(TypeVar(i), b);
        }
        Inst::Op2(f, x, y) => {
          let f = lower_op2(f);
          let a = ValueTypeNode::PrimType(f.arg_type().0);
          let b = ValueTypeNode::PrimType(f.arg_type().1);
          let c = ValueTypeNode::PrimType(f.out_type());
          ctx.solver.value_type(TypeVar(x), a);
          ctx.solver.value_type(TypeVar(y), b);
          ctx.solver.value_type(TypeVar(i), c);
        }
        Inst::Label(n) => {
          let a = Arr::new((0 .. n).map(|_| ctx.solver.fresh()));
          ctx.solver.tuple_type(TypeVar(i), a.clone());
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
          ctx.solver.tuple_type(rettypevar, ctx.block_outs.drain().collect()),
        Inst::Cond(x) =>
          ctx.solver.value_type(TypeVar(x), ValueTypeNode::PrimType(PrimType::Bool)),
        Inst::Goto(a) => {
          match ctx.call_rettypevar {
            None =>
              ctx.solver.tuple_type(TypeVar(a), ctx.block_outs.iter().map(|x| *x).collect()),
            Some(ret) =>
              ctx.solver.unify(TypeVar(a), ret),
          }
        }
        Inst::Call(f) => {
          let x = ctx.solver.fresh();
          let y = ctx.solver.fresh();
          ctx.solver.value_type(TypeVar(f), ValueTypeNode::Fun(x, y));
          ctx.solver.tuple_type(x, ctx.block_outs.drain().collect());
          ctx.call_rettypevar = Some(y);
        }
        Inst::TailCall(f) => {
          let x = ctx.solver.fresh();
          let y = ctx.solver.fresh();
          ctx.solver.value_type(TypeVar(f), ValueTypeNode::Fun(x, y));
          ctx.solver.tuple_type(x, ctx.block_outs.drain().collect());
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
            panic!()
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

    // generalize

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
