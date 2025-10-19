use crate::lexer::Lexer;
use crate::op1::Op1;
use crate::op2::Op2;
use crate::parse;
use crate::token::Token;
use oxcart::Arena;

#[derive(Clone, Copy, Debug)]
pub enum Expr<'a> {
  And(&'a (Expr<'a>, Expr<'a>)),
  Call(&'a (Expr<'a>, &'a [Expr<'a>])),
  Field(&'a (Expr<'a>, &'a [u8])),
  Index(&'a (Expr<'a>, Expr<'a>)),
  Int(i64),
  Op1(&'a (Op1, Expr<'a>)),
  Op2(&'a (Op2, Expr<'a>, Expr<'a>)),
  Or(&'a (Expr<'a>, Expr<'a>)),
  Ternary(&'a (Expr<'a>, Expr<'a>, Expr<'a>)),
  Undefined,
  Variable(&'a [u8]),
}

#[derive(Debug)]
pub enum Stmt<'a> {
  Expr(Expr<'a>),
  Let(&'a (&'a [u8], Expr<'a>)),
  Set(&'a (&'a [u8], Expr<'a>)),
  SetField(&'a (Expr<'a>, &'a [u8], Expr<'a>)),
  SetIndex(&'a (Expr<'a>, Expr<'a>, Expr<'a>)),
  Var(&'a (&'a [u8], Expr<'a>)),
}

pub fn parse_expr<'a>(source: &[u8], arena: &mut Arena<'a>) -> Expr<'a> {
  let mut e = ToAst::new(arena);
  parse::parse_expr(&mut Lexer::new(source), &mut e);
  return e.pop_expr();
}

pub fn parse_stmt<'a>(source: &[u8], arena: &mut Arena<'a>) -> Stmt<'a> {
  let mut e = ToAst::new(arena);
  parse::parse_stmt(&mut Lexer::new(source), &mut e);
  return e.pop_stmt();
}

struct ToAst<'a, 'b> {
  arena: &'b mut Arena<'a>,
  exprs: Vec<Expr<'a>>,
  stmts: Vec<Stmt<'a>>,
}

impl<'a, 'b> ToAst<'a, 'b> {
  fn new(arena: &'b mut Arena<'a>) -> Self {
    Self {
      arena,
      exprs: Vec::new(),
      stmts: Vec::new(),
    }
  }

  fn alloc<T>(&mut self, x: T) -> &'a T {
    return self.arena.alloc().init(x);
  }

  fn copy_symbol(&mut self, symbol: &[u8]) -> &'a [u8] {
    return self.arena.copy_slice(symbol);
  }

  fn put_expr(&mut self, x: Expr<'a>) {
    self.exprs.push(x);
  }

  fn pop_expr(&mut self) -> Expr<'a> {
    return self.exprs.pop().unwrap();
  }

  fn pop_expr_multi(&mut self, n: usize) -> &'a [Expr<'a>] {
    let x = self.exprs.drain(self.exprs.len() - n ..);
    return self.arena.slice_from_iter(x);
  }

  fn put_stmt(&mut self, x: Stmt<'a>) {
    self.stmts.push(x);
  }

  fn pop_stmt(&mut self) -> Stmt<'a> {
    return self.stmts.pop().unwrap();
  }
}

impl<'a, 'b> parse::Sink for ToAst<'a, 'b> {
  fn on_variable(&mut self, symbol: &[u8]) {
    let s = self.copy_symbol(symbol);
    let x = Expr::Variable(s);
    self.put_expr(x);
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
    let s = self.copy_symbol(symbol);
    let x = self.pop_expr();
    let x = Expr::Field(self.alloc((x, s)));
    self.put_expr(x);
  }

  fn on_index(&mut self) {
    let i = self.pop_expr();
    let x = self.pop_expr();
    let x = Expr::Index(self.alloc((x, i)));
    self.put_expr(x);
  }

  fn on_call(&mut self, arity: usize) {
    let x = self.pop_expr_multi(arity);
    let f = self.pop_expr();
    let x = Expr::Call(self.alloc((f, x)));
    self.put_expr(x);
  }

  fn on_stmt_expr(&mut self) {
    let x = self.pop_expr();
    self.put_stmt(Stmt::Expr(x));
  }

  fn on_let(&mut self, symbol: &[u8]) {
    let s = self.copy_symbol(symbol);
    let x = self.pop_expr();
    let x = Stmt::Let(self.alloc((s, x)));
    self.put_stmt(x);
  }

  fn on_set(&mut self, symbol: &[u8]) {
    let s = self.copy_symbol(symbol);
    let x = self.pop_expr();
    let x = Stmt::Set(self.alloc((s, x)));
    self.put_stmt(x);
  }

  fn on_set_field(&mut self, symbol: &[u8]) {
    let s = self.copy_symbol(symbol);
    let y = self.pop_expr();
    let x = self.pop_expr();
    let x = Stmt::SetField(self.alloc((x, s, y)));
    self.put_stmt(x);
  }

  fn on_set_index(&mut self) {
    let y = self.pop_expr();
    let i = self.pop_expr();
    let x = self.pop_expr();
    let x = Stmt::SetIndex(self.alloc((x, i, y)));
    self.put_stmt(x);
  }

  fn on_var(&mut self, symbol: &[u8]) {
    let s = self.copy_symbol(symbol);
    let x = self.pop_expr();
    let x = Stmt::Var(self.alloc((s, x)));
    self.put_stmt(x);
  }

  fn on_error_missing_expected_token(&mut self, token: Token) {
    let _ = token;
    // TODO: accumulate errors
  }

  fn on_error_missing_expr(&mut self) {
    // TODO: accumulate errors
    self.put_expr(Expr::Undefined);
  }
}
