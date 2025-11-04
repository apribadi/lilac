use crate::lexer::Lexer;
use crate::ir1::Op1;
use crate::ir1::Op2;
use crate::parse;
use crate::symbol::Symbol;
use crate::token::Token;
use oxcart::Arena;

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

pub fn parse<'a>(source: &[u8], arena: &mut Arena<'a>) -> Vec<Item<'a>> {
  let mut e = ToAst::new(arena);
  parse::parse(&mut Lexer::new(source), &mut e);
  return e.items;
}

struct ToAst<'a, 'b> {
  arena: &'b mut Arena<'a>,
  items: Vec<Item<'a>>,
  binds: Vec<Bind>,
  exprs: Vec<Expr<'a>>,
  stmts: Vec<Stmt<'a>>,
}

impl<'a, 'b> ToAst<'a, 'b> {
  fn new(arena: &'b mut Arena<'a>) -> Self {
    Self {
      arena,
      items: Vec::new(),
      binds: Vec::new(),
      exprs: Vec::new(),
      stmts: Vec::new(),
    }
  }

  fn alloc<T>(&mut self, x: T) -> &'a T {
    return self.arena.alloc().init(x);
  }

  fn put_item(&mut self, x: Item<'a>) {
    self.items.push(x);
  }

  fn put_bind(&mut self, x: Bind) {
    self.binds.push(x);
  }

  fn pop_bind_list(&mut self, n: usize) -> &'a [Bind] {
    let x = self.binds.drain(self.binds.len() - n ..);
    return self.arena.slice_from_iter(x);
  }

  fn put_expr(&mut self, x: Expr<'a>) {
    self.exprs.push(x);
  }

  fn pop_expr(&mut self) -> Expr<'a> {
    return self.exprs.pop().unwrap();
  }

  fn pop_expr_list(&mut self, n: usize) -> &'a [Expr<'a>] {
    let x = self.exprs.drain(self.exprs.len() - n ..);
    return self.arena.slice_from_iter(x);
  }

  fn put_stmt(&mut self, x: Stmt<'a>) {
    self.stmts.push(x);
  }

  fn pop_stmt_list(&mut self, n: usize) -> &'a [Stmt<'a>] {
    let x = self.stmts.drain(self.stmts.len() - n ..);
    return self.arena.slice_from_iter(x);
  }
}

impl<'a, 'b> parse::Out for ToAst<'a, 'b> {
  fn on_fundef(&mut self, name: &[u8], n_args: usize, n_stmts: usize) {
    let z = self.pop_stmt_list(n_stmts);
    let y = self.pop_bind_list(n_args);
    let x = Symbol::from_bytes(name);
    let x = Item::Fundef(Fundef { name: x, args: y, body: z });
    self.put_item(x);
  }

  fn on_bind(&mut self, name: Option<&[u8]>) {
    let x = Bind { name: name.map(Symbol::from_bytes) };
    self.put_bind(x);
  }

  fn on_variable(&mut self, symbol: &[u8]) {
    let s = Symbol::from_bytes(symbol);
    self.put_expr(Expr::Variable(s));
  }

  fn on_bool(&mut self, x: bool) {
    self.put_expr(Expr::Bool(x));
  }

  fn on_number(&mut self, x: &[u8]) {
    let n =
      match i64::from_str_radix(str::from_utf8(x).unwrap(), 10) {
        Err(_) => {
          self.put_expr(Expr::Undefined);
          return;
        }
        Ok(n) => n
      };
    self.put_expr(Expr::Int(n));
  }

  fn on_ternary(&mut self) {
    let y = self.pop_expr();
    let x = self.pop_expr();
    let p = self.pop_expr();
    let x = Expr::Ternary(self.alloc((p, x, y)));
    self.put_expr(x);
  }

  fn on_or(&mut self) {
    let y = self.pop_expr();
    let x = self.pop_expr();
    let x = Expr::Or(self.alloc((x, y)));
    self.put_expr(x);
  }

  fn on_and(&mut self) {
    let y = self.pop_expr();
    let x = self.pop_expr();
    let x = Expr::And(self.alloc((x, y)));
    self.put_expr(x);
  }

