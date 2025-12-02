use crate::lexer::Lexer;
use crate::ir1::Op1;
use crate::ir1::Op2;
use crate::token::Token;

pub trait Out {
  fn on_fundef(&mut self, name: &[u8], n_args: u32, n_stmts: u32);

  fn on_binding(&mut self, name: Option<&[u8]>); // TODO: type ascription

  fn on_variable(&mut self, symbol: &[u8]);

  fn on_bool(&mut self, x: bool);

  fn on_number(&mut self, number: &[u8]);

  fn on_ternary(&mut self);

  fn on_or(&mut self);

  fn on_and(&mut self);

  fn on_op1(&mut self, op: Op1);

  fn on_op2(&mut self, op: Op2);

  fn on_field(&mut self, symbol: &[u8]);

  fn on_index(&mut self);

  fn on_if(&mut self, n_stmts: u32);

  fn on_if_else(&mut self, n_stmts_then: u32, n_stmts_else: u32);

  fn on_call(&mut self, n_args: u32);

  fn on_loop(&mut self, n_stmts: u32);

  fn on_stmt_expr_list(&mut self, n_exprs: u32);

  fn on_break(&mut self, n_args: u32);

  fn on_continue(&mut self);

  fn on_let(&mut self, n_binds: u32, n_args: u32);

  fn on_return(&mut self, n_args: u32);

  fn on_set(&mut self, symbol: &[u8]);

  fn on_set_field(&mut self, symbol: &[u8]);

  fn on_set_index(&mut self);

  fn on_var(&mut self, symbol: &[u8]);

  fn on_while(&mut self, n_stmts: u32);

  fn on_error_missing_expected_token(&mut self, token: Token);

  fn on_error_missing_expr(&mut self);
}

// toplevel sequence of items

pub fn parse<'a, O: Out>(t: &mut Lexer<'a>, o: &mut O) {
  loop {
    match t.token() {
      Token::Eof => {
        break;
      }
      Token::Fun => {
        t.next();
        let name = expect_symbol(t, o);
        expect(t, o, Token::LParen);
        let m = parse_bind_list(t, o, Token::RParen);
        expect(t, o, Token::RParen);
        let n = parse_block(t, o);
        o.on_fundef(name, m, n);
      }
      _ => {
        // TODO: error?
        break;
      }
    }
  }
}

fn expect<'a, O: Out>(t: &mut Lexer<'a>, o: &mut O, token: Token) {
  if t.token() != token {
    o.on_error_missing_expected_token(token);
  } else {
    t.next();
  }
}

fn expect_symbol<'a, O: Out>(t: &mut Lexer<'a>, o: &mut O) -> &'a [u8] {
  if t.token() != Token::Symbol {
    o.on_error_missing_expected_token(Token::Symbol);
    // TODO: Option::None?
    return b"!!!";
  } else {
    let symbol = t.token_span();
    t.next();
    return symbol;
  }
}

fn parse_bind<'a, O: Out>(t: &mut Lexer<'a>, o: &mut O) {
  match t.token() {
    Token::Symbol => {
      o.on_binding(Some(t.token_span()));
      t.next();
    }
    Token::Underscore => {
      o.on_binding(None);
      t.next();
    }
    _ => {
      o.on_error_missing_expected_token(Token::Symbol);
      o.on_binding(None);
    }
  }
}

fn parse_bind_list<'a, O: Out>(t: &mut Lexer<'a>, o: &mut O, stop: Token) -> u32 {
  let mut n_binds = 0;
  if t.token() != stop {
    loop {
      parse_bind(t, o);
      n_binds += 1;
      if t.token() != Token::Comma { break; }
      t.next();
    }
  }
  return n_binds;
}

fn parse_expr<'a, O: Out>(t: &mut Lexer<'a>, o: &mut O) {
  parse_expr_prec(t, o, 0x00);
}

fn parse_expr_list<'a, O: Out>(t: &mut Lexer<'a>, o: &mut O, stop: Token) -> u32 {
  let mut n_exprs = 0;
  if t.token() != stop {
    loop {
      parse_expr(t, o);
      n_exprs += 1;
      if t.token() != Token::Comma { break; }
      t.next();
    }
  }
  return n_exprs;
}

