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

  fn visit_variable(&mut self, x: &[u8]) -> Self::Expr;

  fn visit_number(&mut self, x: &[u8]) -> Self::Expr;

  fn visit_undefined(&mut self) -> Self::Expr;

  fn visit_op1(&mut self, f: Op1, x: Self::Expr) -> Self::Expr;

  fn visit_op2(&mut self, f: Op2, x: Self::Expr, y: Self::Expr) -> Self::Expr;

  fn visit_field(&mut self, f: &[u8], x: Self::Expr) -> Self::Expr;
}

pub struct Parser<'a> {
  lexer: Lexer<'a>,
}

fn infix_binding_power(op: Op2) -> (usize, usize) {
  match op {
    Op2::CmpEq | Op2::CmpGe | Op2::CmpGt | Op2::CmpLe | Op2::CmpLt | Op2::CmpNe =>
      (1, 2),
    Op2::Shl | Op2::Shr =>
      (3, 4),
    Op2::Add | Op2::Sub =>
      (5, 6),
    Op2::Div | Op2::Mul | Op2::Rem =>
      (7, 8),
  }
}

impl<'a> Parser<'a> {
  pub fn new(source: &'a [u8]) -> Self {
    Self {
      lexer: Lexer::new(source),
    }
  }

  fn advance(&mut self) {
    self.lexer.next();
  }

  fn token(&self) -> Token {
    return self.lexer.token();
  }

  fn token_slice(&self) -> &'a [u8] {
    return self.lexer.token_slice();
  }

  fn expect(&mut self, t: Token) {
    if self.token() == t {
      self.advance()
    }

    // else error
  }

  fn parse_leaf<T: Visitor>(&mut self, visitor: &mut T) -> T::Expr {
    match self.token() {
      Token::Number => {
        let r = visitor.visit_number(self.token_slice());
        self.advance();
        return r;
      }
      Token::Symbol => {
        let r = visitor.visit_variable(self.token_slice());
        self.advance();
        return r;
      }
      _ => {
        // error
        return visitor.visit_undefined();
      }
    }
  }

  fn parse_expr<T: Visitor>(&mut self, visitor: &mut T) -> T::Expr {
    return self.parse_expr_prec(0, visitor);
  }

  fn parse_expr_prec<T: Visitor>(&mut self, n: usize, visitor: &mut T) -> T::Expr {
    let mut x =
      match self.token() {
        Token::Number => {
          let s = self.token_slice();
          self.advance();
          visitor.visit_number(s)
        }
        Token::Symbol => {
          let s = self.token_slice();
          self.advance();
          visitor.visit_variable(s)
        }
        Token::Dash => {
          self.advance();
          let y = self.parse_expr_prec(9, visitor);
          visitor.visit_op1(Op1::Neg, y)
        }
        Token::Not => {
          self.advance();
          let y = self.parse_expr_prec(9, visitor);
          visitor.visit_op1(Op1::Not, y)
        }
        _ => {
          // error
          visitor.visit_undefined()
        }
      };

    loop {
      match self.token() {
        Token::Field => {
          if 20 < n {
            break;
          }
          let f = self.token_slice();
          self.advance();
          x = visitor.visit_field(f, x);
          continue;
        }
        _ => {
        }
      };

      let op =
        match self.token() {
          Token::Add => Op2::Add,
          Token::CmpEq => Op2::CmpEq,
          Token::CmpGe => Op2::CmpGe,
          Token::CmpGt => Op2::CmpGt,
          Token::CmpLe => Op2::CmpLe,
          Token::CmpLt => Op2::CmpLt,
          Token::CmpNe => Op2::CmpNe,
          Token::Dash => Op2::Sub,
          Token::Div => Op2::Div,
          Token::Mul => Op2::Mul,
          Token::Rem => Op2::Rem,
          Token::Shl => Op2::Shl,
          Token::Shr => Op2::Shr,
          _ => { break; }
        };

      let (a, b) = infix_binding_power(op);

      if a < n {
        break;
      }

      self.advance();

      let y = self.parse_expr_prec(b, visitor);

      x = visitor.visit_op2(op, x, y);
    }

    return x;
  }
}

pub struct SexpPrinter;

impl Visitor for SexpPrinter {
  type Expr = Sexp;

  fn visit_variable(&mut self, x: &[u8]) -> Self::Expr {
    Sexp::atom(x)
  }

  fn visit_number(&mut self, x: &[u8]) -> Self::Expr {
    Sexp::atom(x)
  }

  fn visit_undefined(&mut self) -> Self::Expr {
    Sexp::atom(b"undefined")
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
}
