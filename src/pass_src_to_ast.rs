use crate::arr::Arr;
use crate::ast::Bind;
use crate::ast::Expr;
use crate::ast::Fun;
use crate::ast::Item;
use crate::ast::Stmt;
use crate::buf::Buf;
use crate::lexer::Lexer;
use crate::operator::Op1;
use crate::operator::Op2;
use crate::parse;
use crate::symbol::Symbol;
use crate::token::Token;
use oxcart::Arena;

pub fn parse<'a>(source: &[u8], arena: &mut Arena<'a>) -> Arr<Item<'a>> {
  let mut out = ToAst::new(arena);
  parse::parse(&mut Lexer::new(source), &mut out);
  return Arr::new(out.items.drain());
}

struct ToAst<'a, 'b> {
  arena: &'b mut Arena<'a>,
  items: Buf<Item<'a>>,
  binds: Buf<Bind>,
  exprs: Buf<Expr<'a>>,
  stmts: Buf<Stmt<'a>>,
}

impl<'a, 'b> ToAst<'a, 'b> {
  fn new(arena: &'b mut Arena<'a>) -> Self {
    Self {
      arena,
      items: Buf::new(),
      binds: Buf::new(),
      exprs: Buf::new(),
      stmts: Buf::new(),
    }
  }

  fn alloc<T>(&mut self, x: T) -> &'a T {
    return self.arena.alloc().init(x);
  }

  fn put_item(&mut self, x: Item<'a>) {
    let _ = self.items.put(x);
  }

  fn put_bind(&mut self, x: Bind) {
    let _ = self.binds.put(x);
  }

  fn pop_bind_list(&mut self, n: u32) -> &'a [Bind] {
    return self.arena.slice_from_iter(self.binds.pop_list(n));
  }

  fn put_expr(&mut self, x: Expr<'a>) {
    let _ = self.exprs.put(x);
  }

  fn pop_expr(&mut self) -> Expr<'a> {
    return self.exprs.pop();
  }

  fn pop_expr_list(&mut self, n: u32) -> &'a [Expr<'a>] {
    return self.arena.slice_from_iter(self.exprs.pop_list(n));
  }

  fn put_stmt(&mut self, x: Stmt<'a>) {
    let _ = self.stmts.put(x);
  }

  fn pop_stmt_list(&mut self, n: u32) -> &'a [Stmt<'a>] {
    return self.arena.slice_from_iter(self.stmts.pop_list(n));
  }
}

impl<'a, 'b> parse::Out for ToAst<'a, 'b> {
  fn on_fun(&mut self, name: &[u8], n_args: u32, n_stmts: u32) {
    let z = self.pop_stmt_list(n_stmts);
    let y = self.pop_bind_list(n_args);
    let x = Symbol::from_bytes(name);
    let x = Item::Fun(Fun { name: x, args: y, body: z });
    self.put_item(x);
  }

  fn on_binding(&mut self, name: Option<&[u8]>) {
    let x = Bind { name: name.map(Symbol::from_bytes) };
    self.put_bind(x);
  }

  fn on_variable(&mut self, symbol: &[u8]) {
    let s = Symbol::from_bytes(symbol);
    self.put_expr(Expr::Variable(s));
  }

  fn on_literal_bool(&mut self, value: bool) {
    self.put_expr(Expr::Bool(value));
  }

  fn on_literal_number(&mut self, x: &[u8]) {
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

  fn on_post_op2(&mut self, symbol: &[u8], op: Op2) {
    unimplemented!()
  }

  fn on_pre_op2(&mut self, op: Op2, symbol: &[u8]) {
    unimplemented!()
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

  fn on_if(&mut self, n_stmts: u32) {
    let y = self.pop_stmt_list(n_stmts);
    let x = self.pop_expr();
    let x = Expr::If(self.alloc((x, y)));
    self.put_expr(x);
  }

  fn on_if_else(&mut self, n_stmts_then: u32, n_stmts_else: u32) {
    let z = self.pop_stmt_list(n_stmts_else);
    let y = self.pop_stmt_list(n_stmts_then);
    let x = self.pop_expr();
    let x = Expr::IfElse(self.alloc((x, y, z)));
    self.put_expr(x);
  }

  fn on_call(&mut self, n_args: u32) {
    let x = self.pop_expr_list(n_args);
    let f = self.pop_expr();
    let x = Expr::Call(self.alloc((f, x)));
    self.put_expr(x);
  }

  fn on_loop(&mut self, n_stmts: u32) {
    let x = self.pop_stmt_list(n_stmts);
    self.put_expr(Expr::Loop(x));
  }

  fn on_stmt_expr_list(&mut self, n_exprs: u32) {
    let x = self.pop_expr_list(n_exprs);
    self.put_stmt(Stmt::ExprList(x));
  }

  fn on_break(&mut self, n_args: u32) {
    let x = self.pop_expr_list(n_args);
    self.put_stmt(Stmt::Break(x));
  }

  fn on_continue(&mut self) {
    self.put_stmt(Stmt::Continue);
  }

  fn on_let(&mut self, n_bindings: u32, n_exprs: u32) {
    let y = self.pop_expr_list(n_exprs);
    let x = self.pop_bind_list(n_bindings);
    self.put_stmt(Stmt::Let(x, y));
  }

  fn on_return(&mut self, n_args: u32) {
    let x = self.pop_expr_list(n_args);
    self.put_stmt(Stmt::Return(x));
  }

  fn on_set(&mut self, symbol: &[u8]) {
    let s = Symbol::from_bytes(symbol);
    let x = self.pop_expr();
    self.put_stmt(Stmt::Set(s, x));
  }

  fn on_set_op2(&mut self, symbol: &[u8], op: Op2) {
    let s = Symbol::from_bytes(symbol);
    let x = self.pop_expr();
    let x = Expr::Op2(self.alloc((op, Expr::Variable(s), x)));
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

  fn on_while(&mut self, n_stmts: u32) {
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
