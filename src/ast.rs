use oxcart::Arena;
use crate::parse;
use crate::token::Token;
use crate::lexer::Lexer;
use crate::op1::Op1;
use crate::op2::Op2;

#[derive(Clone, Copy, Debug)]
pub enum Expr<'a> {
  And(&'a (Expr<'a>, Expr<'a>)),
  Field(&'a (&'a [u8], Expr<'a>)),
  Index(&'a (Expr<'a>, Expr<'a>)),
  Int(i64),
  Op1(&'a (Op1, Expr<'a>)),
  Op2(&'a (Op2, Expr<'a>, Expr<'a>)),
  Or(&'a (Expr<'a>, Expr<'a>)),
  Ternary(&'a (Expr<'a>, Expr<'a>, Expr<'a>)),
  Undefined,
  Variable(&'a [u8]),
}

#[derive(Debug)]
pub enum Stmt<'a> {
  Expr(Expr<'a>),
  Let(&'a (&'a [u8], Expr<'a>)),
}

pub fn parse_expr<'a>(source: &[u8], arena: &mut Arena<'a>) -> Expr<'a> {
  parse::parse_expr(&mut Lexer::new(source), &mut AstEmit { arena })
}

pub fn parse_stmt<'a>(source: &[u8], arena: &mut Arena<'a>) -> Stmt<'a> {
  parse::parse_stmt(&mut Lexer::new(source), &mut AstEmit { arena })
}

struct AstEmit<'a, 'b> {
  arena: &'b mut Arena<'a>,
}

impl<'a, 'b> parse::Emit for AstEmit<'a, 'b> {
  type Expr = Expr<'a>;

  type Stmt = Stmt<'a>;

  fn emit_variable(&mut self, x: &[u8]) -> Self::Expr {
    return Expr::Variable(self.arena.copy_slice(x));
  }

  fn emit_number(&mut self, x: &[u8]) -> Self::Expr {
    let n =
      match i64::from_str_radix(str::from_utf8(x).unwrap(), 10) {
        Err(_) => {
          return Expr::Undefined;
        }
        Ok(n) => n
      };
    return Expr::Int(n);
  }

  fn emit_ternary(&mut self, p: Self::Expr, x: Self::Expr, y: Self::Expr) -> Self::Expr {
    return Expr::Ternary(self.arena.alloc().init((p, x, y)));
  }

  fn emit_or(&mut self, x: Self::Expr, y: Self::Expr) -> Self::Expr {
    return Expr::Or(self.arena.alloc().init((x, y)));
  }

  fn emit_and(&mut self, x: Self::Expr, y: Self::Expr) -> Self::Expr {
    return Expr::And(self.arena.alloc().init((x, y)));
  }

  fn emit_op1(&mut self, op: Op1, x: Self::Expr) -> Self::Expr {
    return Expr::Op1(self.arena.alloc().init((op, x)));
  }

  fn emit_op2(&mut self, op: Op2, x: Self::Expr, y: Self::Expr) -> Self::Expr {
    return Expr::Op2(self.arena.alloc().init((op, x, y)));
  }

  fn emit_field(&mut self, s: &[u8], x: Self::Expr) -> Self::Expr {
    let s: &_ = self.arena.copy_slice(s);
    return Expr::Field(self.arena.alloc().init((s, x)));
  }

  fn emit_index(&mut self, x: Self::Expr, i: Self::Expr) -> Self::Expr {
    return Expr::Index(self.arena.alloc().init((x, i)));
  }

  fn emit_error_missing_expr(&mut self) -> Self::Expr {
    // TODO: accumulate errors
    return Expr::Undefined;
  }

  fn emit_let(&mut self, s: &[u8], x: Self::Expr) -> Self::Stmt {
    let s: &_ = self.arena.copy_slice(s);
    return Stmt::Let(self.arena.alloc().init((s, x)));
  }

  fn emit_stmt_expr(&mut self, x: Self::Expr) -> Self::Stmt {
    return Stmt::Expr(x);
  }

  fn emit_error_missing_expected_token(&mut self, token: Token) {
    let _ = token;
    // TODO: accumulate errors
  }
}
