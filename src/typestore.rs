use crate::typeid::TypeId;

type Arity = u32;

pub struct TypeStore {
}

pub enum Type {
  Array(TypeId),
  Bool,
  Fun(TypeId, TypeId),
  I64,
  Tuple(Arity, TypeId),
  TupleElt(TypeId, /* next */ TypeId),
  Var(TypeId),
}
