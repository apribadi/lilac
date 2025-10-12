static SOURCE: &'static [u8] =
  b"\
# blah blah blah
fun foo(x: int, y: int) -> int {
  let a = x + y + +100 + -100
  let b = bar(a)
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

// CLASSES
//
// 0 - illegal
// 1 - space        \t sp
// 2 - line feed    \n
// 3 - hash         #
// 4 - punctuation  ( ) , ; [ ] { }
// 5 - plus minus   +-
// 6 - operator     ! $ % & * / < = > ? @ ^ | ~
// 7 - dot          .
// 8 - colon        :
// 9 - underscore   _
// A - alphabet     A ... Z a ... z
// B - digit        0 1 2 3 4 5 6 7 7 8 9
// C - double quote "
// D - single quote '
// E - back quote   `
// F - back slash   \

const CLASS_OF_CHAR: [u8; 128] = [
  0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 0, 0, 0, 0, 0,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  1, 6, C, 3, 6, 6, 6, D, 4, 4, 6, 5, 4, 5, 7, 6,
  B, B, B, B, B, B, B, B, B, B, 8, 4, 6, 6, 6, 6,
  6, A, A, A, A, A, A, A, A, A, A, A, A, A, A, A,
  A, A, A, A, A, A, A, A, A, A, A, 4, F, 4, 6, 9,
  E, A, A, A, A, A, A, A, A, A, A, A, A, A, A, A,
  A, A, A, A, A, A, A, A, A, A, A, 4, 6, 4, 6, 0,
];

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
// C - quote start
// D - number continuation
// E - quote continuation
// F -

const STATE_OF_STATE_OF_CLASS: [[u8; 16]; 16] = [
// states
// 0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F
  [1, 1, 1, 3, 1, 1, 1, 1, 1, 1, 1, 1, E, 1, E, 0], // 0 - illegal
  [0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, E, 0, E, 0], // 1 - space        \t sp
  [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, E, 0, E, 0], // 2 - line feed    \n
  [3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, E, 3, E, 0], // 3 - hash         #
  [4, 4, 4, 3, 4, 4, 4, 4, 4, 4, 4, 4, E, 4, E, 0], // 4 - punctuation  ( ) , ; [ ] { }
  [5, 5, 2, 3, 5, 2, 2, 2, 2, 5, 5, 5, E, 5, E, 0], // 5 - plus minus   +-
  [6, 6, 2, 3, 6, 2, 2, 2, 2, 6, 6, 6, E, 6, E, 0], // 6 - operator     ! $ % & * / < = > ? @ ^ | ~
  [7, 7, 7, 3, 7, 2, 2, 2, 2, 7, 7, D, E, 7, E, 0], // 7 - dot          .
  [8, 8, 8, 3, 8, 2, 2, 2, 2, 8, 8, 8, E, 8, E, 0], // 8 - colon        :
  [A, A, A, 3, A, A, A, A, A, 9, 9, D, E, D, E, 0], // 9 - underscore   _
  [A, A, A, 3, A, A, A, A, A, 9, 9, D, E, D, E, 0], // A - alphabet     A ... Z a ... z
  [B, B, B, 3, B, B, B, B, B, 9, 9, D, E, D, E, 0], // B - digit        0 1 2 3 4 5 6 7 7 8 9
  [C, C, C, 3, C, C, C, C, C, C, C, C, 0, C, 0, 0], // C - double quote "
  [1, 1, 1, 3, 1, 1, 1, 1, 1, 1, 1, 1, E, 1, E, 0], // D - single quote '
  [1, 1, 1, 3, 1, 1, 1, 1, 1, 1, 1, 1, E, 1, E, 0], // E - back quote   `
  [1, 1, 1, 3, 1, 1, 1, 1, 1, 1, 1, 1, E, 1, E, 0], // F - back slash   \
];

static TABLE: [[u8; 16]; 256] = {
  let mut t = [[0u8; 16]; 256];
  let mut i = 0u8;
  loop {
    let k = if i <= 127 { CLASS_OF_CHAR[i as usize] } else { 0 };
    t[i as usize] = STATE_OF_STATE_OF_CLASS[k as usize];
    if i == 255 { break; }
    i += 1;
  }
  t
};

static OUT: [u8; 16] = [
// 0b01 - is_token_start
// 0b10 - is_token
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
  0b00, //
];

pub fn go() {
  let mut s = 0u8;

  for &c in SOURCE.iter() {
    s = TABLE[c as usize][s as usize];
    print!("{:?} {:x}\n", c as char, s);
  }
}


/*

fn show_char(x: impl Iterator<Item = u8>) {
  for c in x {
    print!("{} ", c as char);
  }
  print!("\n");
}

fn show_byte(x: impl Iterator<Item = u8>) {
  for c in x {
    print!("{:X} ", c);
  }
  print!("\n");
}

#[target_feature(enable = "neon")]
pub fn go() {
  print!("Hello!\n");
  show_char(SOURCE.iter().map(|c| *c));
  show_byte(SOURCE.iter().map(|c| KIND[*c as usize]));

  let x = unsafe { vld1q_u8(&raw const SOURCE[0]) };
  let y = foo(x);
  let mut z = [0u8; 16];
  unsafe { aarch64::vst1q_u8(&raw mut z[0], y) };
  show_byte(z.iter().map(|c| *c));
}

#[target_feature(enable = "neon")]
pub fn foo(x: uint8x16_t) -> uint8x16_t {
  let t = unsafe { vld1q_u8_x8(&raw const KIND[0]) };
  return vqtbl8q_u8(t, x);
}
*/
