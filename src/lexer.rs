use crate::token::Token;

// TOOD: This lexer is structured so that it is possible to efficiently
// implement it with SIMD.
//
// The critical path of the core byte-wise state transition should consist of
// just a shuffle instruction (cf Sheng).
//
// After post-processing the state sequence, we can extract token start and
// stop locations with bitwise operations.

pub struct Lexer<'a> {
  source: &'a [u8],
  index: usize,
  start: usize,
  stop: usize,
  state: u8,
  is_attached: bool,
  token: Token,
}

const A: u8 = 10;
const B: u8 = 11;
const C: u8 = 12;
const D: u8 = 13;
const E: u8 = 14;
const F: u8 = 15;

// 0 - nil
// 1 - illegal char
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

const TRANSITION_BY_CHAR_KIND: [[u8; 16]; 16] = [
// 0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F
  [1, 1, 1, 3, 1, 1, 1, 1, 1, 1, 1, 1, E, 1, E, 1], // 0 - illegal
  [0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, E, 0, E, 0], // 1 - space        \t sp
  [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, E, 0, E, 0], // 2 - line feed    \n
  [3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, E, 3, E, 3], // 3 - hash         #
  [4, 4, 4, 3, 4, 4, 4, 4, 4, 4, 4, 4, E, 4, E, 4], // 4 - punctuation  ( ) , ; [ ] { }
  [5, 5, 2, 3, 5, 2, 2, 2, 2, 5, 5, D, E, D, E, 5], // 5 - plus minus   +-
  [6, 6, 2, 3, 6, 2, 2, 2, 2, 6, 6, 6, E, 6, E, 6], // 6 - operator     ! $ % & * / < = > ? @ ^ | ~
  [7, 7, 2, 3, 7, 2, 2, 2, 2, 7, 7, D, E, D, E, 7], // 7 - dot          .
  [8, 8, 2, 3, 8, 2, 2, 2, 2, 8, 8, 8, E, 8, E, 8], // 8 - colon        :
  [A, A, A, 3, A, A, A, 9, 9, 9, 9, D, E, D, E, A], // 9 - underscore   _
  [A, A, A, 3, A, A, A, 9, 9, 9, 9, D, E, D, E, A], // A - alphabet     A ... Z a ... z
  [B, B, B, 3, B, D, B, 9, B, 9, 9, D, E, D, E, B], // B - digit        0 1 2 3 4 5 6 7 7 8 9
  [C, C, C, 3, C, C, C, C, C, C, C, C, F, C, F, C], // C - double quote "
  [1, 1, 1, 3, 1, 1, 1, 1, 1, 1, 1, 1, E, 1, E, 1], // D - single quote '
  [1, 1, 1, 3, 1, 1, 1, 1, 1, 1, 1, 1, E, 1, E, 1], // E - back quote   `
  [1, 1, 1, 3, 1, 1, 1, 1, 1, 1, 1, 1, E, 1, E, 1], // F - back slash   \
];

// 0 - skip
// 1 - token start
// 2 - token continue
static STATE_INFO: [u8; 16] = [
//0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F
  0, 1, 2, 0, 1, 1, 1, 1, 1, 2, 1, 1, 1, 2, 2, 2
];

const CHAR_KIND: [u8; 256] = [
  0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 0, 0, 0, 0, 0,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  1, 6, C, 3, 6, 6, 6, D, 4, 4, 6, 5, 4, 5, 7, 6,
  B, B, B, B, B, B, B, B, B, B, 8, 4, 6, 6, 6, 6,
  6, A, A, A, A, A, A, A, A, A, A, A, A, A, A, A,
  A, A, A, A, A, A, A, A, A, A, A, 4, F, 4, 6, 9,
  E, A, A, A, A, A, A, A, A, A, A, A, A, A, A, A,
  A, A, A, A, A, A, A, A, A, A, A, 4, 6, 4, 6, 0,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

static TRANSITION: [[u8; 16]; 256] = {
  let mut t = [[0u8; 16]; 256];
  let mut i = 0u8;
  loop {
    t[i as usize] = TRANSITION_BY_CHAR_KIND[CHAR_KIND[i as usize] as usize];
    if i == 255 { break; }
    i += 1;
  }
  t
};

fn is_start(x: u8) -> bool {
  return STATE_INFO[(x & 0b1111) as usize] & 1 != 0;
}

fn is_continue(x: u8) -> bool {
  return STATE_INFO[(x & 0b1111) as usize] & 2 != 0;
}

impl<'a> Lexer<'a> {
  pub fn new(source: &'a [u8]) -> Self {
    let mut t =
      Self {
        source,
        index: 0,
        start: 0,
        stop: 0,
        state: 0,
        is_attached: false,
        token: Token::Error,
      };
    t.next();
    t
  }

  pub fn token_start(&self) -> usize {
    return self.start;
  }

  pub fn token_stop(&self) -> usize {
    return self.stop;
  }

  pub fn token_is_attached(&self) -> bool {
    return self.is_attached;
  }

  pub fn token(&self) -> Token {
    return self.token;
  }

  pub fn token_span(&self) -> &'a [u8] {
    return unsafe { self.source.get_unchecked(self.start .. self.stop) };
  }

  pub fn next(&mut self) {
    let n = self.source.len();
    let mut s = self.state;
    let mut i = self.index;

    let start;
    loop {
      if is_start(s) { start = i - 1; break; }
      if i == n { start = i; s = 0; break; }
      s = TRANSITION[unsafe { *self.source.get_unchecked(i) } as usize][s as usize];
      i += 1;
    }

    let mut last_state;
    let stop;
    loop {
      last_state = s;
      if i == n { stop = i; s = 0; break; }
      s = TRANSITION[unsafe { *self.source.get_unchecked(i) } as usize][s as usize];
      i += 1;
      if is_start(s) || ! is_continue(s) { stop = i - 1; break; }
    }

    let is_attached = start == self.stop;

    self.index = i;
    self.start = start;
    self.stop = stop;
    self.state = s;
    self.is_attached = is_attached;

    self.token =
      match last_state {
        0 => Token::Eof,
        F =>
          // can look at first char to see quote kind
          Token::DoubleQuote,
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
            b"<-" => Token::Set,
            b"<<" => Token::Shl,
            b">>" => Token::Shr,
            _ => Token::Error,
          },
        9 =>
          match unsafe { *self.source.get_unchecked(start) } {
            b'.' => Token::Field,
            b':' => Token::StaticField,
            _ =>
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
                b"var" => Token::Var,
                b"while" => Token::While,
                _ => Token::Symbol,
              }
          },
        _ => Token::Error,
      }
  }
}
