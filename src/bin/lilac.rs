static SOURCE: &'static str =
  "\
123
# blah blah blah
...123
123...
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
1_000_000
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
  let mut t = lilac::lex::Lexer::new(SOURCE.as_bytes());

  loop {
    print!("{:?} {} {}\n", t.token(), t.token_is_attached(), str::from_utf8(t.token_slice()).unwrap());
    if t.token() == lilac::token::Token::Eof { break; }
    t.next();
  }
}
