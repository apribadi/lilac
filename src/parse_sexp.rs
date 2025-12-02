use crate::buf::Buf;
use crate::ir1::Op1;
use crate::ir1::Op2;
use crate::lexer::Lexer;
use crate::parse;
use crate::sexp::Sexp;
use crate::sexp;
use crate::token::Token;

struct ToSexp(Buf<Sexp>);

pub fn parse(source: &[u8]) -> Buf<Sexp> {
  let mut out = ToSexp::new();
  parse::parse(&mut Lexer::new(source), &mut out);
  return out.0;
}

impl ToSexp {
  fn new() -> Self {
    return Self(Buf::new());
  }

  fn put(&mut self, x: Sexp) {
    self.0.put(x);
  }

  fn pop(&mut self) -> Sexp {
    return self.0.pop();
  }

  fn pop_list(&mut self, n: u32) -> impl ExactSizeIterator<Item = Sexp> {
    return self.0.pop_list(n);
  }
}

impl parse::Out for ToSexp {
  fn on_fun(&mut self, name: &[u8], n_args: u32, n_stmts: u32) {
    let z = sexp::list(self.pop_list(n_stmts));
    let y = sexp::list(self.pop_list(n_args));
    let x = sexp::atom(name);
    self.put(sexp::list([sexp::atom("fun"), x, y, z]));
  }

  fn on_binding(&mut self, name: Option<&[u8]>) {
    self.put(sexp::atom(match name { None => b"_", Some(name) => name }));
  }

  fn on_variable(&mut self, symbol: &[u8]) {
    self.put(sexp::atom(symbol));
  }

  fn on_literal_bool(&mut self, value: bool) {
    self.put(sexp::atom(if value { "true" } else { "false" }));
  }

  fn on_literal_number(&mut self, value: &[u8]) {
    self.put(sexp::atom(value));
  }

  fn on_ternary(&mut self) {
    let z = self.pop();
    let y = self.pop();
    let x = self.pop();
    self.put(sexp::list([sexp::atom(":?"), x, y, z]));
  }

  fn on_or(&mut self) {
    let y = self.pop();
    let x = self.pop();
    self.put(sexp::list([sexp::atom("||"), x, y]));
  }

  fn on_and(&mut self) {
    let y = self.pop();
    let x = self.pop();
    self.put(sexp::list([sexp::atom("&&"), x, y]));
  }

  fn on_op1(&mut self, op: Op1) {
    let x = self.pop();
    self.put(sexp::list([sexp::atom(op.as_str()), x]));
  }

  fn on_op2(&mut self, op: Op2) {
    let y = self.pop();
    let x = self.pop();
    self.put(sexp::list([sexp::atom(op.as_str()), x, y]));
  }

  fn on_field(&mut self, symbol: &[u8]) {
    let s = sexp::atom(format!(".{}", str::from_utf8(symbol).unwrap()));
    let x = self.pop();
    self.put(sexp::list([s, x]));
  }

  fn on_index(&mut self) {
    let y = self.pop();
    let x = self.pop();
    self.put(sexp::list([sexp::atom("[]"), x, y]));
  }

  fn on_if(&mut self, n_stmts: u32) {
    let y = sexp::list(self.pop_list(n_stmts));
    let x = self.pop();
    self.put(sexp::list([sexp::atom("if"), x, y]));
  }

  fn on_if_else(&mut self, n_stmts_then: u32, n_stmts_else: u32) {
    let z = sexp::list(self.pop_list(n_stmts_else));
    let y = sexp::list(self.pop_list(n_stmts_then));
    let x = self.pop();
    self.put(sexp::list([sexp::atom("if"), x, y, z]));
  }

  fn on_call(&mut self, n_args: u32) {
    let x = sexp::list(self.pop_list(1 + n_args));
    self.put(x);
  }

  fn on_loop(&mut self, n_stmts: u32) {
    let x = sexp::list(self.pop_list(n_stmts));
    self.put(sexp::list([sexp::atom("loop"), x]));
  }

  fn on_stmt_expr_list(&mut self, n_exprs: u32) {
    let x = sexp::list(self.pop_list(n_exprs));
    self.put(x);
  }

  fn on_break(&mut self, n_args: u32) {
    let x = sexp::atom("break");
    let y = sexp::list(self.pop_list(n_args));
    self.put(sexp::list([x, y]));
  }

  fn on_continue(&mut self) {
    self.put(sexp::atom("continue"));
  }

  fn on_let(&mut self, n_bindings: u32, n_exprs: u32) {
    let y = sexp::list(self.pop_list(n_exprs));
    let x = sexp::list(self.pop_list(n_bindings));
    self.put(sexp::list([sexp::atom("let"), x, y]));
  }

  fn on_return(&mut self, n_args: u32) {
    let x = sexp::atom("return");
    let y = sexp::list(self.pop_list(n_args));
    self.put(sexp::list([x, y]));
  }

  fn on_set(&mut self, symbol: &[u8]) {
    let x = self.pop();
    let s = sexp::atom(symbol);
    self.put(sexp::list([sexp::atom("="), s, x]));
  }

  fn on_set_field(&mut self, symbol: &[u8]) {
    let s = sexp::atom(format!(".{}<-", str::from_utf8(symbol).unwrap()).as_bytes());
    let y = self.pop();
    let x = self.pop();
    self.put(sexp::list([s, x, y]));
  }

  fn on_set_index(&mut self) {
    let z = self.pop();
    let y = self.pop();
    let x = self.pop();
    self.put(sexp::list([sexp::atom("[]="), x, y, z]));
  }

  fn on_var(&mut self, symbol: &[u8]) {
    let x = self.pop();
    let s = sexp::atom(symbol);
    self.put(sexp::list([sexp::atom("var"), s, x]));
  }

  fn on_while(&mut self, n_stmts: u32) {
    let y = sexp::list(self.pop_list(n_stmts));
    let x = self.pop();
    self.put(sexp::list([sexp::atom("while"), x, y]));
  }

  fn on_error_missing_expected_token(&mut self, _: Token) {
  }

  fn on_error_missing_expr(&mut self) {
    self.put(sexp::atom("undefined"));
  }
}
