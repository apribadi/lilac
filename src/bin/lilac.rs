static A: &'static str =
  "x == y ? a != b ? 1 + 1 : 2 * 2 : 3 / 3";

static B: &'static str =
  "z != - 2 * a[x.foo - 1] + 3 * ! - y.bar - 10";

static C: &'static str =
  "a & b | c ^ d";

static D: &'static str =
  "a || b && c ? 1 : 2";

static E: &'static str =
  "1 + ";

static F: &'static str =
  "+ 1";

fn parse_expr(source: &str) -> lilac::sexp::Sexp {
  lilac::parse::parse_expr(
      &mut lilac::lexer::Lexer::new(source.as_bytes()),
      &mut lilac::parse::DumpSexp
    )
}

fn main() {
  print!("{}\n", parse_expr(A));
  print!("{}\n", parse_expr(B));
  print!("{}\n", parse_expr(C));
  print!("{}\n", parse_expr(D));
  print!("{}\n", parse_expr(E));
  print!("{}\n", parse_expr(F));
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
