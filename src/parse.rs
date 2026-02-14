use crate::arr::Arr;
use crate::ast::Binding;
use crate::ast::Expr;
use crate::ast::Fun;
use crate::ast::Item;
use crate::ast::Stmt;
use crate::buf::Buf;
use crate::lexer::Lexer;
use crate::operator::Op1;
use crate::operator::Op2;
use crate::symbol::Symbol;
use crate::token::Token;
use oxcart::Arena;

pub fn parse<'a>(source: &[u8], arena: &mut Arena<'a>) -> Arr<Item<'a>> {
  let mut t = T::new(source, arena);
  t.parse_item_list();
  return Arr::from(t.items.drain());
}

struct T<'a, 'b, 'c> {
  lexer: Lexer<'c>,
  arena: &'b mut Arena<'a>,
  items: Buf<Item<'a>>,
  binds: Buf<Binding>,
  exprs: Buf<Expr<'a>>,
  stmts: Buf<Stmt<'a>>,
}

#[derive(Eq, Ord, PartialEq, PartialOrd)]
enum P {
  Any,
  Ternary,
  Or,
  And,
  Cmp,
  BitOr,
  BitXor,
  BitAnd,
  Shift,
  Add,
  Mul,
  Prefix,
}

impl<'a, 'b, 'c> T<'a, 'b, 'c> {
  fn new(source: &'c [u8], arena: &'b mut Arena<'a>) -> Self {
    Self {
      lexer: Lexer::new(source),
      arena,
      items: Buf::new(),
      binds: Buf::new(),
      exprs: Buf::new(),
      stmts: Buf::new(),
    }
  }

  fn next(&mut self) {
    self.lexer.next();
  }

  fn token(&self) -> Token {
    return self.lexer.token();
  }

  fn token_start(&self) -> usize {
    return self.lexer.token_start();
  }

