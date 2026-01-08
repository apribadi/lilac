// mid-level intermediate representation
//
// - bytecode
// - typed
// - polymorphic

use crate::arr::Arr;
use crate::prim::PrimOp1;
use crate::prim::PrimOp2;
use crate::prim::PrimType;
use crate::symbol::Symbol;

type Arity = u32;
type Index = u32;
type Label = u32;
type Local = u32;
type Value = u32;

type TupleType = u32;
type ValueType = u32;

pub struct Module {
  pub code: Arr<Inst>,
  pub decl: Arr<Fun>,
}

#[derive(Debug)]
pub struct Fun {
  pub name: Symbol,
  pub pos: u32,
  pub len: u32,
}

#[derive(Clone, Copy)]
pub enum Type {
  Array(ValueType),
  Fun(TupleType, TupleType),
  Multi(Arity),
  MultiElt(ValueType),
  PrimType(PrimType),
}

#[derive(Clone, Copy)]
pub enum Inst {
  GotoStaticError,
  Label(Arity),
  Get(Index, ValueType),
  Put(Index, Value),
  Goto(Label),
  Cond(Value),
  Ret,
  Call(Value),
  TailCall(Value),
  Const(Symbol, ValueType),
  ConstBool(bool),
  ConstInt(i64),
  PrimOp1(PrimOp1, Value),
  PrimOp2(PrimOp2, Value, Value),
  Local(Value),
  GetLocal(Local),
  SetLocal(Local, Value),
}
