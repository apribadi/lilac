fn parse_stmt(source: &str) -> lilac::sexp::Sexp {
  lilac::parse::parse_stmt(
      &mut lilac::lexer::Lexer::new(source.as_bytes()),
      &mut lilac::parse::DumpSexp
    )
}

fn parse_expr(source: &str) -> lilac::sexp::Sexp {
  lilac::parse::parse_expr(
      &mut lilac::lexer::Lexer::new(source.as_bytes()),
      &mut lilac::parse::DumpSexp
    )
}

fn main() {
  print!("{}\n", parse_stmt("let x = 1 + 2 * e"));
  print!("{}\n", parse_expr("x == y ? a != b ? 1 + 1 : 2 * 2 : 3 / 3"));
  print!("{}\n", parse_expr("z != - 2 * a[x.foo - 1] + 3 * ! - y.bar - 10"));
  print!("{}\n", parse_expr("a & b | c ^ d"));
  print!("{}\n", parse_expr("a || b && c ? 1 : 2"));
  print!("{}\n", parse_expr("1 +"));
  print!("{}\n", parse_expr("+ 1"));
}

/*
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
  let mut t = lilac::lexer::Lexer::new(SOURCE.as_bytes());

  loop {
    print!("{:?} {} {}\n", t.token(), t.token_is_attached(), str::from_utf8(t.token_slice()).unwrap());
    if t.token() == lilac::token::Token::Eof { break; }
    t.next();
  }
}
*/
