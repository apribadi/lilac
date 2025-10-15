use crate::lex::Lexer;
use crate::token::Token;
use crate::ir1::Inst;
use crate::sexp::Sexp;

pub fn parse<T: Visitor>(source: &[u8], visitor: &mut T) -> T::Expr {
  return Parser::new(source).parse_expr(visitor);
}

pub trait Visitor {
  type Expr;

  fn visit_variable(&mut self, x: &[u8]) -> Self::Expr;

  fn visit_number(&mut self, x: &[u8]) -> Self::Expr;

  fn visit_undefined(&mut self) -> Self::Expr;

  fn visit_add(&mut self, x: Self::Expr, y: Self::Expr) -> Self::Expr;
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
    let mut x = self.parse_leaf(visitor);
    loop {
      match self.token() {
        Token::Add => {
          self.advance();
          let y = self.parse_leaf(visitor);
          x = visitor.visit_add(x, y);
        }
        _ => {
          return x;
        }
      }
    }
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

  fn visit_add(&mut self, x: Sexp, y: Sexp) -> Self::Expr {
    Sexp::from_array([Sexp::atom(b"add"), x, y])
  }
}
