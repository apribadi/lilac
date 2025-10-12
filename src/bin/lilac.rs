static SOURCE: &'static [u8] =
  b"\
# blah blah blah
.foo
._foo
:foo
:_foo
:99
`
`
\\
\x00
.
..
...
:
::
:::
.:123
:.123
fun foo(x: int, y: int) -> int {
  let a = x * y
  let b = bar(a)
  let _ = 1 + 1. + .1 + 1.1 + 1.1e10 + 1.1e+10
  let _ = +1 + +1. + +.1 + +1.1 + +1.1e10 + +1.1e+10
  let _ = +. - +.+
  print(\"hello\")
  return a << b
}
\"blah";

fn main() {
  let mut t = lilac::lexer::Lexer::new(&SOURCE);

  loop {
    print!("{:?} {:?}\n", t.token(), str::from_utf8(t.span()).unwrap());
    if t.token() == lilac::token::Token::Eof { break; }
    t.next();
  }
  t.next();
}
