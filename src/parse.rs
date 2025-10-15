use crate::lex::Lexer;
use crate::token::Token;
use crate::ir1::Inst;
use crate::sexp::Sexp;
use crate::operator::Op1;
use crate::operator::Op2;

pub fn parse<T: Visitor>(source: &[u8], visitor: &mut T) -> T::Expr {
  return Parser::new(source).parse_expr(visitor);
}

pub trait Visitor {
  type Expr;

  fn visit_undefined(&mut self) -> Self::Expr;

  fn visit_variable(&mut self, x: &[u8]) -> Self::Expr;

  fn visit_number(&mut self, x: &[u8]) -> Self::Expr;

  fn visit_ternary(&mut self, p: Self::Expr, x: Self::Expr, y: Self::Expr) -> Self::Expr;

  fn visit_or(&mut self, x: Self::Expr, y: Self::Expr) -> Self::Expr;

  fn visit_and(&mut self, x: Self::Expr, y: Self::Expr) -> Self::Expr;

  fn visit_op1(&mut self, f: Op1, x: Self::Expr) -> Self::Expr;

  fn visit_op2(&mut self, f: Op2, x: Self::Expr, y: Self::Expr) -> Self::Expr;

  fn visit_field(&mut self, f: &[u8], x: Self::Expr) -> Self::Expr;

  fn visit_index(&mut self, x: Self::Expr, i: Self::Expr) -> Self::Expr;
}

pub struct Parser<'a> {
  lexer: Lexer<'a>,
}

impl<'a> Parser<'a> {
  pub fn new(source: &'a [u8]) -> Self {
    Self {
      lexer: Lexer::new(source),
    }
  }

  fn token(&self) -> Token {
    return self.lexer.token();
  }

  fn token_is_attached(&self) -> bool {
    return self.lexer.token_is_attached();
  }

  fn next(&mut self) {
    self.lexer.next();
  }

  fn next_span(&mut self) -> &'a [u8] {
    let s = self.lexer.token_span();
    self.lexer.next();
    s
  }

  fn expect(&mut self, t: Token) {
    if self.lexer.token() == t {
      self.lexer.next()
    }

    // else error
  }

  fn parse_expr<T: Visitor>(&mut self, visitor: &mut T) -> T::Expr {
    return self.parse_prec(visitor, 0);
  }

  fn parse_prec<T: Visitor>(&mut self, visitor: &mut T, n: usize) -> T::Expr {
    let mut x =
      match self.token() {
        Token::LParen => {
          self.next();
          let y = self.parse_expr(visitor);
          self.expect(Token::RParen);
          y
        }
        Token::Number => {
          visitor.visit_number(self.next_span())
        }
        Token::Symbol => {
          visitor.visit_variable(self.next_span())
        }
        Token::Hyphen => {
          self.next();
          let y = self.parse_prec(visitor, 30);
          visitor.visit_op1(Op1::Neg, y)
        }
        Token::Not => {
          self.next();
          let y = self.parse_prec(visitor, 30);
          visitor.visit_op1(Op1::Not, y)
        }
        _ => {
          // error
          visitor.visit_undefined()
        }
      };

    loop {
      x =
        match self.token() {
          Token::Query if n <= 1 => {
            self.next();
            let y = self.parse_expr(visitor);
            self.expect(Token::Colon);
            let z = self.parse_prec(visitor, 0);
            visitor.visit_ternary(x, y, z)
          }
          Token::Or if n <= 6 => {
            self.next();
            let y = self.parse_prec(visitor, 7);
            visitor.visit_or(x, y)
          }
          Token::And if n <= 8 => {
            self.next();
            let y = self.parse_prec(visitor, 9);
            visitor.visit_and(x, y)
          }
          Token::CmpEq if n <= 10 => {
            self.next();
            let y = self.parse_prec(visitor, 11);
            visitor.visit_op2(Op2::CmpEq, x, y)
          }
          Token::CmpGe if n <= 10 => {
            self.next();
            let y = self.parse_prec(visitor, 11);
            visitor.visit_op2(Op2::CmpGe, x, y)
          }
          Token::CmpGt if n <= 10 => {
            self.next();
            let y = self.parse_prec(visitor, 11);
            visitor.visit_op2(Op2::CmpGt, x, y)
          }
          Token::CmpLe if n <= 10 => {
            self.next();
            let y = self.parse_prec(visitor, 11);
            visitor.visit_op2(Op2::CmpLe, x, y)
          }
          Token::CmpLt if n <= 10 => {
            self.next();
            let y = self.parse_prec(visitor, 11);
            visitor.visit_op2(Op2::CmpLt, x, y)
          }
          Token::CmpNe if n <= 10 => {
            self.next();
            let y = self.parse_prec(visitor, 11);
            visitor.visit_op2(Op2::CmpNe, x, y)
          }
          Token::BitOr if n <= 12 => {
            self.next();
            let y = self.parse_prec(visitor, 13);
            visitor.visit_op2(Op2::BitOr, x, y)
          }
          Token::BitXor if n <= 14 => {
            self.next();
            let y = self.parse_prec(visitor, 15);
            visitor.visit_op2(Op2::BitXor, x, y)
          }
          Token::BitAnd if n <= 16 => {
            self.next();
            let y = self.parse_prec(visitor, 17);
            visitor.visit_op2(Op2::BitAnd, x, y)
          }
          Token::Shl if n <= 18 => {
            self.next();
            let y = self.parse_prec(visitor, 19);
            visitor.visit_op2(Op2::Shl, x, y)
          }
          Token::Shr if n <= 18 => {
            self.next();
            let y = self.parse_prec(visitor, 19);
            visitor.visit_op2(Op2::Shr, x, y)
          }
          Token::Add if n <= 20 => {
            self.next();
            let y = self.parse_prec(visitor, 21);
            visitor.visit_op2(Op2::Add, x, y)
          }
          Token::Hyphen if n <= 20 => {
            self.next();
            let y = self.parse_prec(visitor, 21);
            visitor.visit_op2(Op2::Sub, x, y)
          }
          Token::Div if n <= 22 => {
            self.next();
            let y = self.parse_prec(visitor, 23);
            visitor.visit_op2(Op2::Div, x, y)
          }
          Token::Mul if n <= 22 => {
            self.next();
            let y = self.parse_prec(visitor, 23);
            visitor.visit_op2(Op2::Mul, x, y)
          }
          Token::Rem if n <= 22 => {
            self.next();
            let y = self.parse_prec(visitor, 23);
            visitor.visit_op2(Op2::Rem, x, y)
          }
          Token::Field if self.token_is_attached() && n <= 40 => {
            visitor.visit_field(self.next_span(), x)
          }
          Token::LBracket if self.token_is_attached() && n <= 40 => {
            self.next();
            let i = self.parse_expr(visitor);
            self.expect(Token::RBracket);
            visitor.visit_index(x, i)
          }
          _ => {
            return x;
          }
        };
    }
  }
}

pub struct SexpPrinter;

impl Visitor for SexpPrinter {
  type Expr = Sexp;

  fn visit_undefined(&mut self) -> Self::Expr {
    Sexp::atom(b"undefined")
  }

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
}
