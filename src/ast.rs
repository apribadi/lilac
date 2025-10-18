use oxcart::Arena;
use crate::parse;
use crate::token::Token;
use crate::lexer::Lexer;
use crate::op1::Op1;
use crate::op2::Op2;

#[derive(Clone, Copy, Debug)]
pub enum Expr<'a> {
  And(&'a (Expr<'a>, Expr<'a>)),
  Call(&'a (Expr<'a>, &'a [Expr<'a>])),
  Field(&'a (&'a [u8], Expr<'a>)),
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
}

pub fn parse_expr<'a>(source: &[u8], arena: &mut Arena<'a>) -> Expr<'a> {
  let mut e = AstEmit::new(arena);
  parse::parse_expr(&mut Lexer::new(source), &mut e);
  return e.pop_expr();
}

pub fn parse_stmt<'a>(source: &[u8], arena: &mut Arena<'a>) -> Stmt<'a> {
  let mut e = AstEmit::new(arena);
  parse::parse_stmt(&mut Lexer::new(source), &mut e);
  return e.stmts.pop().unwrap();
}

struct AstEmit<'a, 'b> {
  arena: &'b mut Arena<'a>,
  exprs: Vec<Expr<'a>>,
  stmts: Vec<Stmt<'a>>,
}

impl<'a, 'b> AstEmit<'a, 'b> {
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
}

impl<'a, 'b> parse::Emit for AstEmit<'a, 'b> {
  fn emit_variable(&mut self, symbol: &[u8]) {
    let x = Expr::Variable(self.copy_symbol(symbol));
    self.put_expr(x);
  }

  fn emit_number(&mut self, x: &[u8]) {
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

  fn emit_ternary(&mut self) {
    let y = self.pop_expr();
    let x = self.pop_expr();
    let p = self.pop_expr();
    let x = Expr::Ternary(self.alloc((p, x, y)));
    self.put_expr(x);
  }

  fn emit_or(&mut self) {
    let y = self.pop_expr();
    let x = self.pop_expr();
    let x = Expr::Or(self.alloc((x, y)));
    self.put_expr(x);
  }

  fn emit_and(&mut self) {
    let y = self.pop_expr();
    let x = self.pop_expr();
    let x = Expr::And(self.alloc((x, y)));
    self.put_expr(x);
  }

  fn emit_op1(&mut self, op: Op1) {
    let x = self.pop_expr();
    let x = Expr::Op1(self.alloc((op, x)));
    self.put_expr(x);
  }

  fn emit_op2(&mut self, op: Op2) {
    let y = self.pop_expr();
    let x = self.pop_expr();
    let x = Expr::Op2(self.alloc((op, x, y)));
    self.put_expr(x);
  }

  fn emit_field(&mut self, symbol: &[u8]) {
    let symbol = self.copy_symbol(symbol);
    let x = self.pop_expr();
    let x = Expr::Field(self.alloc((symbol, x)));
    self.put_expr(x);
  }

  fn emit_index(&mut self) {
    let i = self.pop_expr();
    let x = self.pop_expr();
    let x = Expr::Index(self.alloc((x, i)));
    self.put_expr(x);
  }

  fn emit_call(&mut self, arity: usize) {
    let x = self.pop_expr_multi(arity);
    let f = self.pop_expr();
    let x = Expr::Call(self.alloc((f, x)));
    self.put_expr(x);
  }

  fn emit_let(&mut self, symbol: &[u8]) {
    let symbol = self.copy_symbol(symbol);
    let x = self.pop_expr();
    let x = Stmt::Let(self.alloc((symbol, x)));
    self.put_stmt(x);
  }

  fn emit_stmt_expr(&mut self) {
    let x = self.pop_expr();
    self.put_stmt(Stmt::Expr(x));
  }

  fn emit_error_missing_expected_token(&mut self, token: Token) {
    let _ = token;
    // TODO: accumulate errors
  }

  fn emit_error_missing_expr(&mut self) {
    // TODO: accumulate errors
    self.put_expr(Expr::Undefined);
  }
}
