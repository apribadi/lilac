use crate::token::Token;

pub struct Lexer<'a> {
  source: &'a [u8],
  token: Token,
  state: u8,
  token_start: isize,
  token_stop: isize,
}

static SOURCE: &'static [u8] =
  b"\
# blah blah blah
fun foo(x: int, y: int) -> int {
  let a = x + y
  let b = bar(a)
  let _ = 1 + 1. + .1 + 1.1 + 1.1e10 + 1.1e+10
  let _ = +1 + +1. + +.1 + +1.1 + +1.1e10 + +1.1e+10
  let _ = +. + +.+
  print(\"hello\")
  return a *** b
}
";

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
  [7, 7, 7, 3, 7, D, 2, 2, 2, 7, 7, D, E, D, E, 7], // 7 - dot          .
  [8, 8, 8, 3, 8, 2, 2, 2, 2, 8, 8, 8, E, 8, E, 8], // 8 - colon        :
  [A, A, A, 3, A, A, A, A, A, 9, 9, D, E, D, E, A], // 9 - underscore   _
  [A, A, A, 3, A, A, A, A, A, 9, 9, D, E, D, E, A], // A - alphabet     A ... Z a ... z
  [B, B, B, 3, B, D, B, B, B, 9, 9, D, E, D, E, B], // B - digit        0 1 2 3 4 5 6 7 7 8 9
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
        token: Token::Error,
        state: 0,
        token_start: -1,
        token_stop: -1,
      };
    // TODO: inline specialized next
    t.next();
    t
  }

  pub fn next(&mut self) {
    let n = self.source.len();
    let mut s = self.state;
    let mut i = (self.token_stop + 1) as usize;
    while ! is_start(s) && i != n {
      s = TABLE[self.source[i] as usize][s as usize];
      i += 1;
    }
    let start = i - 1;
    if i != n {
      s = TABLE[self.source[i] as usize][s as usize];
      i += 1;
      while ! is_start(s) && is_token(s) && i != n {
        s = TABLE[self.source[i] as usize][s as usize];
        i += 1;
      }
    }
    let stop = i - 1;
    self.state = s;
    self.token_start = start as isize;
    self.token_stop = stop as isize;
    if start == stop { self.token = Token::Eof; }
  }

  pub fn token(&self) -> Token {
    return self.token;
  }

  pub fn span(&self) -> &'a [u8] {
    // TODO: unsafe
    return &self.source[self.token_start as usize .. self.token_stop as usize];
  }
}

pub fn go() {
  let mut t = Lexer::new(&SOURCE);

  while t.token() != Token::Eof {
    print!("{}\n", str::from_utf8(t.span()).unwrap());
    t.next();
  }
}

/*
pub fn go() {
  print!("{}\n", str::from_utf8(&SOURCE).unwrap());
  let mut s = 0u8;
  let mut a = 0;
  let mut p = false;
  let mut i = 0;

  for &c in SOURCE.iter() {
    s = TABLE[c as usize][s as usize];
    if p && (is_start(s) || ! is_token(s)) {
      p = false;
      print!("{:x} {}\n", s, str::from_utf8(&SOURCE[a .. i]).unwrap());
    }
    if is_start(s) {
      a = i;
      p = true;
    }
    i += 1;
  }
}
*/
