use crate::lexer::Lexer;
use crate::op1::Op1;
use crate::op2::Op2;
use crate::sexp::Sexp;
use crate::token::Token;

pub trait Out {
  fn on_variable(&mut self, symbol: &[u8]);

  fn on_number(&mut self, number: &[u8]);

  fn on_ternary(&mut self);

  fn on_or(&mut self);

  fn on_and(&mut self);

  fn on_op1(&mut self, op: Op1);

  fn on_op2(&mut self, op: Op2);

  fn on_field(&mut self, symbol: &[u8]);

  fn on_index(&mut self);

  fn on_call(&mut self, arity: usize);

  fn on_loop(&mut self, n_stmts: usize);

  fn on_stmt_expr(&mut self);

  fn on_break(&mut self, arity: usize);

  fn on_let(&mut self, symbol: &[u8]);

  fn on_return(&mut self, arity: usize);

  fn on_set(&mut self, symbol: &[u8]);

  fn on_set_field(&mut self, symbol: &[u8]);

  fn on_set_index(&mut self);

  fn on_var(&mut self, symbol: &[u8]);

  fn on_error_missing_expected_token(&mut self, token: Token);

  fn on_error_missing_expr(&mut self);
}

pub fn parse_expr<'a, O: Out>(t: &mut Lexer<'a>, o: &mut O) {
  return parse_prec(t, o, 0x00, false);
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
    return b"!!!";
  } else {
    let symbol = t.token_span();
    t.next();
    return symbol;
  }
}

fn parse_expr_list<'a, O: Out>(t: &mut Lexer<'a>, o: &mut O, stop: Token) -> usize {
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

fn parse_prec<'a, O: Out>(t: &mut Lexer<'a>, o: &mut O, n: usize, is_stmt: bool) {
  match t.token() {
    Token::LParen => {
      t.next();
      parse_expr(t, o);
      expect(t, o, Token::RParen);
    }
    Token::Number => {
      let number = t.token_span();
      t.next();
      o.on_number(number);
    }
    Token::Symbol => {
      let symbol = t.token_span();
      t.next();
      if is_stmt && t.token() == Token::Set {
        t.next();
        parse_expr(t, o);
        o.on_set(symbol);
        return;
      } else {
        o.on_variable(symbol);
      }
    }
    Token::Hyphen => {
      t.next();
      parse_prec(t, o, 0xff, false);
      o.on_op1(Op1::Neg);
    }
    Token::Not => {
      t.next();
      parse_prec(t, o, 0xff, false);
      o.on_op1(Op1::Not);
    }
    Token::Loop => {
      t.next();
      let n_stmts = parse_block(t, o);
      o.on_loop(n_stmts);
    }
    _ => {
      o.on_error_missing_expr();
    }
  }

  loop {
    match t.token() {
      Token::Query if n <= 0x10 => {
        t.next();
        parse_expr(t, o);
        expect(t, o, Token::Colon);
        parse_prec(t, o, 0x10, false);
        o.on_ternary();
      }
      Token::Or if n <= 0x20 => {
        t.next();
        parse_prec(t, o, 0x21, false);
        o.on_or();
      }
      Token::And if n <= 0x30 => {
        t.next();
        parse_prec(t, o, 0x31, false);
        o.on_and();
      }
      Token::CmpEq if n <= 0x40 => {
        t.next();
        parse_prec(t, o, 0x41, false);
        o.on_op2(Op2::CmpEq);
      }
      Token::CmpGe if n <= 0x40 => {
        t.next();
        parse_prec(t, o, 0x41, false);
        o.on_op2(Op2::CmpGe);
      }
      Token::CmpGt if n <= 0x40 => {
        t.next();
        parse_prec(t, o, 0x41, false);
        o.on_op2(Op2::CmpGt);
      }
      Token::CmpLe if n <= 0x40 => {
        t.next();
        parse_prec(t, o, 0x41, false);
        o.on_op2(Op2::CmpLe);
      }
      Token::CmpLt if n <= 0x40 => {
        t.next();
        parse_prec(t, o, 0x41, false);
        o.on_op2(Op2::CmpLt);
      }
      Token::CmpNe if n <= 0x40 => {
        t.next();
        parse_prec(t, o, 0x41, false);
        o.on_op2(Op2::CmpNe);
      }
      Token::BitOr if n <= 0x50 => {
        t.next();
        parse_prec(t, o, 0x51, false);
        o.on_op2(Op2::BitOr);
      }
      Token::BitXor if n <= 0x60 => {
        t.next();
        parse_prec(t, o, 0x61, false);
        o.on_op2(Op2::BitXor);
      }
      Token::BitAnd if n <= 0x70 => {
        t.next();
        parse_prec(t, o, 0x71, false);
        o.on_op2(Op2::BitAnd);
      }
      Token::Shl if n <= 0x80 => {
        t.next();
        parse_prec(t, o, 0x81, false);
        o.on_op2(Op2::Shl);
      }
      Token::Shr if n <= 0x80 => {
        t.next();
        parse_prec(t, o, 0x81, false);
        o.on_op2(Op2::Shr);
      }
      Token::Add if n <= 0x90 => {
        t.next();
        parse_prec(t, o, 0x91, false);
        o.on_op2(Op2::Add);
      }
      Token::Hyphen if n <= 0x90 => {
        t.next();
        parse_prec(t, o, 0x91, false);
        o.on_op2(Op2::Sub);
      }
      Token::Div if n <= 0xA0 => {
        t.next();
        parse_prec(t, o, 0xA1, false);
        o.on_op2(Op2::Div);
      }
      Token::Mul if n <= 0xA0 => {
        t.next();
        parse_prec(t, o, 0xA1, false);
        o.on_op2(Op2::Mul);
      }
      Token::Rem if n <= 0xA0 => {
        t.next();
        parse_prec(t, o, 0xA1, false);
        o.on_op2(Op2::Rem);
      }
      Token::Field if t.token_is_attached() => {
        let symbol = &t.token_span()[1 ..];
        t.next();
        if is_stmt && t.token() == Token::Set {
          t.next();
          parse_expr(t, o);
          o.on_set_field(symbol);
          return;
        } else {
          o.on_field(symbol);
        }
      }
      Token::LBracket if t.token_is_attached() => {
        t.next();
        parse_expr(t, o);
        expect(t, o, Token::RBracket);
        if is_stmt && t.token() == Token::Set {
          t.next();
          parse_expr(t, o);
          o.on_set_index();
          return;
        } else {
          o.on_index();
        }
      }
      Token::LParen if t.token_is_attached() => {
        t.next();
        let arity = parse_expr_list(t, o, Token::RParen);
        expect(t, o, Token::RParen);
        o.on_call(arity);
      }
      _ => {
        if is_stmt {
          o.on_stmt_expr();
        }
        return;
      }
    }
  }
}

