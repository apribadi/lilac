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

pub fn parse_expr<'a, E: Emit>(t: &mut Lexer<'a>, e: &mut E) {
  return parse_prec(t, e, 0x00);
}

pub fn parse_stmt<'a, E: Emit>(t: &mut Lexer<'a>, e: &mut E) {
  match t.token() {
    Token::Let => {
      // TODO: multiple value bind
      t.next();
      let symbol = expect_symbol(t, e);
      expect(t, e, Token::Equal);
      parse_expr(t, e);
      e.emit_let(symbol);
    }
    _ => {
      parse_expr(t, e);
      e.emit_stmt_expr();
    }
  }
}

fn expect<'a, E: Emit>(t: &mut Lexer<'a>, e: &mut E, token: Token) {
  if t.token() != token {
    e.emit_error_missing_expected_token(token);
  } else {
    t.next();
  }
}

fn expect_symbol<'a, E: Emit>(t: &mut Lexer<'a>, e: &mut E) -> &'a [u8] {
  if t.token() != Token::Symbol {
    e.emit_error_missing_expected_token(Token::Symbol);
    return b"!!!";
  } else {
    let symbol = t.token_span();
    t.next();
    return symbol;
  }
}

fn parse_prec<'a, E: Emit>(t: &mut Lexer<'a>, e: &mut E, n: usize) {
  // TODO: parse black structured expressions, like
  //
  //   [expr] = if (...) { ... } else { ... }
  //
  // that start with a keyword

  match t.token() {
    Token::LParen => {
      t.next();
      parse_expr(t, e);
      expect(t, e, Token::RParen);
    }
    Token::Number => {
      let number = t.token_span();
      t.next();
      e.emit_number(number);
    }
    Token::Symbol => {
      let symbol = t.token_span();
      t.next();
      e.emit_variable(symbol);
    }
    Token::Hyphen => {
      t.next();
      parse_prec(t, e, 0xff);
      e.emit_op1(Op1::Neg);
    }
    Token::Not => {
      t.next();
      parse_prec(t, e, 0xff);
      e.emit_op1(Op1::Not);
    }
    _ => {
      e.emit_error_missing_expr();
    }
  }

  loop {
    match t.token() {
      Token::Query if n <= 0x10 => {
        t.next();
        parse_expr(t, e);
        expect(t, e, Token::Colon);
        parse_prec(t, e, 0x10);
        e.emit_ternary();
      }
      Token::Or if n <= 0x20 => {
        t.next();
        parse_prec(t, e, 0x21);
        e.emit_or();
      }
      Token::And if n <= 0x30 => {
        t.next();
        parse_prec(t, e, 0x31);
        e.emit_and();
      }
      Token::CmpEq if n <= 0x40 => {
        t.next();
        parse_prec(t, e, 0x41);
        e.emit_op2(Op2::CmpEq);
      }
      Token::CmpGe if n <= 0x40 => {
        t.next();
        parse_prec(t, e, 0x41);
        e.emit_op2(Op2::CmpGe);
      }
      Token::CmpGt if n <= 0x40 => {
        t.next();
        parse_prec(t, e, 0x41);
        e.emit_op2(Op2::CmpGt);
      }
      Token::CmpLe if n <= 0x40 => {
        t.next();
        parse_prec(t, e, 0x41);
        e.emit_op2(Op2::CmpLe);
      }
      Token::CmpLt if n <= 0x40 => {
        t.next();
        parse_prec(t, e, 0x41);
        e.emit_op2(Op2::CmpLt);
      }
      Token::CmpNe if n <= 0x40 => {
        t.next();
        parse_prec(t, e, 0x41);
        e.emit_op2(Op2::CmpNe);
      }
      Token::BitOr if n <= 0x50 => {
        t.next();
        parse_prec(t, e, 0x51);
        e.emit_op2(Op2::BitOr);
      }
      Token::BitXor if n <= 0x60 => {
        t.next();
        parse_prec(t, e, 0x61);
        e.emit_op2(Op2::BitXor);
      }
      Token::BitAnd if n <= 0x70 => {
        t.next();
        parse_prec(t, e, 0x71);
        e.emit_op2(Op2::BitAnd);
      }
      Token::Shl if n <= 0x80 => {
        t.next();
        parse_prec(t, e, 0x81);
        e.emit_op2(Op2::Shl);
      }
      Token::Shr if n <= 0x80 => {
        t.next();
        parse_prec(t, e, 0x81);
        e.emit_op2(Op2::Shr);
      }
      Token::Add if n <= 0x90 => {
        t.next();
        parse_prec(t, e, 0x91);
        e.emit_op2(Op2::Add);
      }
      Token::Hyphen if n <= 0x90 => {
        t.next();
        parse_prec(t, e, 0x91);
        e.emit_op2(Op2::Sub);
      }
      Token::Div if n <= 0xA0 => {
        t.next();
        parse_prec(t, e, 0xA1);
        e.emit_op2(Op2::Div);
      }
      Token::Mul if n <= 0xA0 => {
        t.next();
        parse_prec(t, e, 0xA1);
        e.emit_op2(Op2::Mul);
      }
      Token::Rem if n <= 0xA0 => {
        t.next();
        parse_prec(t, e, 0xA1);
        e.emit_op2(Op2::Rem);
      }
      Token::Field if t.token_is_attached() => {
        let symbol = &t.token_span()[1 ..];
        t.next();
        e.emit_field(symbol);
      }
      Token::LBracket if t.token_is_attached() => {
        t.next();
        parse_expr(t, e);
        expect(t, e, Token::RBracket);
        e.emit_index();
      }
      Token::LParen if t.token_is_attached() => {
        t.next();
        if t.token() == Token::RParen {
          t.next();
          e.emit_call(0);
        } else {
          let mut arity = 0;
          loop {
            parse_expr(t, e);
            arity += 1;
            if t.token() != Token::Comma { break; }
            t.next();
          }
          expect(t, e, Token::RParen);
          e.emit_call(arity);
        }
      }
      _ => {
        return;
      }
    }
  }
}

