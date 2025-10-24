#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Token {
  Not      = 0x21, // !
  Dollar   = 0x24, // $
  Rem      = 0x25, // %
  BitAnd   = 0x26, // &
  LParen   = 0x28, // (
  RParen   = 0x29, // )
  Mul      = 0x2a, // *
  Add      = 0x2b, // +
  Comma    = 0x2c, // ,
  Hyphen   = 0x2d, // -
  Dot      = 0x2e, // .
  Div      = 0x2f, // /
  Colon    = 0x3a, // :
  Semi     = 0x3b, // ;
  CmpLt    = 0x3c, // <
  Equal    = 0x3d, // =
  CmpGt    = 0x3e, // >
  Query    = 0x3f, // ?
  At       = 0x40, // @
  LBracket = 0x5b, // [
  RBracket = 0x5d, // ]
  BitXor   = 0x5e, // ^
  LBrace   = 0x7b, // {
  BitOr    = 0x7c, // |
  RBrace   = 0x7d, // }
  Tilde    = 0x7e, // ~
  Eof      = 0,
  Error    = 1,
  And,             // &&
  CmpEq,           // ==
  CmpGe,           // >=
  CmpLe,           // <=
  CmpNe,           // !=
  DotDotDot,       // ...
  Or,              // ||
  Set,             // <-
  Shl,             // <<
  Shr,             // >>
  Underscore,      // _
  Field,           // .foo
  StaticField,     // :foo
  Break,
  Continue,
  Do,
  Elif,
  Else,
  False,
  For,
  Fun,
  If,
  Let,
  Loop,
  Return,
  True,
  Var,
  While,
  Symbol,
  Number,
  DoubleQuote,
}
