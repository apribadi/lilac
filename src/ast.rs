use crate::operator::Op1;
use crate::operator::Op2;
use crate::symbol::Symbol;

pub enum Item<'a> {
  Fun(Fun<'a>),
}

pub struct Fun<'a> {
  pub name: Symbol,
  pub args: &'a [Binding],
  pub body: &'a [Stmt<'a>],
}

// TODO: add optional type ascription

pub struct Binding {
  pub name: Option<Symbol>,
}

// TODO: consider, e.g.,
//
// And(&'a Expr<'a>, &'a Expr<'a>)
//
// instead

pub enum Expr<'a> {
  And(&'a (Expr<'a>, Expr<'a>)),
  Bool(bool),
  Call(&'a (Expr<'a>, &'a [Expr<'a>])),
  Field(&'a (Expr<'a>, Symbol)),
  If(&'a (Expr<'a>, &'a [Stmt<'a>])),
  IfElse(&'a (Expr<'a>, &'a [Stmt<'a>], &'a [Stmt<'a>])),
  Index(&'a (Expr<'a>, Expr<'a>)),
  Int(i64),
  Loop(&'a [Stmt<'a>]),
  Op1(&'a (Op1, Expr<'a>)),
  Op2(&'a (Op2, Expr<'a>, Expr<'a>)),
  Or(&'a (Expr<'a>, Expr<'a>)),
  Ternary(&'a (Expr<'a>, Expr<'a>, Expr<'a>)),
  Undefined,
  Variable(Symbol),
}

pub enum Stmt<'a> {
  ExprList(&'a [Expr<'a>]),
  Break(&'a [Expr<'a>]),
  Continue,
  Let(&'a [Binding], &'a [Expr<'a>]),
  Return(&'a [Expr<'a>]),
  Set(Symbol, Expr<'a>),
  SetField(Expr<'a>, Symbol, Expr<'a>),
  SetIndex(Expr<'a>, Expr<'a>, Expr<'a>),
  Var(Symbol, Expr<'a>),
  While(Expr<'a>, &'a [Stmt<'a>]),
}
