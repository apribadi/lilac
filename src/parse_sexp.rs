use crate::buf::Buf;
use crate::ir1::Op1;
use crate::ir1::Op2;
use crate::lexer::Lexer;
use crate::parse;
use crate::sexp::Sexp;
use crate::token::Token;

struct ToSexp(Buf<Sexp>);

pub fn parse(source: &[u8]) -> Buf<Sexp> {
  let mut o = ToSexp::new();
  parse::parse(&mut Lexer::new(source), &mut o);
  return o.0;
}

impl ToSexp {
  fn new() -> Self {
    Self(Buf::new())
  }

  fn put(&mut self, x: Sexp) {
    let _ = self.0.put(x);
  }

  fn pop(&mut self) -> Sexp {
    return self.0.pop();
  }

  fn pop_list(&mut self, n: u32) -> impl ExactSizeIterator<Item = Sexp> {
    return self.0.pop_list(n);
  }
}

impl parse::Out for ToSexp {
  fn on_fundef(&mut self, name: &[u8], n_args: u32, n_stmts: u32) {
    let z = Sexp::list(self.pop_list(n_stmts));
    let y = Sexp::list(self.pop_list(n_args));
    let x = Sexp::atom(name);
    let t = Sexp::atom("fun");
    self.put(Sexp::list([t, x, y, z]));
  }

  fn on_bind(&mut self, name: Option<&[u8]>) {
    let x =
      match name {
        None => Sexp::atom("_"),
        Some(name) => Sexp::atom(name),
      };
    self.put(x);
  }

  fn on_variable(&mut self, x: &[u8]) {
    self.put(Sexp::atom(x));
  }

  fn on_bool(&mut self, x: bool) {
    self.put(if x { Sexp::atom("true") } else { Sexp::atom("false") });
  }

  fn on_number(&mut self, x: &[u8]) {
    self.put(Sexp::atom(x));
  }

  fn on_ternary(&mut self) {
    let z = self.pop();
    let y = self.pop();
    let x = self.pop();
    let t = Sexp::atom("?:");
    self.put(Sexp::list([t, x, y, z]));
  }

  fn on_or(&mut self) {
    let y = self.pop();
    let x = self.pop();
    let t = Sexp::atom("||");
    self.put(Sexp::list([t, x, y]));
  }

  fn on_and(&mut self) {
    let y = self.pop();
    let x = self.pop();
    let t = Sexp::atom("&&");
    self.put(Sexp::list([t, x, y]));
  }

  fn on_op1(&mut self, op: Op1) {
    let x = self.pop();
    let t = Sexp::atom(op.as_str().as_bytes());
    self.put(Sexp::list([t, x]));
  }

  fn on_op2(&mut self, op: Op2) {
    let y = self.pop();
    let x = self.pop();
    let t = Sexp::atom(op.as_str().as_bytes());
    self.put(Sexp::list([t, x, y]));
  }

  fn on_field(&mut self, symbol: &[u8]) {
    let s = Sexp::atom(format!(".{}", str::from_utf8(symbol).unwrap()).as_bytes());
    let x = self.pop();
    self.put(Sexp::list([s, x]));
  }

  fn on_index(&mut self) {
    let y = self.pop();
    let x = self.pop();
    let t = Sexp::atom("[]");
    self.put(Sexp::list([t, x, y]));
  }

  fn on_if(&mut self, n_stmts: u32) {
    let y = Sexp::list(self.pop_list(n_stmts));
    let x = self.pop();
    let t = Sexp::atom("if");
    self.put(Sexp::list([t, x, y]));
  }

  fn on_if_else(&mut self, n_stmts_then: u32, n_stmts_else: u32) {
    let z = Sexp::list(self.pop_list(n_stmts_else));
    let y = Sexp::list(self.pop_list(n_stmts_then));
    let x = self.pop();
    let t = Sexp::atom("if");
    self.put(Sexp::list([t, x, y, z]));
  }

  fn on_call(&mut self, arity: u32) {
    let x = Sexp::list(self.pop_list(1 + arity));
    self.put(x);
  }

  fn on_loop(&mut self, n_stmts: u32) {
    let x = Sexp::list(self.pop_list(n_stmts));
    let t = Sexp::atom("loop");
    self.put(Sexp::list([t, x]));
  }

  fn on_stmt_expr_list(&mut self, n_exprs: u32) {
    let x = Sexp::list(self.pop_list(n_exprs));
    self.put(x);
  }

  fn on_break(&mut self, arity: u32) {
    let x = Sexp::atom("break");
    let y = Sexp::list(self.pop_list(arity));
    self.put(Sexp::list([x, y]));
  }

  fn on_continue(&mut self) {
    self.put(Sexp::atom("continue"));
  }

  fn on_let(&mut self, n_binds: u32, n_exprs: u32) {
    let y = Sexp::list(self.pop_list(n_exprs));
    let x = Sexp::list(self.pop_list(n_binds));
    self.put(Sexp::list([Sexp::atom("let"), x, y]));
  }

  fn on_return(&mut self, arity: u32) {
    let x = Sexp::atom("return");
    let y = Sexp::list(self.pop_list(arity));
    self.put(Sexp::list([x, y]));
  }

  fn on_set(&mut self, symbol: &[u8]) {
    let x = self.pop();
    let s = Sexp::atom(symbol);
    let t = Sexp::atom("<-");
    self.put(Sexp::list([t, s, x]));
  }

  fn on_set_field(&mut self, symbol: &[u8]) {
    let s = Sexp::atom(format!(".{}<-", str::from_utf8(symbol).unwrap()).as_bytes());
    let y = self.pop();
    let x = self.pop();
    self.put(Sexp::list([s, x, y]));
  }

  fn on_set_index(&mut self) {
    let z = self.pop();
    let y = self.pop();
    let x = self.pop();
    let t = Sexp::atom("[]<-");
    self.put(Sexp::list([t, x, y, z]));
  }

  fn on_var(&mut self, symbol: &[u8]) {
    let x = self.pop();
    let s = Sexp::atom(symbol);
    let t = Sexp::atom("var");
    self.put(Sexp::list([t, s, x]));
  }

  fn on_while(&mut self, n_stmts: u32) {
    let y = Sexp::list(self.pop_list(n_stmts));
    let x = self.pop();
    let t = Sexp::atom("while");
    self.put(Sexp::list([t, x, y]));
  }

  fn on_error_missing_expected_token(&mut self, _: Token) {
  }

  fn on_error_missing_expr(&mut self) {
    self.put(Sexp::atom("undefined"));
  }
}
