use crate::lexer::Lexer;
use crate::op1::Op1;
use crate::op2::Op2;
use crate::sexp::Sexp;
use crate::token::Token;

pub trait Emit {
  fn emit_variable(&mut self, symbol: &[u8]);

  fn emit_number(&mut self, number: &[u8]);

  fn emit_ternary(&mut self);

  fn emit_or(&mut self);

  fn emit_and(&mut self);

  fn emit_op1(&mut self, op: Op1);

  fn emit_op2(&mut self, op: Op2);

  fn emit_field(&mut self, symbol: &[u8]);

  fn emit_index(&mut self);

  fn emit_call(&mut self, arity: usize);

  fn emit_let(&mut self, symbol: &[u8]);

  fn emit_stmt_expr(&mut self);

  fn emit_error_missing_expected_token(&mut self, token: Token);

  fn emit_error_missing_expr(&mut self);
}

pub fn parse_expr<'a, E: Emit>(t: &mut Lexer<'a>, o: &mut E) {
  return parse_prec(t, o, 0x00);
}

pub fn parse_stmt<'a, E: Emit>(t: &mut Lexer<'a>, o: &mut E) {
  match t.token() {
    Token::Let => {
      // TODO: multiple value bind
      t.next();
      let symbol = expect_symbol(t, o);
      expect(t, o, Token::Equal);
      parse_expr(t, o);
      o.emit_let(symbol);
    }
    _ => {
      parse_expr(t, o);
      o.emit_stmt_expr();
    }
  }
}

fn expect<'a, E: Emit>(t: &mut Lexer<'a>, o: &mut E, token: Token) {
  if t.token() != token {
    o.emit_error_missing_expected_token(token);
  } else {
    t.next();
  }
}

fn expect_symbol<'a, E: Emit>(t: &mut Lexer<'a>, o: &mut E) -> &'a [u8] {
  if t.token() != Token::Symbol {
    o.emit_error_missing_expected_token(Token::Symbol);
    return b"!!!";
  } else {
    let symbol = t.token_span();
    t.next();
    return symbol;
  }
}

fn parse_prec<'a, E: Emit>(t: &mut Lexer<'a>, o: &mut E, n: usize) {
  // TODO: parse black structured expressions, like
  //
  //   [expr] = if (...) { ... } else { ... }
  //
  // that start with a keyword

  match t.token() {
    Token::LParen => {
      t.next();
      parse_expr(t, o);
      expect(t, o, Token::RParen);
    }
    Token::Number => {
      let number = t.token_span();
      t.next();
      o.emit_number(number);
    }
    Token::Symbol => {
      let symbol = t.token_span();
      t.next();
      o.emit_variable(symbol);
    }
    Token::Hyphen => {
      t.next();
      parse_prec(t, o, 0xff);
      o.emit_op1(Op1::Neg);
    }
    Token::Not => {
      t.next();
      parse_prec(t, o, 0xff);
      o.emit_op1(Op1::Not);
    }
    _ => {
      o.emit_error_missing_expr();
    }
  }

  loop {
    match t.token() {
      Token::Query if n <= 0x10 => {
        t.next();
        parse_expr(t, o);
        expect(t, o, Token::Colon);
        parse_prec(t, o, 0x10);
        o.emit_ternary();
      }
      Token::Or if n <= 0x20 => {
        t.next();
        parse_prec(t, o, 0x21);
        o.emit_or();
      }
      Token::And if n <= 0x30 => {
        t.next();
        parse_prec(t, o, 0x31);
        o.emit_and();
      }
      Token::CmpEq if n <= 0x40 => {
        t.next();
        parse_prec(t, o, 0x41);
        o.emit_op2(Op2::CmpEq);
      }
      Token::CmpGe if n <= 0x40 => {
        t.next();
        parse_prec(t, o, 0x41);
        o.emit_op2(Op2::CmpGe);
      }
      Token::CmpGt if n <= 0x40 => {
        t.next();
        parse_prec(t, o, 0x41);
        o.emit_op2(Op2::CmpGt);
      }
      Token::CmpLe if n <= 0x40 => {
        t.next();
        parse_prec(t, o, 0x41);
        o.emit_op2(Op2::CmpLe);
      }
      Token::CmpLt if n <= 0x40 => {
        t.next();
        parse_prec(t, o, 0x41);
        o.emit_op2(Op2::CmpLt);
      }
      Token::CmpNe if n <= 0x40 => {
        t.next();
        parse_prec(t, o, 0x41);
        o.emit_op2(Op2::CmpNe);
      }
      Token::BitOr if n <= 0x50 => {
        t.next();
        parse_prec(t, o, 0x51);
        o.emit_op2(Op2::BitOr);
      }
      Token::BitXor if n <= 0x60 => {
        t.next();
        parse_prec(t, o, 0x61);
        o.emit_op2(Op2::BitXor);
      }
      Token::BitAnd if n <= 0x70 => {
        t.next();
        parse_prec(t, o, 0x71);
        o.emit_op2(Op2::BitAnd);
      }
      Token::Shl if n <= 0x80 => {
        t.next();
        parse_prec(t, o, 0x81);
        o.emit_op2(Op2::Shl);
      }
      Token::Shr if n <= 0x80 => {
        t.next();
        parse_prec(t, o, 0x81);
        o.emit_op2(Op2::Shr);
      }
      Token::Add if n <= 0x90 => {
        t.next();
        parse_prec(t, o, 0x91);
        o.emit_op2(Op2::Add);
      }
      Token::Hyphen if n <= 0x90 => {
        t.next();
        parse_prec(t, o, 0x91);
        o.emit_op2(Op2::Sub);
      }
      Token::Div if n <= 0xA0 => {
        t.next();
        parse_prec(t, o, 0xA1);
        o.emit_op2(Op2::Div);
      }
      Token::Mul if n <= 0xA0 => {
        t.next();
        parse_prec(t, o, 0xA1);
        o.emit_op2(Op2::Mul);
      }
      Token::Rem if n <= 0xA0 => {
        t.next();
        parse_prec(t, o, 0xA1);
        o.emit_op2(Op2::Rem);
      }
      Token::Field if t.token_is_attached() => {
        let symbol = &t.token_span()[1 ..];
        t.next();
        o.emit_field(symbol);
      }
      Token::LBracket if t.token_is_attached() => {
        t.next();
        parse_expr(t, o);
        expect(t, o, Token::RBracket);
        o.emit_index();
      }
      Token::LParen if t.token_is_attached() => {
        t.next();
        if t.token() == Token::RParen {
          t.next();
          o.emit_call(0);
        } else {
          let mut arity = 0;
          loop {
            parse_expr(t, o);
            arity += 1;
            if t.token() != Token::Comma { break; }
            t.next();
          }
          expect(t, o, Token::RParen);
          o.emit_call(arity);
        }
      }
      _ => {
        return;
      }
    }
  }
}