  fn token_span(&self) -> &'c [u8] {
    return self.lexer.token_span();
  }

  fn token_is_attached(&self) -> bool {
    return self.lexer.token_is_attached();
  }

  fn expect(&mut self, token: Token) {
    if self.token() != token {
      self.on_error_missing_expected_token(token);
    } else {
      self.next();
    }
  }

  fn expect_symbol(&mut self) -> &'c [u8] {
    if self.token() != Token::Symbol {
      self.on_error_missing_expected_token(Token::Symbol);
      // TODO: Option::None?
      return b"!!!";
    } else {
      let symbol = self.token_span();
      self.next();
      return symbol;
    }
  }

  fn parse_item_list(&mut self) {
    loop {
      match self.token() {
        Token::Eof => {
          break;
        }
        Token::Fun => {
          self.next();
          let name = self.expect_symbol();
          self.expect(Token::LParen);
          let m = self.parse_binding_list(Token::RParen);
          self.expect(Token::RParen);
          let n = self.parse_block();
          self.on_fun(name, m, n);
        }
        _ => {
          // TODO: error?
          break;
        }
      }
    }
  }

  fn parse_binding(&mut self) {
    match self.token() {
      Token::Symbol => {
        let s = self.token_span();
        self.on_binding(Some(s));
        self.next();
      }
      Token::Underscore => {
        self.on_binding(None);
        self.next();
      }
      _ => {
        self.on_error_missing_expected_token(Token::Symbol);
        self.on_binding(None);
      }
    }
  }

  fn parse_binding_list(&mut self, stop: Token) -> u32 {
    let mut n_bindings = 0;
    if self.token() != stop {
      loop {
        self.parse_binding();
        n_bindings += 1;
        if self.token() != Token::Comma { break; }
        self.next();
      }
    }
    return n_bindings;
  }

  fn parse_expr(&mut self) {
    self.parse_expr_prec(P::Any);
  }

  fn parse_expr_list(&mut self, stop: Token) -> u32 {
    let mut n_exprs = 0;
    if self.token() != stop {
      loop {
        self.parse_expr();
        n_exprs += 1;
        if self.token() != Token::Comma { break; }
        self.next();
      }
    }
    return n_exprs;
  }

  fn parse_expr_prec(&mut self, p: P) {
    let _: bool = self.parse_prec(p, false);
  }

  // returns `true` if we parsed a statement

  fn parse_prec(&mut self, p: P, is_stmt: bool) -> bool {
    match self.token() {
      Token::LParen => {
        self.next();
        self.parse_expr();
        self.expect(Token::RParen);
      }
      Token::True => {
        self.next();
        self.on_literal_bool(true);
      }
      Token::False => {
        self.next();
        self.on_literal_bool(false);
      }
      Token::Number => {
        let value = self.token_span();
        self.next();
        self.on_literal_number(value);
      }
      Token::Symbol => {
        let symbol = self.token_span();
        self.next();
        match self.token() {
          Token::Equal if is_stmt => {
            self.next();
            self.parse_expr();
            self.on_set(symbol);
            return true;
          }
          Token::Dec => {
            self.next();
            self.on_post_op(symbol, Op1::Dec);
          }
          Token::Inc => {
            self.next();
            self.on_variable(symbol);
            self.on_post_op(symbol, Op1::Inc);
          }
          _ => {
            self.on_variable(symbol);
          }
        }
      }
      Token::Dec => {
        self.next();
        let s = self.expect_symbol();
        self.on_pre_op(s, Op1::Dec);
      }
      Token::Inc => {
        self.next();
        let s = self.expect_symbol();
        self.on_pre_op(s, Op1::Inc);
      }
      Token::Hyphen => {
        self.next();
        self.parse_expr_prec(P::Prefix);
        self.on_op1(Op1::Neg);
      }
      Token::Not => {
        self.next();
        self.parse_expr_prec(P::Prefix);
        self.on_op1(Op1::Not);
      }
      Token::If => {
        self.next();
        self.parse_expr();
        let n = self.parse_block();
        if self.token() == Token::Else {
          self.next();
          let m = self.parse_block();
          self.on_if_else(n, m);
        } else {
          self.on_if(n);
        }
      }
      Token::Loop => {
        self.next();
        let n = self.parse_block();
        self.on_loop(n);
      }
      _ => {
        self.on_error_missing_expr();
      }
    }

    loop {
      match self.token() {
        Token::Query if p <= P::Ternary => {
          self.next();
          self.parse_expr();
          self.expect(Token::Colon);
          self.parse_expr_prec(P::Ternary);
          self.on_ternary();
        }
        Token::Or if p < P::Or => {
          self.next();
          self.parse_expr_prec(P::Or);
          self.on_or();
        }
        Token::And if p < P::And => {
          self.next();
          self.parse_expr_prec(P::And);
          self.on_and();
        }
        Token::CmpEq if p < P::Cmp => {
          self.next();
          self.parse_expr_prec(P::Cmp);
          self.on_op2(Op2::CmpEq);
        }
        Token::CmpGe if p < P::Cmp => {
          self.next();
          self.parse_expr_prec(P::Cmp);
          self.on_op2(Op2::CmpGe);
        }
        Token::CmpGt if p < P::Cmp => {
          self.next();
          self.parse_expr_prec(P::Cmp);
          self.on_op2(Op2::CmpGt);
        }
        Token::CmpLe if p < P::Cmp => {
          self.next();
          self.parse_expr_prec(P::Cmp);
          self.on_op2(Op2::CmpLe);
        }
        Token::CmpLt if p < P::Cmp => {
          self.next();
          self.parse_expr_prec(P::Cmp);
          self.on_op2(Op2::CmpLt);
        }
        Token::CmpNe if p < P::Cmp => {
          self.next();
          self.parse_expr_prec(P::Cmp);
          self.on_op2(Op2::CmpNe);
        }
        Token::BitOr if p < P::BitOr => {
          self.next();
          self.parse_expr_prec(P::BitOr);
          self.on_op2(Op2::BitOr);
        }
        Token::BitXor if p < P::BitXor => {
          self.next();
          self.parse_expr_prec(P::BitXor);
          self.on_op2(Op2::BitXor);
        }
        Token::BitAnd if p < P::BitAnd => {
          self.next();
          self.parse_expr_prec(P::BitAnd);
          self.on_op2(Op2::BitAnd);
        }
        Token::Shl if p < P::Shift => {
          self.next();
          self.parse_expr_prec(P::Shift);
          self.on_op2(Op2::Shl);
        }
        Token::Shr if p < P::Shift => {
          self.next();
          self.parse_expr_prec(P::Shift);
          self.on_op2(Op2::Shr);
        }
        Token::Add if p < P::Add => {
          self.next();
          self.parse_expr_prec(P::Add);
          self.on_op2(Op2::Add);
        }
        Token::Hyphen if p < P::Add => {
          self.next();
          self.parse_expr_prec(P::Add);
          self.on_op2(Op2::Sub);
        }
        Token::Div if p < P::Mul => {
          self.next();
          self.parse_expr_prec(P::Mul);
          self.on_op2(Op2::Div);
        }
        Token::Mul if p < P::Mul => {
          self.next();
          self.parse_expr_prec(P::Mul);
          self.on_op2(Op2::Mul);
        }
        Token::Rem if p < P::Mul => {
          self.next();
          self.parse_expr_prec(P::Mul);
          self.on_op2(Op2::Rem);
        }
        Token::Field if self.token_is_attached() => {
          let symbol = &self.token_span()[1 ..];
          self.next();
          if is_stmt && self.token() == Token::Equal {
            self.next();
            self.parse_expr();
            self.on_set_field(symbol);
            return true;
          } else {
            self.on_field(symbol);
          }
        }
        Token::LBracket if self.token_is_attached() => {
          self.next();
          self.parse_expr();
          self.expect(Token::RBracket);
          if is_stmt && self.token() == Token::Equal {
            self.next();
            self.parse_expr();
            self.on_set_index();
            return true;
          } else {
            self.on_index();
          }
        }
        Token::LParen if self.token_is_attached() => {
          self.next();
          let n_args = self.parse_expr_list(Token::RParen);
          self.expect(Token::RParen);
          self.on_call(n_args);
        }
        _ => {
          return false;
        }
      }
    }
  }

  fn parse_block(&mut self) -> u32 {
    self.expect(Token::LBrace);

    let mut n_stmts = 0;

    loop {
      match self.token() {
        Token::RBrace => {
          self.next();
          break;
        }
        Token::Break => {
          self.next();
          let n_args = self.parse_expr_list(Token::RBrace);
          self.on_break(n_args);
          n_stmts += 1;
          self.expect(Token::RBrace);
          break;
        }
        Token::Continue => {
          self.next();
          self.on_continue();
          n_stmts += 1;
          self.expect(Token::RBrace);
          break;
        }
        Token::Let => {
          self.next();
          // NB: we allow a list of zero bindings, like
          //
          //   let = f(x)
          //
          // which is a bit weird. but works semantically
          let n_bindings = self.parse_binding_list(Token::Equal);
          self.expect(Token::Equal);
          let mut n_exprs = 0;
          loop {
            self.parse_expr();
            n_exprs += 1;
            if self.token() != Token::Comma { break; }
            self.next();
          }
          self.on_let(n_bindings, n_exprs);
          n_stmts += 1;
        }
        Token::Return => {
          self.next();
          let n_args = self.parse_expr_list(Token::RBrace);
          self.on_return(n_args);
          n_stmts += 1;
          self.expect(Token::RBrace);
          break;
        }
        Token::Var => {
          self.next();
          let symbol = self.expect_symbol();
          self.expect(Token::Equal);
          self.parse_expr();
          self.on_var(symbol);
          n_stmts += 1;
        }
        Token::While => {
          self.next();
          self.parse_expr();
          let n = self.parse_block();
          self.on_while(n);
          n_stmts += 1;
        }
        _ => {
          // NB: If we couldn't parse anything at all, then we immediately close
          // the block so that we don't get stuck in an infinite loop.
          //
          // Note that we already know that there ISN'T an RBrace here, so the
          // expect will fail.
          //
          // Also, note that in this case we still emit an `undefined` expr/stmt.

          let pos = self.token_start();

          if ! self.parse_prec(P::Any, true) {
            let mut n_exprs = 1;
            while self.token() == Token::Comma {
              self.next();
              self.parse_expr();
              n_exprs += 1;
            }
            self.on_stmt_expr_list(n_exprs);
          }

          n_stmts += 1;

          if self.token_start() == pos {
            self.expect(Token::RBrace);
            break;
          }
        }
      }
    }

    return n_stmts;
  }

  // ------- PARSER OUTPUT TO AST -------

  fn alloc<T>(&mut self, x: T) -> &'a T {
    return self.arena.alloc().init(x);
  }

  fn push_item(&mut self, x: Item<'a>) {
    self.items.push(x);
  }

  fn push_bind(&mut self, x: Binding) {
    self.binds.push(x);
  }

  fn pop_bind_list(&mut self, n: u32) -> &'a [Binding] {
    return self.arena.slice_from_iter(self.binds.pop_list(n));
  }

  fn push_expr(&mut self, x: Expr<'a>) {
    self.exprs.push(x);
  }

  fn pop_expr(&mut self) -> Expr<'a> {
    return self.exprs.pop().unwrap();
  }

  fn pop_expr_list(&mut self, n: u32) -> &'a [Expr<'a>] {
    return self.arena.slice_from_iter(self.exprs.pop_list(n));
  }

  fn push_stmt(&mut self, x: Stmt<'a>) {
    self.stmts.push(x);
  }

  fn pop_stmt_list(&mut self, n: u32) -> &'a [Stmt<'a>] {
    return self.arena.slice_from_iter(self.stmts.pop_list(n));
  }

  fn on_fun(&mut self, name: &[u8], n_args: u32, n_stmts: u32) {
    let z = self.pop_stmt_list(n_stmts);
    let y = self.pop_bind_list(n_args);
    let x = Symbol::from_bytes(name);
    let x = Item::Fun(Fun { name: x, args: y, body: z });
    self.push_item(x);
  }

  fn on_binding(&mut self, name: Option<&[u8]>) {
    let x = Binding { name: name.map(Symbol::from_bytes) };
    self.push_bind(x);
  }

  fn on_variable(&mut self, symbol: &[u8]) {
    let s = Symbol::from_bytes(symbol);
    self.push_expr(Expr::Variable(s));
  }

  fn on_literal_bool(&mut self, value: bool) {
    self.push_expr(Expr::Bool(value));
  }

  fn on_literal_number(&mut self, x: &[u8]) {
    let n =
      match i64::from_str_radix(str::from_utf8(x).unwrap(), 10) {
        Err(_) => {
          self.push_expr(Expr::Undefined);
          return;
        }
        Ok(n) => n
      };
    self.push_expr(Expr::Int(n));
  }

  fn on_ternary(&mut self) {
    let y = self.pop_expr();
    let x = self.pop_expr();
    let p = self.pop_expr();
    let x = Expr::Ternary(self.alloc((p, x, y)));
    self.push_expr(x);
  }

  fn on_or(&mut self) {
    let y = self.pop_expr();
    let x = self.pop_expr();
    let x = Expr::Or(self.alloc((x, y)));
    self.push_expr(x);
  }

  fn on_and(&mut self) {
    let y = self.pop_expr();
    let x = self.pop_expr();
    let x = Expr::And(self.alloc((x, y)));
    self.push_expr(x);
  }

  fn on_op1(&mut self, op: Op1) {
    let x = self.pop_expr();
    let x = Expr::Op1(self.alloc((op, x)));
    self.push_expr(x);
  }

  fn on_op2(&mut self, op: Op2) {
    let y = self.pop_expr();
    let x = self.pop_expr();
    let x = Expr::Op2(self.alloc((op, x, y)));
    self.push_expr(x);
  }

  fn on_post_op(&mut self, symbol: &[u8], op: Op1) {
    let s = Symbol::from_bytes(symbol);
    let x = Expr::PostOp(self.alloc((s, op)));
    self.push_expr(x);
  }

  fn on_pre_op(&mut self, symbol: &[u8], op: Op1) {
    let s = Symbol::from_bytes(symbol);
    let x = Expr::PreOp(self.alloc((s, op)));
    self.push_expr(x);
  }

  fn on_field(&mut self, symbol: &[u8]) {
    let s = Symbol::from_bytes(symbol);
    let x = self.pop_expr();
    let x = Expr::Field(self.alloc((x, s)));
    self.push_expr(x);
  }

  fn on_index(&mut self) {
    let y = self.pop_expr();
    let x = self.pop_expr();
    let x = Expr::Index(self.alloc((x, y)));
    self.push_expr(x);
  }

  fn on_if(&mut self, n_stmts: u32) {
    let y = self.pop_stmt_list(n_stmts);
    let x = self.pop_expr();
    let x = Expr::If(self.alloc((x, y)));
    self.push_expr(x);
  }

  fn on_if_else(&mut self, n_stmts_then: u32, n_stmts_else: u32) {
    let z = self.pop_stmt_list(n_stmts_else);
    let y = self.pop_stmt_list(n_stmts_then);
    let x = self.pop_expr();
    let x = Expr::IfElse(self.alloc((x, y, z)));
    self.push_expr(x);
  }

  fn on_call(&mut self, n_args: u32) {
    let x = self.pop_expr_list(n_args);
    let f = self.pop_expr();
    let x = Expr::Call(self.alloc((f, x)));
    self.push_expr(x);
  }

  fn on_loop(&mut self, n_stmts: u32) {
    let x = self.pop_stmt_list(n_stmts);
    self.push_expr(Expr::Loop(x));
  }

  fn on_stmt_expr_list(&mut self, n_exprs: u32) {
    let x = self.pop_expr_list(n_exprs);
    self.push_stmt(Stmt::ExprList(x));
  }

  fn on_break(&mut self, n_args: u32) {
    let x = self.pop_expr_list(n_args);
    self.push_stmt(Stmt::Break(x));
  }

  fn on_continue(&mut self) {
    self.push_stmt(Stmt::Continue);
  }

  fn on_let(&mut self, n_bindings: u32, n_exprs: u32) {
    let y = self.pop_expr_list(n_exprs);
    let x = self.pop_bind_list(n_bindings);
    self.push_stmt(Stmt::Let(x, y));
  }

  fn on_return(&mut self, n_args: u32) {
    let x = self.pop_expr_list(n_args);
    self.push_stmt(Stmt::Return(x));
  }

  fn on_set(&mut self, symbol: &[u8]) {
    let s = Symbol::from_bytes(symbol);
    let x = self.pop_expr();
    self.push_stmt(Stmt::Set(s, x));
  }

  fn on_set_field(&mut self, symbol: &[u8]) {
    let s = Symbol::from_bytes(symbol);
    let y = self.pop_expr();
    let x = self.pop_expr();
    self.push_stmt(Stmt::SetField(x, s, y));
  }

  fn on_set_index(&mut self) {
    let z = self.pop_expr();
    let y = self.pop_expr();
    let x = self.pop_expr();
    self.push_stmt(Stmt::SetIndex(x, y, z));
  }

  fn on_var(&mut self, symbol: &[u8]) {
    let s = Symbol::from_bytes(symbol);
    let x = self.pop_expr();
    self.push_stmt(Stmt::Var(s, x));
  }

  fn on_while(&mut self, n_stmts: u32) {
    let y = self.pop_stmt_list(n_stmts);
    let x = self.pop_expr();
    self.push_stmt(Stmt::While(x, y));
  }

  fn on_error_missing_expected_token(&mut self, token: Token) {
    let _ = token;
    // TODO: report error on missing expected token
  }

  fn on_error_missing_expr(&mut self) {
    // TODO: report error on missing expected expression
    self.push_expr(Expr::Undefined);
  }
}
