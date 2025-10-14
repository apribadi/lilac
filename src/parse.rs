use crate::lex::Lexer;
use crate::ir1::Inst;

pub struct Parser<'a> {
  lexer: Lexer<'a>,
}

impl<'a> Parser<'a> {
  pub fn new(source: &'a [u8]) -> Self {
    Self {
      lexer: Lexer::new(source),
    }
  }
}
