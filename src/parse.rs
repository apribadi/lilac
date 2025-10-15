use crate::lexer::Lexer;
use crate::operator::Op1;
use crate::operator::Op2;
use crate::sexp::Sexp;
use crate::token::Token;

pub fn parse_expr<'a, V: Visitor>(t: &mut Lexer<'a>, v: &mut V) -> V::Expr {
  return parse_prec(t, v, 0);
}

pub trait Visitor {
  type Expr;

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

  fn visit_error_missing_expected_token(&mut self, token: Token);
}

fn expect<'a, V: Visitor>(t: &mut Lexer<'a>, v: &mut V, token: Token) {
  if t.token() == token {
    t.next();
  } else {
    v.visit_error_missing_expected_token(token);
  }
}

fn parse_prec<'a, V: Visitor>(t: &mut Lexer<'a>, v: &mut V, n: usize) -> V::Expr {
  let mut x =
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
        let y = parse_prec(t, v, 30);
        v.visit_op1(Op1::Neg, y)
      }
      Token::Not => {
        t.next();
        let y = parse_prec(t, v, 30);
        v.visit_op1(Op1::Not, y)
      }
      _ => {
        v.visit_error_missing_expr()
      }
    };

  loop {
    x =
      match t.token() {
        Token::Query if n <= 1 => {
          t.next();
          let y = parse_expr(t, v);
          expect(t, v, Token::Colon);
          let z = parse_prec(t, v, 0);
          v.visit_ternary(x, y, z)
        }
        Token::Or if n <= 6 => {
          t.next();
          let y = parse_prec(t, v, 7);
          v.visit_or(x, y)
        }
        Token::And if n <= 8 => {
          t.next();
          let y = parse_prec(t, v, 9);
          v.visit_and(x, y)
        }
        Token::CmpEq if n <= 10 => {
          t.next();
          let y = parse_prec(t, v, 11);
          v.visit_op2(Op2::CmpEq, x, y)
        }
        Token::CmpGe if n <= 10 => {
          t.next();
          let y = parse_prec(t, v, 11);
          v.visit_op2(Op2::CmpGe, x, y)
        }
        Token::CmpGt if n <= 10 => {
          t.next();
          let y = parse_prec(t, v, 11);
          v.visit_op2(Op2::CmpGt, x, y)
        }
        Token::CmpLe if n <= 10 => {
          t.next();
          let y = parse_prec(t, v, 11);
          v.visit_op2(Op2::CmpLe, x, y)
        }
        Token::CmpLt if n <= 10 => {
          t.next();
          let y = parse_prec(t, v, 11);
          v.visit_op2(Op2::CmpLt, x, y)
        }
        Token::CmpNe if n <= 10 => {
          t.next();
          let y = parse_prec(t, v, 11);
          v.visit_op2(Op2::CmpNe, x, y)
        }
        Token::BitOr if n <= 12 => {
          t.next();
          let y = parse_prec(t, v, 13);
          v.visit_op2(Op2::BitOr, x, y)
        }
        Token::BitXor if n <= 14 => {
          t.next();
          let y = parse_prec(t, v, 15);
          v.visit_op2(Op2::BitXor, x, y)
        }
        Token::BitAnd if n <= 16 => {
          t.next();
          let y = parse_prec(t, v, 17);
          v.visit_op2(Op2::BitAnd, x, y)
        }
        Token::Shl if n <= 18 => {
          t.next();
          let y = parse_prec(t, v, 19);
          v.visit_op2(Op2::Shl, x, y)
        }
        Token::Shr if n <= 18 => {
          t.next();
          let y = parse_prec(t, v, 19);
          v.visit_op2(Op2::Shr, x, y)
        }
        Token::Add if n <= 20 => {
          t.next();
          let y = parse_prec(t, v, 21);
          v.visit_op2(Op2::Add, x, y)
        }
        Token::Hyphen if n <= 20 => {
          t.next();
          let y = parse_prec(t, v, 21);
          v.visit_op2(Op2::Sub, x, y)
        }
        Token::Div if n <= 22 => {
          t.next();
          let y = parse_prec(t, v, 23);
          v.visit_op2(Op2::Div, x, y)
        }
        Token::Mul if n <= 22 => {
          t.next();
          let y = parse_prec(t, v, 23);
          v.visit_op2(Op2::Mul, x, y)
        }
        Token::Rem if n <= 22 => {
          t.next();
          let y = parse_prec(t, v, 23);
          v.visit_op2(Op2::Rem, x, y)
        }
        Token::Field if t.token_is_attached() && n <= 40 => {
          let s = t.token_span();
          t.next();
          v.visit_field(s, x)
        }
        Token::LBracket if t.token_is_attached() && n <= 40 => {
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

  fn visit_variable(&mut self, x: &[u8]) -> Self::Expr {
    Sexp::atom(x)
  }

  fn visit_number(&mut self, x: &[u8]) -> Self::Expr {
    Sexp::atom(x)
  }

  fn visit_ternary(&mut self, p: Sexp, x: Sexp, y: Sexp) -> Self::Expr {
    Sexp::from_array([Sexp::atom(b"?:"), p, x, y])
  }

  fn visit_or(&mut self, x: Sexp, y: Sexp) -> Self::Expr {
    Sexp::from_array([Sexp::atom(b"||"), x, y])
  }

  fn visit_and(&mut self, x: Sexp, y: Sexp) -> Self::Expr {
    Sexp::from_array([Sexp::atom(b"&&"), x, y])
  }

  fn visit_op1(&mut self, f: Op1, x: Sexp) -> Self::Expr {
    Sexp::from_array([Sexp::atom(f.as_str().as_bytes()), x])
  }

  fn visit_op2(&mut self, f: Op2, x: Sexp, y: Sexp) -> Self::Expr {
    Sexp::from_array([Sexp::atom(f.as_str().as_bytes()), x, y])
  }

  fn visit_field(&mut self, f: &[u8], x: Sexp) -> Self::Expr {
    Sexp::from_array([Sexp::atom(f), x])
  }

  fn visit_index(&mut self, x: Sexp, i: Sexp) -> Self::Expr {
    Sexp::from_array([Sexp::atom(b"[]"), x, i])
  }

  fn visit_error_missing_expr(&mut self) -> Self::Expr {
    Sexp::atom(b"undefined")
  }

  fn visit_error_missing_expected_token(&mut self, _: Token) {
  }
}
