use crate::arr::Arr;
use crate::ast::Binding;
use crate::ast::Expr;
use crate::ast::Fun;
use crate::ast::Item;
use crate::ast::Stmt;
use crate::buf::Buf;
use crate::lexer::Lexer;
use crate::operator::Op1;
use crate::operator::Op2;
use crate::sexp::Sexp;
use crate::sexp;
use crate::symbol::Symbol;
use crate::token::Token;
use oxcart::Arena;

pub trait Out {
  fn on_and(&mut self);
  fn on_binding(&mut self, name: Option<&[u8]>); // TODO: type ascription
  fn on_break(&mut self, n_args: u32);
  fn on_call(&mut self, n_args: u32);
  fn on_continue(&mut self);
  fn on_error_missing_expected_token(&mut self, token: Token);
  fn on_error_missing_expr(&mut self);
  fn on_field(&mut self, symbol: &[u8]);
  fn on_fun(&mut self, name: &[u8], n_args: u32, n_stmts: u32);
  fn on_if(&mut self, n_stmts: u32);
  fn on_if_else(&mut self, n_stmts_then: u32, n_stmts_else: u32);
  fn on_index(&mut self);
  fn on_let(&mut self, n_bindings: u32, n_args: u32);
  fn on_literal_bool(&mut self, value: bool);
  fn on_literal_number(&mut self, value: &[u8]);
  fn on_loop(&mut self, n_stmts: u32);
  fn on_op1(&mut self, op: Op1);
  fn on_op2(&mut self, op: Op2);
  fn on_or(&mut self);
  fn on_post_op(&mut self, op: Op1);
  fn on_pre_op(&mut self, op: Op1);
  fn on_return(&mut self, n_args: u32);
  fn on_set(&mut self, symbol: &[u8]);
  fn on_set_field(&mut self, symbol: &[u8]);
  fn on_set_index(&mut self);
  fn on_stmt_expr_list(&mut self, n_exprs: u32);
  fn on_ternary(&mut self);
  fn on_var(&mut self, symbol: &[u8]);
  fn on_variable(&mut self, symbol: &[u8]);
  fn on_while(&mut self, n_stmts: u32);
}

// toplevel sequence of items

pub fn parse<'a, O: Out>(t: &mut Lexer<'a>, o: &mut O) {
  loop {
    match t.token() {
      Token::Eof => {
        break;
      }
      Token::Fun => {
        t.next();
        let name = expect_symbol(t, o);
        expect(t, o, Token::LParen);
        let m = parse_binding_list(t, o, Token::RParen);
        expect(t, o, Token::RParen);
        let n = parse_block(t, o);
        o.on_fun(name, m, n);
      }
      _ => {
        // TODO: error?
        break;
      }
    }
  }
}

fn expect<'a, O: Out>(t: &mut Lexer<'a>, o: &mut O, token: Token) {
  if t.token() != token {
    o.on_error_missing_expected_token(token);
  } else {
    t.next();
  }
}

fn expect_symbol<'a, O: Out>(t: &mut Lexer<'a>, o: &mut O) -> &'a [u8] {
  if t.token() != Token::Symbol {
    o.on_error_missing_expected_token(Token::Symbol);
    // TODO: Option::None?
    return b"!!!";
  } else {
    let symbol = t.token_span();
    t.next();
    return symbol;
  }
}

fn parse_binding<'a, O: Out>(t: &mut Lexer<'a>, o: &mut O) {
  match t.token() {
    Token::Symbol => {
      o.on_binding(Some(t.token_span()));
      t.next();
    }
    Token::Underscore => {
      o.on_binding(None);
      t.next();
    }
    _ => {
      o.on_error_missing_expected_token(Token::Symbol);
      o.on_binding(None);
    }
  }
}

fn parse_binding_list<'a, O: Out>(t: &mut Lexer<'a>, o: &mut O, stop: Token) -> u32 {
  let mut n_bindings = 0;
  if t.token() != stop {
    loop {
      parse_binding(t, o);
      n_bindings += 1;
      if t.token() != Token::Comma { break; }
      t.next();
    }
  }
  return n_bindings;
}