struct EmitSexp {
  stack: Vec<Sexp>,
}

pub fn parse_expr_sexp(source: &str) -> Sexp {
  let mut e = EmitSexp::new();
  parse_expr(&mut Lexer::new(source.as_bytes()), &mut e);
  return e.pop();
}

pub fn parse_stmt_sexp(source: &str) -> Sexp {
  let mut e = EmitSexp::new();
  parse_stmt(&mut Lexer::new(source.as_bytes()), &mut e);
  return e.pop();
}

impl EmitSexp {
  fn new() -> Self {
    Self { stack: Vec::new() }
  }

  fn push(&mut self, x: Sexp) {
    self.stack.push(x);
  }

  fn pop(&mut self) -> Sexp {
    return self.stack.pop().unwrap();
  }

  fn pop_multi(&mut self, n: usize) -> Vec<Sexp> {
    return self.stack.split_off(self.stack.len() - n);
  }
}

impl Emit for EmitSexp {
  fn emit_variable(&mut self, x: &[u8]) {
    self.push(Sexp::atom(x));
  }

  fn emit_number(&mut self, x: &[u8]) {
    self.push(Sexp::atom(x));
  }

  fn emit_ternary(&mut self) {
    let y = self.pop();
    let x = self.pop();
    let p = self.pop();
    self.push(Sexp::from_array([Sexp::atom(b"?:"), p, x, y]));
  }

  fn emit_or(&mut self) {
    let y = self.pop();
    let x = self.pop();
    self.push(Sexp::from_array([Sexp::atom(b"||"), x, y]));
  }

  fn emit_and(&mut self) {
    let y = self.pop();
    let x = self.pop();
    self.push(Sexp::from_array([Sexp::atom(b"&&"), x, y]));
  }

  fn emit_op1(&mut self, op: Op1) {
    let x = self.pop();
    self.push(Sexp::from_array([Sexp::atom(op.as_str().as_bytes()), x]));
  }

  fn emit_op2(&mut self, op: Op2) {
    let y = self.pop();
    let x = self.pop();
    self.push(Sexp::from_array([Sexp::atom(op.as_str().as_bytes()), x, y]));
  }

  fn emit_field(&mut self, symbol: &[u8]) {
    let x = self.pop();
    self.push(Sexp::from_array([Sexp::atom(symbol), x]));
  }

  fn emit_index(&mut self) {
    let i = self.pop();
    let x = self.pop();
    self.push(Sexp::from_array([Sexp::atom(b"[]"), x, i]));
  }

  fn emit_call(&mut self, arity: usize) {
    let x = self.pop_multi(1 + arity).into_boxed_slice();
    self.push(Sexp::List(x));
  }

  fn emit_let(&mut self, symbol: &[u8]) {
    let x = self.pop();
    self.push(Sexp::from_array([Sexp::atom(b"let"), Sexp::atom(symbol), Sexp::atom(b"="), x]))
  }

  fn emit_stmt_expr(&mut self) {
    // push(pop())
  }

  fn emit_error_missing_expected_token(&mut self, _: Token) {
  }

  fn emit_error_missing_expr(&mut self) {
    self.push(Sexp::atom(b"undefined"));
  }
}
