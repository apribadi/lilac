use crate::ir1::Op1;
use crate::ir1::Op2;
use crate::symbol::Symbol;

#[derive(Clone, Copy)]
pub enum Item<'a> {
  Fundef(Fundef<'a>),
}

#[derive(Clone, Copy)]
pub struct Fundef<'a> {
  pub name: Symbol,
  pub args: &'a [Bind],
  pub body: &'a [Stmt<'a>],
}

#[derive(Clone, Copy)]
pub struct Bind {
  pub name: Option<Symbol>,
}

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
pub enum Stmt<'a> {
  ExprList(&'a [Expr<'a>]),
  Break(&'a [Expr<'a>]),
  Continue,
  Let(&'a [Bind], &'a [Expr<'a>]),
  Return(&'a [Expr<'a>]),
  Set(Symbol, Expr<'a>),
  SetField(Expr<'a>, Symbol, Expr<'a>),
  SetIndex(Expr<'a>, Expr<'a>, Expr<'a>),
  Var(Symbol, Expr<'a>),
  While(Expr<'a>, &'a [Stmt<'a>]),
}