fn parse_expr<'a, O: Out>(t: &mut Lexer<'a>, o: &mut O) {
  parse_expr_prec(t, o, 0);
}

fn parse_expr_list<'a, O: Out>(t: &mut Lexer<'a>, o: &mut O, stop: Token) -> u32 {
  let mut n_exprs = 0;
  if t.token() != stop {
    loop {
      parse_expr(t, o);
      n_exprs += 1;
      if t.token() != Token::Comma { break; }
      t.next();
    }
  }
  return n_exprs;
}

fn parse_expr_prec<'a, O: Out>(t: &mut Lexer<'a>, o: &mut O, n: u32) {
  let _: bool = parse_prec(t, o, n, false);
}

// returns `true` if we parsed a statement

fn parse_prec<'a, O: Out>(t: &mut Lexer<'a>, o: &mut O, n: u32, is_stmt: bool) -> bool {
  match t.token() {
    Token::LParen => {
      t.next();
      parse_expr(t, o);
      expect(t, o, Token::RParen);
    }
    Token::True => {
      t.next();
      o.on_literal_bool(true);
    }
    Token::False => {
      t.next();
      o.on_literal_bool(false);
    }
    Token::Number => {
      let value = t.token_span();
      t.next();
      o.on_literal_number(value);
    }
    Token::Symbol => {
      let symbol = t.token_span();
      t.next();
      match t.token() {
        Token::Equal if is_stmt => {
          t.next();
          parse_expr(t, o);
          o.on_set(symbol);
          return true;
        }
        _ => {
          o.on_variable(symbol);
        }
      }
    }
    Token::Dec => {
      t.next();
      parse_expr_prec(t, o, u32::MAX);
      o.on_pre_op(Op1::Dec);
    }
    Token::Inc => {
      t.next();
      parse_expr_prec(t, o, u32::MAX);
      o.on_pre_op(Op1::Inc);
    }
    Token::Hyphen => {
      t.next();
      parse_expr_prec(t, o, u32::MAX);
      o.on_op1(Op1::Neg);
    }
    Token::Not => {
      t.next();
      parse_expr_prec(t, o, u32::MAX);
      o.on_op1(Op1::Not);
    }
    Token::If => {
      t.next();
      parse_expr(t, o);
      let n = parse_block(t, o);
      if t.token() == Token::Else {
        t.next();
        let m = parse_block(t, o);
        o.on_if_else(n, m);
      } else {
        o.on_if(n);
      }
    }
    Token::Loop => {
      t.next();
      let n = parse_block(t, o);
      o.on_loop(n);
    }
    _ => {
      o.on_error_missing_expr();
    }
  }

  loop {
    match t.token() {
      Token::Query if n <= 10 => {
        t.next();
        parse_expr(t, o);
        expect(t, o, Token::Colon);
        parse_expr_prec(t, o, 10);
        o.on_ternary();
      }
      Token::Or if n <= 20 => {
        t.next();
        parse_expr_prec(t, o, 21);
        o.on_or();
      }
      Token::And if n <= 30 => {
        t.next();
        parse_expr_prec(t, o, 31);
        o.on_and();
      }
      Token::CmpEq if n <= 40 => {
        t.next();
        parse_expr_prec(t, o, 41);
        o.on_op2(Op2::CmpEq);
      }
      Token::CmpGe if n <= 40 => {
        t.next();
        parse_expr_prec(t, o, 41);
        o.on_op2(Op2::CmpGe);
      }
      Token::CmpGt if n <= 40 => {
        t.next();
        parse_expr_prec(t, o, 41);
        o.on_op2(Op2::CmpGt);
      }
      Token::CmpLe if n <= 40 => {
        t.next();
        parse_expr_prec(t, o, 41);
        o.on_op2(Op2::CmpLe);
      }
      Token::CmpLt if n <= 40 => {
        t.next();
        parse_expr_prec(t, o, 41);
        o.on_op2(Op2::CmpLt);
      }
      Token::CmpNe if n <= 40 => {
        t.next();
        parse_expr_prec(t, o, 41);
        o.on_op2(Op2::CmpNe);
      }
      Token::BitOr if n <= 50 => {
        t.next();
        parse_expr_prec(t, o, 51);
        o.on_op2(Op2::BitOr);
      }
      Token::BitXor if n <= 60 => {
        t.next();
        parse_expr_prec(t, o, 61);
        o.on_op2(Op2::BitXor);
      }
      Token::BitAnd if n <= 70 => {
        t.next();
        parse_expr_prec(t, o, 71);
        o.on_op2(Op2::BitAnd);
      }
      Token::Shl if n <= 80 => {
        t.next();
        parse_expr_prec(t, o, 81);
        o.on_op2(Op2::Shl);
      }
      Token::Shr if n <= 80 => {
        t.next();
        parse_expr_prec(t, o, 81);
        o.on_op2(Op2::Shr);
      }
      Token::Add if n <= 90 => {
        t.next();
        parse_expr_prec(t, o, 91);
        o.on_op2(Op2::Add);
      }
      Token::Hyphen if n <= 90 => {
        t.next();
        parse_expr_prec(t, o, 91);
        o.on_op2(Op2::Sub);
      }
      Token::Div if n <= 100 => {
        t.next();
        parse_expr_prec(t, o, 101);
        o.on_op2(Op2::Div);
      }
      Token::Mul if n <= 100 => {
        t.next();
        parse_expr_prec(t, o, 101);
        o.on_op2(Op2::Mul);
      }
      Token::Rem if n <= 100 => {
        t.next();
        parse_expr_prec(t, o, 101);
        o.on_op2(Op2::Rem);
      }
      Token::Dec => {
        t.next();
        o.on_post_op(Op1::Dec);
      }
      Token::Inc => {
        t.next();
        o.on_post_op(Op1::Inc);
      }
      Token::Field if t.token_is_attached() => {
        let symbol = &t.token_span()[1 ..];
        t.next();
        if is_stmt && t.token() == Token::Equal {
          t.next();
          parse_expr(t, o);
          o.on_set_field(symbol);
          return true;
        } else {
          o.on_field(symbol);
        }
      }
      Token::LBracket if t.token_is_attached() => {
        t.next();
        parse_expr(t, o);
        expect(t, o, Token::RBracket);
        if is_stmt && t.token() == Token::Equal {
          t.next();
          parse_expr(t, o);
          o.on_set_index();
          return true;
        } else {
          o.on_index();
        }
      }
      Token::LParen if t.token_is_attached() => {
        t.next();
        let n_args = parse_expr_list(t, o, Token::RParen);
        expect(t, o, Token::RParen);
        o.on_call(n_args);
      }
      _ => {
        return false;
      }
    }
  }
}