fn parse_expr_prec<'a, O: Out>(t: &mut Lexer<'a>, o: &mut O, n: u32) {
  let _: bool = parse_prec(t, o, n, false);
}

// returns `true` if we parsed a statement

fn parse_prec<'a, O: Out>(t: &mut Lexer<'a>, o: &mut O, n: u32, is_stmt: bool) -> bool {
  match t.token() {
    Token::LParen => {
      t.next();
      parse_expr(t, o);
      expect(t, o, Token::RParen);
    }
    Token::True => {
      t.next();
      o.on_bool(true);
    }
    Token::False => {
      t.next();
      o.on_bool(false);
    }
    Token::Number => {
      let number = t.token_span();
      t.next();
      o.on_number(number);
    }
    Token::Symbol => {
      let symbol = t.token_span();
      t.next();
      if is_stmt && t.token() == Token::Equal {
        t.next();
        parse_expr(t, o);
        o.on_set(symbol);
        return true;
      } else {
        o.on_variable(symbol);
      }
    }
    Token::Hyphen => {
      t.next();
      parse_expr_prec(t, o, 0xff);
      o.on_op1(Op1::Neg);
    }
    Token::Not => {
      t.next();
      parse_expr_prec(t, o, 0xff);
      o.on_op1(Op1::Not);
    }
    Token::If => {
      t.next();
      parse_expr(t, o);
      let n = parse_block(t, o);
      if t.token() == Token::Else {
        t.next();
        let m = parse_block(t, o);
        o.on_if_else(n, m);
      } else {
        o.on_if(n);
      }
    }
    Token::Loop => {
      t.next();
      let n = parse_block(t, o);
      o.on_loop(n);
    }
    _ => {
      o.on_error_missing_expr();
    }
  }

  loop {
    match t.token() {
      Token::Query if n <= 0x10 => {
        t.next();
        parse_expr(t, o);
        expect(t, o, Token::Colon);
        parse_expr_prec(t, o, 0x10);
        o.on_ternary();
      }
      Token::Or if n <= 0x20 => {
        t.next();
        parse_expr_prec(t, o, 0x21);
        o.on_or();
      }
      Token::And if n <= 0x30 => {
        t.next();
        parse_expr_prec(t, o, 0x31);
        o.on_and();
      }
      Token::CmpEq if n <= 0x40 => {
        t.next();
        parse_expr_prec(t, o, 0x41);
        o.on_op2(Op2::CmpEq);
      }
      Token::CmpGe if n <= 0x40 => {
        t.next();
        parse_expr_prec(t, o, 0x41);
        o.on_op2(Op2::CmpGe);
      }
      Token::CmpGt if n <= 0x40 => {
        t.next();
        parse_expr_prec(t, o, 0x41);
        o.on_op2(Op2::CmpGt);
      }
      Token::CmpLe if n <= 0x40 => {
        t.next();
        parse_expr_prec(t, o, 0x41);
        o.on_op2(Op2::CmpLe);
      }
      Token::CmpLt if n <= 0x40 => {
        t.next();
        parse_expr_prec(t, o, 0x41);
        o.on_op2(Op2::CmpLt);
      }
      Token::CmpNe if n <= 0x40 => {
        t.next();
        parse_expr_prec(t, o, 0x41);
        o.on_op2(Op2::CmpNe);
      }
      Token::BitOr if n <= 0x50 => {
        t.next();
        parse_expr_prec(t, o, 0x51);
        o.on_op2(Op2::BitOr);
      }
      Token::BitXor if n <= 0x60 => {
        t.next();
        parse_expr_prec(t, o, 0x61);
        o.on_op2(Op2::BitXor);
      }
      Token::BitAnd if n <= 0x70 => {
        t.next();
        parse_expr_prec(t, o, 0x71);
        o.on_op2(Op2::BitAnd);
      }
      Token::Shl if n <= 0x80 => {
        t.next();
        parse_expr_prec(t, o, 0x81);
        o.on_op2(Op2::Shl);
      }
      Token::Shr if n <= 0x80 => {
        t.next();
        parse_expr_prec(t, o, 0x81);
        o.on_op2(Op2::Shr);
      }
      Token::Add if n <= 0x90 => {
        t.next();
        parse_expr_prec(t, o, 0x91);
        o.on_op2(Op2::Add);
      }
      Token::Hyphen if n <= 0x90 => {
        t.next();
        parse_expr_prec(t, o, 0x91);
        o.on_op2(Op2::Sub);
      }
      Token::Div if n <= 0xA0 => {
        t.next();
        parse_expr_prec(t, o, 0xA1);
        o.on_op2(Op2::Div);
      }
      Token::Mul if n <= 0xA0 => {
        t.next();
        parse_expr_prec(t, o, 0xA1);
        o.on_op2(Op2::Mul);
      }
      Token::Rem if n <= 0xA0 => {
        t.next();
        parse_expr_prec(t, o, 0xA1);
        o.on_op2(Op2::Rem);
      }
      Token::Field if t.token_is_attached() => {
        let symbol = &t.token_span()[1 ..];
        t.next();
        if is_stmt && t.token() == Token::Equal {
          t.next();
          parse_expr(t, o);
          o.on_set_field(symbol);
          return true;
        } else {
          o.on_field(symbol);
        }
      }
      Token::LBracket if t.token_is_attached() => {
        t.next();
        parse_expr(t, o);
        expect(t, o, Token::RBracket);
        if is_stmt && t.token() == Token::Equal {
          t.next();
          parse_expr(t, o);
          o.on_set_index();
          return true;
        } else {
          o.on_index();
        }
      }
      Token::LParen if t.token_is_attached() => {
        t.next();
        let n_args = parse_expr_list(t, o, Token::RParen);
        expect(t, o, Token::RParen);
        o.on_call(n_args);
      }
      _ => {
        return false;
      }
    }
  }
}

