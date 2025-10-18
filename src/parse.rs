use crate::lexer::Lexer;
use crate::op1::Op1;
use crate::op2::Op2;
use crate::sexp::Sexp;
use crate::token::Token;

pub trait Emit {
  type Expr;

  type Stmt;

  fn emit_variable(&mut self, x: &[u8]) -> Self::Expr;

  fn emit_number(&mut self, x: &[u8]) -> Self::Expr;

  fn emit_ternary(&mut self, p: Self::Expr, x: Self::Expr, y: Self::Expr) -> Self::Expr;

  fn emit_or(&mut self, x: Self::Expr, y: Self::Expr) -> Self::Expr;

  fn emit_and(&mut self, x: Self::Expr, y: Self::Expr) -> Self::Expr;

  fn emit_op1(&mut self, f: Op1, x: Self::Expr) -> Self::Expr;

  fn emit_op2(&mut self, f: Op2, x: Self::Expr, y: Self::Expr) -> Self::Expr;

  fn emit_field(&mut self, f: &[u8], x: Self::Expr) -> Self::Expr;

  fn emit_index(&mut self, x: Self::Expr, i: Self::Expr) -> Self::Expr;

  fn emit_error_missing_expr(&mut self) -> Self::Expr;

  fn emit_let(&mut self, s: &[u8], x: Self::Expr) -> Self::Stmt;

  fn emit_stmt_expr(&mut self, x: Self::Expr) -> Self::Stmt;

  fn emit_error_missing_expected_token(&mut self, token: Token);
}

pub fn parse_expr<'a, E: Emit>(t: &mut Lexer<'a>, e: &mut E) -> E::Expr {
  return parse_prec(t, e, 0x00);
}

pub fn parse_stmt<'a, E: Emit>(t: &mut Lexer<'a>, e: &mut E) -> E::Stmt {
  match t.token() {
    Token::Let => {
      // TODO: multiple value bind
      t.next();
      let s = expect_symbol(t, e);
      expect(t, e, Token::Equal);
      let x = parse_expr(t, e);
      e.emit_let(s, x)
    }
    _ => {
      let x = parse_expr(t, e);
      e.emit_stmt_expr(x)
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
    let s = t.token_span();
    t.next();
    return s;
  }
}

fn parse_prec<'a, E: Emit>(t: &mut Lexer<'a>, e: &mut E, n: usize) -> E::Expr {
  let mut x =
    // TODO: parse black structured expressions, like
    //
    //   [expr] = if (...) { ... } else { ... }
    //
    // that start with a keyword

    match t.token() {
      Token::LParen => {
        t.next();
        let y = parse_expr(t, e);
        expect(t, e, Token::RParen);
        y
      }
      Token::Number => {
        let s = t.token_span();
        t.next();
        e.emit_number(s)
      }
      Token::Symbol => {
        let s = t.token_span();
        t.next();
        e.emit_variable(s)
      }
      Token::Hyphen => {
        t.next();
        let y = parse_prec(t, e, 0xff);
        e.emit_op1(Op1::Neg, y)
      }
      Token::Not => {
        t.next();
        let y = parse_prec(t, e, 0xff);
        e.emit_op1(Op1::Not, y)
      }
      _ => {
        e.emit_error_missing_expr()
      }
    };

  loop {
    x =
      match t.token() {
        Token::Query if n <= 0x10 => {
          t.next();
          let y = parse_expr(t, e);
          expect(t, e, Token::Colon);
          let z = parse_prec(t, e, 0x10);
          e.emit_ternary(x, y, z)
        }
        Token::Or if n <= 0x20 => {
          t.next();
          let y = parse_prec(t, e, 0x21);
          e.emit_or(x, y)
        }
        Token::And if n <= 0x30 => {
          t.next();
          let y = parse_prec(t, e, 0x31);
          e.emit_and(x, y)
        }
        Token::CmpEq if n <= 0x40 => {
          t.next();
          let y = parse_prec(t, e, 0x41);
          e.emit_op2(Op2::CmpEq, x, y)
        }
        Token::CmpGe if n <= 0x40 => {
          t.next();
          let y = parse_prec(t, e, 0x41);
          e.emit_op2(Op2::CmpGe, x, y)
        }
        Token::CmpGt if n <= 0x40 => {
          t.next();
          let y = parse_prec(t, e, 0x41);
          e.emit_op2(Op2::CmpGt, x, y)
        }
        Token::CmpLe if n <= 0x40 => {
          t.next();
          let y = parse_prec(t, e, 0x41);
          e.emit_op2(Op2::CmpLe, x, y)
        }
        Token::CmpLt if n <= 0x40 => {
          t.next();
          let y = parse_prec(t, e, 0x41);
          e.emit_op2(Op2::CmpLt, x, y)
        }
        Token::CmpNe if n <= 0x40 => {
          t.next();
          let y = parse_prec(t, e, 0x41);
          e.emit_op2(Op2::CmpNe, x, y)
        }
        Token::BitOr if n <= 0x50 => {
          t.next();
          let y = parse_prec(t, e, 0x51);
          e.emit_op2(Op2::BitOr, x, y)
        }
        Token::BitXor if n <= 0x60 => {
          t.next();
          let y = parse_prec(t, e, 0x61);
          e.emit_op2(Op2::BitXor, x, y)
        }
        Token::BitAnd if n <= 0x70 => {
          t.next();
          let y = parse_prec(t, e, 0x71);
          e.emit_op2(Op2::BitAnd, x, y)
        }
        Token::Shl if n <= 0x80 => {
          t.next();
          let y = parse_prec(t, e, 0x81);
          e.emit_op2(Op2::Shl, x, y)
        }
        Token::Shr if n <= 0x80 => {
          t.next();
          let y = parse_prec(t, e, 0x81);
          e.emit_op2(Op2::Shr, x, y)
        }
        Token::Add if n <= 0x90 => {
          t.next();
          let y = parse_prec(t, e, 0x91);
          e.emit_op2(Op2::Add, x, y)
        }
        Token::Hyphen if n <= 0x90 => {
          t.next();
          let y = parse_prec(t, e, 0x91);
          e.emit_op2(Op2::Sub, x, y)
        }
        Token::Div if n <= 0xA0 => {
          t.next();
          let y = parse_prec(t, e, 0xA1);
          e.emit_op2(Op2::Div, x, y)
        }
        Token::Mul if n <= 0xA0 => {
          t.next();
          let y = parse_prec(t, e, 0xA1);
          e.emit_op2(Op2::Mul, x, y)
        }
        Token::Rem if n <= 0xA0 => {
          t.next();
          let y = parse_prec(t, e, 0xA1);
          e.emit_op2(Op2::Rem, x, y)
        }
        Token::Field if t.token_is_attached() => {
          let s = t.token_span();
          t.next();
          e.emit_field(s, x)
        }
        Token::LBracket if t.token_is_attached() => {
          t.next();
          let i = parse_expr(t, e);
          expect(t, e, Token::RBracket);
          e.emit_index(x, i)
        }
        // TODO: parse function application
        _ => {
          return x;
        }
      };
  }
}