fn parse_block<'a, O: Out>(t: &mut Lexer<'a>, o: &mut O) -> u32 {
  expect(t, o, Token::LBrace);

  let mut n_stmts = 0;

  loop {
    match t.token() {
      Token::RBrace => {
        t.next();
        break;
      }
      Token::Break => {
        t.next();
        let n_args = parse_expr_list(t, o, Token::RBrace);
        o.on_break(n_args);
        n_stmts += 1;
        expect(t, o, Token::RBrace);
        break;
      }
      Token::Continue => {
        t.next();
        o.on_continue();
        n_stmts += 1;
        expect(t, o, Token::RBrace);
        break;
      }
      Token::Let => {
        t.next();
        // NB: we allow a list of zero bindings, like
        //
        //   let = f(x)
        //
        // which is a bit weird. but works semantically
        let n_bindings = parse_binding_list(t, o, Token::Equal);
        expect(t, o, Token::Equal);
        let mut n_exprs = 0;
        loop {
          parse_expr(t, o);
          n_exprs += 1;
          if t.token() != Token::Comma { break; }
          t.next();
        }
        o.on_let(n_bindings, n_exprs);
        n_stmts += 1;
      }
      Token::Return => {
        t.next();
        let n_args = parse_expr_list(t, o, Token::RBrace);
        o.on_return(n_args);
        n_stmts += 1;
        expect(t, o, Token::RBrace);
        break;
      }
      Token::Var => {
        t.next();
        let symbol = expect_symbol(t, o);
        expect(t, o, Token::Equal);
        parse_expr(t, o);
        o.on_var(symbol);
        n_stmts += 1;
      }
      Token::While => {
        t.next();
        parse_expr(t, o);
        let n = parse_block(t, o);
        o.on_while(n);
        n_stmts += 1;
      }
      _ => {
        // NB: If we couldn't parse anything at all, then we immediately close
        // the block so that we don't get stuck in an infinite loop.
        //
        // Note that we already know that there ISN'T an RBrace here, so the
        // expect will fail.
        //
        // Also, note that in this case we still emit an `undefined` expr/stmt.

        let pos = t.token_start();

        if ! parse_prec(t, o, 0, true) {
          let mut n_exprs = 1;
          while t.token() == Token::Comma {
            t.next();
            parse_expr(t, o);
            n_exprs += 1;
          }
          o.on_stmt_expr_list(n_exprs);
        }

        n_stmts += 1;

        if t.token_start() == pos {
          expect(t, o, Token::RBrace);
          break;
        }
      }
    }
  }

  return n_stmts;
}