fn parse_block<'a, O: Out>(t: &mut Lexer<'a>, o: &mut O) -> usize {
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
        let arity = parse_expr_list(t, o, Token::RBrace);
        o.on_break(arity);
        n_stmts += 1;
        expect(t, o, Token::RBrace);
        break;
      }
      Token::Let => {
        // TODO: multiple value bind
        t.next();
        let symbol = expect_symbol(t, o);
        expect(t, o, Token::Equal);
        parse_expr(t, o);
        o.on_let(symbol);
        n_stmts += 1;
      }
      Token::Return => {
        t.next();
        let arity = parse_expr_list(t, o, Token::RBrace);
        o.on_return(arity);
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
      _ => {
        // If we couldn't parse anything at all, then we end the block so that
        // we don't get stuck. Note that we already know that there ISN'T
        // an RBrace here, so the expect will fail.

        let pos = t.token_start();
        parse_prec(t, o, 0x00, true);
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


struct ToSexp(Vec<Sexp>);

pub fn parse_expr_sexp(source: &[u8]) -> Sexp {
  let mut o = ToSexp::new();
  parse_expr(&mut Lexer::new(source), &mut o);
  return o.pop();
}

impl ToSexp {
  fn new() -> Self {
    Self(Vec::new())
  }

  fn put(&mut self, x: Sexp) {
    self.0.push(x);
  }

  fn pop(&mut self) -> Sexp {
    return self.0.pop().unwrap();
  }

  fn pop_multi(&mut self, n: usize) -> impl Iterator<Item = Sexp> {
    return self.0.drain(self.0.len() - n ..);
  }
}

impl Out for ToSexp {
  fn on_variable(&mut self, x: &[u8]) {
    self.put(Sexp::from_bytes(x));
  }

  fn on_number(&mut self, x: &[u8]) {
    self.put(Sexp::from_bytes(x));
  }

  fn on_ternary(&mut self) {
    let y = self.pop();
    let x = self.pop();
    let p = self.pop();
    self.put(Sexp::from_array([Sexp::from_bytes(b"?:"), p, x, y]));
  }

  fn on_or(&mut self) {
    let y = self.pop();
    let x = self.pop();
    self.put(Sexp::from_array([Sexp::from_bytes(b"||"), x, y]));
  }

  fn on_and(&mut self) {
    let y = self.pop();
    let x = self.pop();
    self.put(Sexp::from_array([Sexp::from_bytes(b"&&"), x, y]));
  }

  fn on_op1(&mut self, op: Op1) {
    let x = self.pop();
    let op = Sexp::from_bytes(op.as_str().as_bytes());
    self.put(Sexp::from_array([op, x]));
  }

  fn on_op2(&mut self, op: Op2) {
    let y = self.pop();
    let x = self.pop();
    let op = Sexp::from_bytes(op.as_str().as_bytes());
    self.put(Sexp::from_array([op, x, y]));
  }

  fn on_field(&mut self, symbol: &[u8]) {
    let s = Sexp::from_bytes(format!(".{}", str::from_utf8(symbol).unwrap()).as_bytes());
    let x = self.pop();
    self.put(Sexp::from_array([s, x]));
  }

  fn on_index(&mut self) {
    let y = self.pop();
    let x = self.pop();
    self.put(Sexp::from_array([Sexp::from_bytes(b"[]"), x, y]));
  }

  fn on_call(&mut self, arity: usize) {
    let x = self.pop_multi(1 + arity).collect();
    self.put(Sexp::List(x));
  }

  fn on_loop(&mut self, n_stmts: usize) {
    let mut x = Vec::new();
    x.push(Sexp::from_bytes(b"loop"));
    x.extend(self.pop_multi(n_stmts));
    self.put(Sexp::List(x.into_boxed_slice()));
  }

  fn on_stmt_expr(&mut self) {
    let x = self.pop();
    self.put(Sexp::from_array([Sexp::from_bytes(b"$"), x]));
  }

  fn on_break(&mut self, arity: usize) {
    let mut x = Vec::new();
    x.push(Sexp::from_bytes(b"break"));
    x.extend(self.pop_multi(arity));
    self.put(Sexp::List(x.into_boxed_slice()));
  }

  fn on_let(&mut self, symbol: &[u8]) {
    let x = self.pop();
    let s = Sexp::from_bytes(symbol);
    self.put(Sexp::from_array([Sexp::from_bytes(b"let"), s, x]));
  }

  fn on_return(&mut self, arity: usize) {
    let mut x = Vec::new();
    x.push(Sexp::from_bytes(b"return"));
    x.extend(self.pop_multi(arity));
    self.put(Sexp::List(x.into_boxed_slice()));
  }

  fn on_set(&mut self, symbol: &[u8]) {
    let x = self.pop();
    let s = Sexp::from_bytes(symbol);
    self.put(Sexp::from_array([Sexp::from_bytes(b"<-"), s, x]));
  }

  fn on_set_field(&mut self, symbol: &[u8]) {
    let s = Sexp::from_bytes(format!(".{}<-", str::from_utf8(symbol).unwrap()).as_bytes());
    let y = self.pop();
    let x = self.pop();
    self.put(Sexp::from_array([s, x, y]));
  }

  fn on_set_index(&mut self) {
    let z = self.pop();
    let y = self.pop();
    let x = self.pop();
    self.put(Sexp::from_array([Sexp::from_bytes(b"[]<-"), x, y, z]));
  }

  fn on_var(&mut self, symbol: &[u8]) {
    let x = self.pop();
    let s = Sexp::from_bytes(symbol);
    self.put(Sexp::from_array([Sexp::from_bytes(b"var"), s, x]));
  }

  fn on_error_missing_expected_token(&mut self, _: Token) {
  }

  fn on_error_missing_expr(&mut self) {
    self.put(Sexp::from_bytes(b"undefined"));
  }
}