struct EmitSexp(Vec<Sexp>);

pub fn parse_expr_sexp(source: &str) -> Sexp {
  let mut o = EmitSexp::new();
  parse_expr(&mut Lexer::new(source.as_bytes()), &mut o);
  return o.pop();
}

pub fn parse_stmt_sexp(source: &str) -> Sexp {
  let mut o = EmitSexp::new();
  parse_stmt(&mut Lexer::new(source.as_bytes()), &mut o);
  return o.pop();
}

impl EmitSexp {
  fn new() -> Self {
    Self(Vec::new())
  }

  fn put(&mut self, x: Sexp) {
    self.0.push(x);
  }

  fn pop(&mut self) -> Sexp {
    return self.0.pop().unwrap();
  }

  fn pop_multi(&mut self, n: usize) -> Box<[Sexp]> {
    return self.0.split_off(self.0.len() - n).into_boxed_slice();
  }
}

impl Emit for EmitSexp {
  fn emit_variable(&mut self, x: &[u8]) {
    self.put(Sexp::from_bytes(x));
  }

  fn emit_number(&mut self, x: &[u8]) {
    self.put(Sexp::from_bytes(x));
  }

  fn emit_ternary(&mut self) {
    let y = self.pop();
    let x = self.pop();
    let p = self.pop();
    self.put(Sexp::from_array([Sexp::from_bytes(b"?:"), p, x, y]));
  }

  fn emit_or(&mut self) {
    let y = self.pop();
    let x = self.pop();
    self.put(Sexp::from_array([Sexp::from_bytes(b"||"), x, y]));
  }

  fn emit_and(&mut self) {
    let y = self.pop();
    let x = self.pop();
    self.put(Sexp::from_array([Sexp::from_bytes(b"&&"), x, y]));
  }

  fn emit_op1(&mut self, op: Op1) {
    let x = self.pop();
    self.put(Sexp::from_array([Sexp::from_bytes(op.as_str().as_bytes()), x]));
  }

  fn emit_op2(&mut self, op: Op2) {
    let y = self.pop();
    let x = self.pop();
    self.put(Sexp::from_array([Sexp::from_bytes(op.as_str().as_bytes()), x, y]));
  }

  fn emit_field(&mut self, symbol: &[u8]) {
    let x = self.pop();
    self.put(Sexp::from_array([Sexp::from_bytes(symbol), x]));
  }

  fn emit_index(&mut self) {
    let i = self.pop();
    let x = self.pop();
    self.put(Sexp::from_array([Sexp::from_bytes(b"[]"), x, i]));
  }

  fn emit_call(&mut self, arity: usize) {
    let x = self.pop_multi(1 + arity);
    self.put(Sexp::List(x));
  }

  fn emit_let(&mut self, symbol: &[u8]) {
    let x = self.pop();
    self.put(Sexp::from_array([Sexp::from_bytes(b"let"), Sexp::from_bytes(symbol), x]));
  }

  fn emit_stmt_expr(&mut self) {
    // put(pop())
  }

  fn emit_error_missing_expected_token(&mut self, _: Token) {
  }

  fn emit_error_missing_expr(&mut self) {
    self.put(Sexp::from_bytes(b"undefined"));
  }
}