// ------- EXAMPLE PARSE OUTPUT - DUMP AS SEXP -------

struct ToSexp(Buf<Sexp>);

pub fn parse_sexp(source: &[u8]) -> Buf<Sexp> {
  let mut out = ToSexp::new();
  parse(&mut Lexer::new(source), &mut out);
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

impl Out for ToSexp {
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

  fn on_post_op(&mut self, op: Op1) {
    let x = self.pop();
    self.put(sexp::list([sexp::atom("%post"), sexp::atom(op.as_str()), x]));
  }

  fn on_pre_op(&mut self, op: Op1) {
    let x = self.pop();
    self.put(sexp::list([sexp::atom("%pre"), sexp::atom(op.as_str()), x]));
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

// ------- PARSE INTO ARENA-ALLOCATED AST -------

pub fn parse_ast<'a>(source: &[u8], arena: &mut Arena<'a>) -> Arr<Item<'a>> {
  let mut out = ToAst::new(arena);
  parse(&mut Lexer::new(source), &mut out);
  return Arr::new(out.items.drain());
}

struct ToAst<'a, 'b> {
  arena: &'b mut Arena<'a>,
  items: Buf<Item<'a>>,
  binds: Buf<Binding>,
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

  fn put_bind(&mut self, x: Binding) {
    let _ = self.binds.put(x);
  }

  fn pop_bind_list(&mut self, n: u32) -> &'a [Binding] {
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

impl<'a, 'b> Out for ToAst<'a, 'b> {
  fn on_fun(&mut self, name: &[u8], n_args: u32, n_stmts: u32) {
    let z = self.pop_stmt_list(n_stmts);
    let y = self.pop_bind_list(n_args);
    let x = Symbol::from_bytes(name);
    let x = Item::Fun(Fun { name: x, args: y, body: z });
    self.put_item(x);
  }

  fn on_binding(&mut self, name: Option<&[u8]>) {
    let x = Binding { name: name.map(Symbol::from_bytes) };
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

  fn on_post_op(&mut self, op: Op1) {
    let x = self.pop_expr();
    let x = Expr::PostOp(self.alloc((op, x)));
    self.put_expr(x);
  }

  fn on_pre_op(&mut self, op: Op1) {
    let x = self.pop_expr();
    let x = Expr::PreOp(self.alloc((op, x)));
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
