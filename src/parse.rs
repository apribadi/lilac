use crate::lexer::Lexer;
use crate::operator::Op1;
use crate::operator::Op2;
use crate::sexp::Sexp;
use crate::token::Token;

pub trait Visitor {
  type Expr;

  type Stmt;

  fn visit_variable(&mut self, x: &[u8]) -> Self::Expr;

  fn visit_number(&mut self, x: &[u8]) -> Self::Expr;

  fn visit_ternary(&mut self, p: Self::Expr, x: Self::Expr, y: Self::Expr) -> Self::Expr;

  fn visit_or(&mut self, x: Self::Expr, y: Self::Expr) -> Self::Expr;

  fn visit_and(&mut self, x: Self::Expr, y: Self::Expr) -> Self::Expr;

  fn visit_op1(&mut self, f: Op1, x: Self::Expr) -> Self::Expr;

  fn visit_op2(&mut self, f: Op2, x: Self::Expr, y: Self::Expr) -> Self::Expr;

  fn visit_field(&mut self, f: &[u8], x: Self::Expr) -> Self::Expr;

  fn visit_index(&mut self, x: Self::Expr, i: Self::Expr) -> Self::Expr;

  fn visit_error_missing_expr(&mut self) -> Self::Expr;

  fn visit_let(&mut self, s: &[u8], x: Self::Expr) -> Self::Stmt;

  fn visit_stmt_expr(&mut self, x: Self::Expr) -> Self::Stmt;

  fn visit_error_missing_expected_token(&mut self, token: Token);
}

pub fn parse_expr<'a, V: Visitor>(t: &mut Lexer<'a>, v: &mut V) -> V::Expr {
  return parse_prec(t, v, 0x00);
}

pub fn parse_stmt<'a, V: Visitor>(t: &mut Lexer<'a>, v: &mut V) -> V::Stmt {
  match t.token() {
    Token::Let => {
      // TODO: multiple value bind
      t.next();
      let s = expect_symbol(t, v);
      expect(t, v, Token::Equal);
      let x = parse_expr(t, v);
      v.visit_let(s, x)
    }
    _ => {
      let x = parse_expr(t, v);
      v.visit_stmt_expr(x)
    }
  }
}

fn expect<'a, V: Visitor>(t: &mut Lexer<'a>, v: &mut V, token: Token) {
  if t.token() == token {
    t.next();
  } else {
    v.visit_error_missing_expected_token(token);
  }
}

fn expect_symbol<'a, V: Visitor>(t: &mut Lexer<'a>, v: &mut V) -> &'a [u8] {
  if t.token() == Token::Symbol {
    let s = t.token_span();
    t.next();
    s
  } else {
    v.visit_error_missing_expected_token(Token::Symbol);
    b"!!!"
  }
}

