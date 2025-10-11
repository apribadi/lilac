use crate::neon::*;

static SOURCE: &'static [u8; 16] =
  b"foo(x + y) == 13";

const A: u8 = 10;
const B: u8 = 11;
const C: u8 = 12;
const D: u8 = 13;
const E: u8 = 14;
const F: u8 = 15;

static KIND: [u8; 128] = [
  0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 0, 0, 0, 0, 0,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  1, 6, C, 3, 6, 6, 6, D, 4, 4, 6, 5, 4, 5, 7, 6,
  B, B, B, B, B, B, B, B, B, B, 8, 4, 6, 6, 6, 6,
  6, A, A, A, A, A, A, A, A, A, A, A, A, A, A, A,
  A, A, A, A, A, A, A, A, A, A, A, 4, F, 4, 6, 9,
  E, A, A, A, A, A, A, A, A, A, A, A, A, A, A, A,
  A, A, A, A, A, A, A, A, A, A, A, 4, 6, 4, 6, 0,
];

// states
//
// 0 - start
// 1 - illegal char
// 2 - comment
// 3 - punctuation
// 4 - unused
// 5 - unused
// 6 - unused
// 7 - unused
// 8 - unused
// 9 - unused
// A - unused
// B - unused
// C - unused
// D - unused
// E - unused
// F - unused

static TABLE: [[u8; 16]; 16] = [
// 0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F
// S  I  Z  C  P  +  O  .  :  _  A  N  Q  .a :a
  [1, 1, 1, 3, 1, 1, 1, 1, 1, 1, 1, 1, C, 1, 1, 0], // illegal
  [0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, C, 0, 0, 0], // space        \t sp
  [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, C, 0, 0, 0], // line feed    \n
  [3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, C, 3, 3, 0], // hash         #
  [4, 4, 4, 3, 4, 4, 4, 4, 4, 4, 4, 4, C, 4, 4, 0], // punctuation  ( ) , ; [ ] { }
  [5, 5, 5, 3, 5, 6, 6, 6, 6, 5, 5, 5, C, 5, 5, 0], // sign         +-
  [6, 6, 6, 3, 6, 6, 6, 6, 6, 6, 6, 6, C, 6, 6, 0], // operator     ! $ % & * / < = > ? @ ^ | ~
  [7, 7, 7, 3, 7, B, 7, 6, 6, 7, 7, B, C, 7, 7, 0], // dot          .
  [8, 8, 8, 3, 8, 8, 8, 6, 6, 8, 8, 8, C, 8, 8, 0], // colon        :
  [9, 9, 9, 3, 9, 9, 9, D, E, A, A, 9, C, D, E, 0], // underscore   _
  [A, A, A, 3, A, A, A, D, E, A, A, A, C, D, E, 0], // alphabet     A ... Z a ... z
  [B, B, B, 3, B, B, B, B, B, A, A, B, C, D, E, 0], // digit        0 1 2 3 4 5 6 7 7 8 9
  [C, C, C, 3, C, C, C, C, C, C, C, C, 2, C, C, 0], // double quote "
  [1, 1, 1, 3, 1, 1, 1, 1, 1, 1, 1, 1, C, 1, 1, 0], // single quote '
  [1, 1, 1, 3, 1, 1, 1, 1, 1, 1, 1, 1, C, 1, 1, 0], // back quote   `
  [1, 1, 1, 3, 1, 1, 1, 1, 1, 1, 1, 1, C, 1, 1, 0], // back slash   \
];

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