fn parse_block<'a, O: Out>(t: &mut Lexer<'a>, o: &mut O) -> u32 {
  expect(t, o, Token::LBrace);

  let mut n_stmts = 0;

  loop {
    match t.token() {
      Token::RBrace => {
        t.next();
        break;
      }
      Token::Break => {
        t.next();
        let n_args = parse_expr_list(t, o, Token::RBrace);
        o.on_break(n_args);
        n_stmts += 1;
        expect(t, o, Token::RBrace);
        break;
      }
      Token::Continue => {
        t.next();
        o.on_continue();
        n_stmts += 1;
        expect(t, o, Token::RBrace);
        break;
      }
      Token::Let => {
        t.next();
        // NB: we allow a list of zero bindings, like
        //
        //   let = f(x)
        //
        // which is a bit weird. but works semantically
        let n_binds = parse_bind_list(t, o, Token::Equal);
        expect(t, o, Token::Equal);
        let mut n_exprs = 0;
        loop {
          parse_expr(t, o);
          n_exprs += 1;
          if t.token() != Token::Comma { break; }
          t.next();
        }
        o.on_let(n_binds, n_exprs);
        n_stmts += 1;
      }
      Token::Return => {
        t.next();
        let n_args = parse_expr_list(t, o, Token::RBrace);
        o.on_return(n_args);
        n_stmts += 1;
        expect(t, o, Token::RBrace);
        break;
      }
      Token::Var => {
        t.next();
        let symbol = expect_symbol(t, o);
        expect(t, o, Token::Equal);
        parse_expr(t, o);
        o.on_var(symbol);
        n_stmts += 1;
      }
      Token::While => {
        t.next();
        parse_expr(t, o);
        let n = parse_block(t, o);
        o.on_while(n);
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

        let pos = t.token_start();

        if ! parse_prec(t, o, 0x00, true) {
          let mut n_exprs = 1;
          while t.token() == Token::Comma {
            t.next();
            parse_expr(t, o);
            n_exprs += 1;
          }
          o.on_stmt_expr_list(n_exprs);
        }

        n_stmts += 1;

        if t.token_start() == pos {
          expect(t, o, Token::RBrace);
          break;
        }
      }
    }
  }

  return n_stmts;
}