fn parse_prec<'a, V: Visitor>(t: &mut Lexer<'a>, v: &mut V, n: usize) -> V::Expr {
  let mut x =
    // TODO: parse black structured expressions, like
    //
    //   [expr] = if (...) { ... } else { ... }
    //
    // that start with a keyword

    match t.token() {
      Token::LParen => {
        t.next();
        let y = parse_expr(t, v);
        expect(t, v, Token::RParen);
        y
      }
      Token::Number => {
        let s = t.token_span();
        t.next();
        v.visit_number(s)
      }
      Token::Symbol => {
        let s = t.token_span();
        t.next();
        v.visit_variable(s)
      }
      Token::Hyphen => {
        t.next();
        let y = parse_prec(t, v, 0xff);
        v.visit_op1(Op1::Neg, y)
      }
      Token::Not => {
        t.next();
        let y = parse_prec(t, v, 0xff);
        v.visit_op1(Op1::Not, y)
      }
      _ => {
        v.visit_error_missing_expr()
      }
    };

  loop {
    x =
      match t.token() {
        Token::Query if n <= 0x10 => {
          t.next();
          let y = parse_expr(t, v);
          expect(t, v, Token::Colon);
          let z = parse_prec(t, v, 0x10);
          v.visit_ternary(x, y, z)
        }
        Token::Or if n <= 0x20 => {
          t.next();
          let y = parse_prec(t, v, 0x21);
          v.visit_or(x, y)
        }
        Token::And if n <= 0x30 => {
          t.next();
          let y = parse_prec(t, v, 0x31);
          v.visit_and(x, y)
        }
        Token::CmpEq if n <= 0x40 => {
          t.next();
          let y = parse_prec(t, v, 0x41);
          v.visit_op2(Op2::CmpEq, x, y)
        }
        Token::CmpGe if n <= 0x40 => {
          t.next();
          let y = parse_prec(t, v, 0x41);
          v.visit_op2(Op2::CmpGe, x, y)
        }
        Token::CmpGt if n <= 0x40 => {
          t.next();
          let y = parse_prec(t, v, 0x41);
          v.visit_op2(Op2::CmpGt, x, y)
        }
        Token::CmpLe if n <= 0x40 => {
          t.next();
          let y = parse_prec(t, v, 0x41);
          v.visit_op2(Op2::CmpLe, x, y)
        }
        Token::CmpLt if n <= 0x40 => {
          t.next();
          let y = parse_prec(t, v, 0x41);
          v.visit_op2(Op2::CmpLt, x, y)
        }
        Token::CmpNe if n <= 0x40 => {
          t.next();
          let y = parse_prec(t, v, 0x41);
          v.visit_op2(Op2::CmpNe, x, y)
        }
        Token::BitOr if n <= 0x50 => {
          t.next();
          let y = parse_prec(t, v, 0x51);
          v.visit_op2(Op2::BitOr, x, y)
        }
        Token::BitXor if n <= 0x60 => {
          t.next();
          let y = parse_prec(t, v, 0x61);
          v.visit_op2(Op2::BitXor, x, y)
        }
        Token::BitAnd if n <= 0x70 => {
          t.next();
          let y = parse_prec(t, v, 0x71);
          v.visit_op2(Op2::BitAnd, x, y)
        }
        Token::Shl if n <= 0x80 => {
          t.next();
          let y = parse_prec(t, v, 0x81);
          v.visit_op2(Op2::Shl, x, y)
        }
        Token::Shr if n <= 0x80 => {
          t.next();
          let y = parse_prec(t, v, 0x81);
          v.visit_op2(Op2::Shr, x, y)
        }
        Token::Add if n <= 0x90 => {
          t.next();
          let y = parse_prec(t, v, 0x91);
          v.visit_op2(Op2::Add, x, y)
        }
        Token::Hyphen if n <= 0x90 => {
          t.next();
          let y = parse_prec(t, v, 0x91);
          v.visit_op2(Op2::Sub, x, y)
        }
        Token::Div if n <= 0xA0 => {
          t.next();
          let y = parse_prec(t, v, 0xA1);
          v.visit_op2(Op2::Div, x, y)
        }
        Token::Mul if n <= 0xA0 => {
          t.next();
          let y = parse_prec(t, v, 0xA1);
          v.visit_op2(Op2::Mul, x, y)
        }
        Token::Rem if n <= 0xA0 => {
          t.next();
          let y = parse_prec(t, v, 0xA1);
          v.visit_op2(Op2::Rem, x, y)
        }
        Token::Field if t.token_is_attached() => {
          let s = t.token_span();
          t.next();
          v.visit_field(s, x)
        }
        Token::LBracket if t.token_is_attached() => {
          t.next();
          let i = parse_expr(t, v);
          expect(t, v, Token::RBracket);
          v.visit_index(x, i)
        }
        _ => {
          return x;
        }
      };
  }
}

pub struct DumpSexp;

impl Visitor for DumpSexp {
  type Expr = Sexp;

  type Stmt = Sexp;

  fn visit_variable(&mut self, x: &[u8]) -> Sexp {
    Sexp::atom(x)
  }

  fn visit_number(&mut self, x: &[u8]) -> Sexp {
    Sexp::atom(x)
  }

  fn visit_ternary(&mut self, p: Sexp, x: Sexp, y: Sexp) -> Sexp {
    Sexp::from_array([Sexp::atom(b"?:"), p, x, y])
  }

  fn visit_or(&mut self, x: Sexp, y: Sexp) -> Sexp {
    Sexp::from_array([Sexp::atom(b"||"), x, y])
  }

  fn visit_and(&mut self, x: Sexp, y: Sexp) -> Sexp {
    Sexp::from_array([Sexp::atom(b"&&"), x, y])
  }

  fn visit_op1(&mut self, f: Op1, x: Sexp) -> Sexp {
    Sexp::from_array([Sexp::atom(f.as_str().as_bytes()), x])
  }

  fn visit_op2(&mut self, f: Op2, x: Sexp, y: Sexp) -> Sexp {
    Sexp::from_array([Sexp::atom(f.as_str().as_bytes()), x, y])
  }

  fn visit_field(&mut self, f: &[u8], x: Sexp) -> Sexp {
    Sexp::from_array([Sexp::atom(f), x])
  }

  fn visit_index(&mut self, x: Sexp, i: Sexp) -> Sexp {
    Sexp::from_array([Sexp::atom(b"[]"), x, i])
  }

  fn visit_error_missing_expr(&mut self) -> Sexp {
    Sexp::atom(b"undefined")
  }

  fn visit_let(&mut self, s: &[u8], x: Sexp) -> Sexp {
    Sexp::from_array([Sexp::atom(b"let"), Sexp::atom(s), Sexp::atom(b"="), x])
  }

  fn visit_stmt_expr(&mut self, x: Sexp) -> Sexp {
    x
  }

  fn visit_error_missing_expected_token(&mut self, _: Token) {
  }
}
