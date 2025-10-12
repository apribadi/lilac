use crate::token::Token;

pub struct Lexer<'a> {
  source: &'a [u8],
  index: usize,
  token_start: usize,
  token_stop: usize,
  state: u8,
  token: Token,
}

const A: u8 = 10;
const B: u8 = 11;
const C: u8 = 12;
const D: u8 = 13;
const E: u8 = 14;
const F: u8 = 15;

// STATES
//
// 0 - reset
// 1 - illegal
// 2 - operator continuation
// 3 - comment
// 4 - punctuation
// 5 - plus minus
// 6 - operator start
// 7 - dot
// 8 - colon
// 9 - symbol continuation
// A - symbol start
// B - number start
// C - double quote start
// D - number continuation
// E - double quote continuation
// F - quote end

const STATE: [[u8; 16]; 16] = [
// 0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F
  [1, 1, 1, 3, 1, 1, 1, 1, 1, 1, 1, 1, E, 1, E, 1], // 0 - illegal
  [0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, E, 0, E, 0], // 1 - space        \t sp
  [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, E, 0, E, 0], // 2 - line feed    \n
  [3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, E, 3, E, 3], // 3 - hash         #
  [4, 4, 4, 3, 4, 4, 4, 4, 4, 4, 4, 4, E, 4, E, 4], // 4 - punctuation  ( ) , ; [ ] { }
  [5, 5, 2, 3, 5, 2, 2, 2, 2, 5, 5, D, E, D, E, 5], // 5 - plus minus   +-
  [6, 6, 2, 3, 6, 2, 2, 2, 2, 6, 6, 6, E, 6, E, 6], // 6 - operator     ! $ % & * / < = > ? @ ^ | ~
  [7, 7, 2, 3, 7, D, 2, 2, 2, 7, 7, D, E, D, E, 7], // 7 - dot          .
  [8, 8, 2, 3, 8, 2, 2, 2, 2, 8, 8, 8, E, 8, E, 8], // 8 - colon        :
  [A, A, A, 3, A, A, A, 9, 9, 9, 9, D, E, D, E, A], // 9 - underscore   _
  [A, A, A, 3, A, A, A, 9, 9, 9, 9, D, E, D, E, A], // A - alphabet     A ... Z a ... z
  [B, B, B, 3, B, D, B, D, B, 9, 9, D, E, D, E, B], // B - digit        0 1 2 3 4 5 6 7 7 8 9
  [C, C, C, 3, C, C, C, C, C, C, C, C, F, C, F, C], // C - double quote "
  [1, 1, 1, 3, 1, 1, 1, 1, 1, 1, 1, 1, E, 1, E, 1], // D - single quote '
  [1, 1, 1, 3, 1, 1, 1, 1, 1, 1, 1, 1, E, 1, E, 1], // E - back quote   `
  [1, 1, 1, 3, 1, 1, 1, 1, 1, 1, 1, 1, E, 1, E, 1], // F - back slash   \
];

const CLASS: [u8; 128] = [
  0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 0, 0, 0, 0, 0,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  1, 6, C, 3, 6, 6, 6, D, 4, 4, 6, 5, 4, 5, 7, 6,
  B, B, B, B, B, B, B, B, B, B, 8, 4, 6, 6, 6, 6,
  6, A, A, A, A, A, A, A, A, A, A, A, A, A, A, A,
  A, A, A, A, A, A, A, A, A, A, A, 4, F, 4, 6, 9,
  E, A, A, A, A, A, A, A, A, A, A, A, A, A, A, A,
  A, A, A, A, A, A, A, A, A, A, A, 4, 6, 4, 6, 0,
];

static TABLE: [[u8; 16]; 256] = {
  let mut t = [[0u8; 16]; 256];
  let mut i = 0u8;
  loop {
    t[i as usize] = STATE[if i <= 127 { CLASS[i as usize] as usize } else { 0 }];
    if i == 255 { break; }
    i += 1;
  }
  t
};

static OUT: [u8; 16] = [
  0b00, // reset
  0b11, // illegal
  0b10, // operator continuation
  0b00, // comment
  0b11, // punctuation
  0b11, // plus/minus
  0b11, // operator start
  0b11, // dot
  0b11, // colon
  0b10, // symbol continuation
  0b11, // symbol start
  0b11, // number start
  0b11, // quote start
  0b10, // number continuation
  0b10, // quote continuation
  0b10, // quote end
];

fn is_start(x: u8) -> bool {
  OUT[(x & 0b1111) as usize] & 1 != 0
}

fn is_token(x: u8) -> bool {
  OUT[(x & 0b1111) as usize] & 2 != 0
}

impl<'a> Lexer<'a> {
  pub fn new(source: &'a [u8]) -> Self {
    let mut t =
      Self {
        source,
        index: 0,
        token_start: 0,
        token_stop: 0,
        state: 0,
        token: Token::Error,
      };
    t.next();
    t
  }

  pub fn next(&mut self) {
    let n = self.source.len();
    let mut s = self.state;
    let mut z;
    let mut i = self.index;

    let start;
    loop {
      if is_start(s) { start = i - 1; break; }
      if i == n { start = i; s = 0; break; }
      s = TABLE[unsafe { *self.source.get_unchecked(i) } as usize][s as usize];
      i += 1;
    }

    let stop;
    loop {
      z = s;
      if i == n { stop = i; s = 0; break; }
      s = TABLE[unsafe { *self.source.get_unchecked(i) } as usize][s as usize];
      i += 1;
      if is_start(s) || ! is_token(s) { stop = i - 1; break; }
    }

    self.index = i;
    self.token_start = start;
    self.token_stop = stop;
    self.state = s;

    self.token =
      match z {
        0 => Token::Eof,
        F => Token::DoubleQuote,
        4 | 5 | 6 | 7 | 8 =>
          unsafe { core::mem::transmute::<u8, Token>(*self.source.get_unchecked(start)) },
        B | D =>
          Token::Number,
        A =>
          match unsafe { self.source.get_unchecked(start) } {
            b'_' => Token::Underscore,
            _ => Token::Symbol,
          },
        2 =>
          match unsafe { self.source.get_unchecked(start .. stop) } {
            b"&&" => Token::And,
            b"==" => Token::CmpEq,
            b">=" => Token::CmpGe,
            b"<=" => Token::CmpLe,
            b"!=" => Token::CmpNe,
            b"||" => Token::Or,
            b"<<" => Token::Shl,
            b">>" => Token::Shr,
            _ => Token::Error,
          },
        9 =>
          match unsafe { self.source.get_unchecked(start .. stop) } {
            b"break" => Token::Break,
            b"continue" => Token::Continue,
            b"do" => Token::Do,
            b"elif" => Token::Elif,
            b"else" => Token::Else,
            b"for" => Token::For,
            b"fun" => Token::Fun,
            b"if" => Token::If,
            b"let" => Token::Let,
            b"loop" => Token::Loop,
            b"return" => Token::Return,
            b"while" => Token::While,
            _ => {
              match unsafe { *self.source.get_unchecked(start) } {
                b'.' => Token::Field,
                b':' => Token::Error,
                _ => Token::Symbol,
              }
            }
          },
        _ => Token::Error,
      }
  }

  pub fn token(&self) -> Token {
    return self.token;
  }

  pub fn span(&self) -> &'a [u8] {
    return unsafe { self.source.get_unchecked(self.token_start .. self.token_stop) };
  }
}