  fn on_op1(&mut self, op: Op1) {
    let x = self.pop_expr();
    let x = Expr::Op1(self.alloc((op, x)));
    self.put_expr(x);
  }

  fn on_op2(&mut self, op: Op2) {
    let y = self.pop_expr();
    let x = self.pop_expr();
    let x = Expr::Op2(self.alloc((op, x, y)));
    self.put_expr(x);
  }

  fn on_field(&mut self, symbol: &[u8]) {
    let s = Symbol::from_bytes(symbol);
    let x = self.pop_expr();
    let x = Expr::Field(self.alloc((x, s)));
    self.put_expr(x);
  }

  fn on_index(&mut self) {
    let y = self.pop_expr();
    let x = self.pop_expr();
    let x = Expr::Index(self.alloc((x, y)));
    self.put_expr(x);
  }

  fn on_if(&mut self, n_stmts: usize) {
    let y = self.pop_stmt_list(n_stmts);
    let x = self.pop_expr();
    let x = Expr::If(self.alloc((x, y)));
    self.put_expr(x);
  }

  fn on_if_else(&mut self, n_stmts_then: usize, n_stmts_else: usize) {
    let z = self.pop_stmt_list(n_stmts_else);
    let y = self.pop_stmt_list(n_stmts_then);
    let x = self.pop_expr();
    let x = Expr::IfElse(self.alloc((x, y, z)));
    self.put_expr(x);
  }

  fn on_call(&mut self, arity: usize) {
    let x = self.pop_expr_list(arity);
    let f = self.pop_expr();
    let x = Expr::Call(self.alloc((f, x)));
    self.put_expr(x);
  }

  fn on_loop(&mut self, n_stmts: usize) {
    let x = self.pop_stmt_list(n_stmts);
    self.put_expr(Expr::Loop(x));
  }

  fn on_stmt_expr_list(&mut self, n_exprs: usize) {
    let x = self.pop_expr_list(n_exprs);
    self.put_stmt(Stmt::ExprList(x));
  }

  fn on_break(&mut self, arity: usize) {
    let x = self.pop_expr_list(arity);
    self.put_stmt(Stmt::Break(x));
  }

  fn on_continue(&mut self) {
    self.put_stmt(Stmt::Continue);
  }

  fn on_let(&mut self, n_binds: usize, n_exprs: usize) {
    let y = self.pop_expr_list(n_exprs);
    let x = self.pop_bind_list(n_binds);
    self.put_stmt(Stmt::Let(x, y));
  }

  fn on_return(&mut self, arity: usize) {
    let x = self.pop_expr_list(arity);
    self.put_stmt(Stmt::Return(x));
  }

  fn on_set(&mut self, symbol: &[u8]) {
    let s = Symbol::from_bytes(symbol);
    let x = self.pop_expr();
    self.put_stmt(Stmt::Set(s, x));
  }

  fn on_set_field(&mut self, symbol: &[u8]) {
    let s = Symbol::from_bytes(symbol);
    let y = self.pop_expr();
    let x = self.pop_expr();
    self.put_stmt(Stmt::SetField(x, s, y));
  }

  fn on_set_index(&mut self) {
    let z = self.pop_expr();
    let y = self.pop_expr();
    let x = self.pop_expr();
    self.put_stmt(Stmt::SetIndex(x, y, z));
  }

  fn on_var(&mut self, symbol: &[u8]) {
    let s = Symbol::from_bytes(symbol);
    let x = self.pop_expr();
    self.put_stmt(Stmt::Var(s, x));
  }

  fn on_while(&mut self, n_stmts: usize) {
    let y = self.pop_stmt_list(n_stmts);
    let x = self.pop_expr();
    self.put_stmt(Stmt::While(x, y));
  }

  fn on_error_missing_expected_token(&mut self, token: Token) {
    let _ = token;
    // TODO: report error on missing expected token
  }

  fn on_error_missing_expr(&mut self) {
    // TODO: report error on missing expected expression
    self.put_expr(Expr::Undefined);
  }
}