pub struct EmitSexp;

impl Emit for EmitSexp {
  type Expr = Sexp;

  type Stmt = Sexp;

  fn emit_variable(&mut self, x: &[u8]) -> Sexp {
    Sexp::atom(x)
  }

  fn emit_number(&mut self, x: &[u8]) -> Sexp {
    Sexp::atom(x)
  }

  fn emit_ternary(&mut self, p: Sexp, x: Sexp, y: Sexp) -> Sexp {
    Sexp::from_array([Sexp::atom(b"?:"), p, x, y])
  }

  fn emit_or(&mut self, x: Sexp, y: Sexp) -> Sexp {
    Sexp::from_array([Sexp::atom(b"||"), x, y])
  }

  fn emit_and(&mut self, x: Sexp, y: Sexp) -> Sexp {
    Sexp::from_array([Sexp::atom(b"&&"), x, y])
  }

  fn emit_op1(&mut self, f: Op1, x: Sexp) -> Sexp {
    Sexp::from_array([Sexp::atom(f.as_str().as_bytes()), x])
  }

  fn emit_op2(&mut self, f: Op2, x: Sexp, y: Sexp) -> Sexp {
    Sexp::from_array([Sexp::atom(f.as_str().as_bytes()), x, y])
  }

  fn emit_field(&mut self, f: &[u8], x: Sexp) -> Sexp {
    Sexp::from_array([Sexp::atom(f), x])
  }

  fn emit_index(&mut self, x: Sexp, i: Sexp) -> Sexp {
    Sexp::from_array([Sexp::atom(b"[]"), x, i])
  }

  fn emit_error_missing_expr(&mut self) -> Sexp {
    Sexp::atom(b"undefined")
  }

  fn emit_let(&mut self, s: &[u8], x: Sexp) -> Sexp {
    Sexp::from_array([Sexp::atom(b"let"), Sexp::atom(s), Sexp::atom(b"="), x])
  }

  fn emit_stmt_expr(&mut self, x: Sexp) -> Sexp {
    x
  }

  fn emit_error_missing_expected_token(&mut self, _: Token) {
  }
}
