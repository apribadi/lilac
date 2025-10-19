fn parse(source: &str) {
  print!("{}\n", lilac::parse::parse_stmt_sexp(source.as_bytes()));
}

fn parse_ast(source: &str) {
  print!("{:?}\n", lilac::ast::parse_stmt(source.as_bytes(), &mut oxcart::Store::new().arena()));
}

fn compile(source: &str) {
  let mut store = oxcart::Store::new();
  let mut arena = store.arena();

  let a = lilac::ast::parse_expr(source.as_bytes(), &mut arena);
  let a = lilac::compile::compile(a);

  for (i, x) in a.iter().enumerate() {
    print!("%{} {}\n", i, x);
  }

  print!("\n");
}

fn main() {
  parse("1 * 2 + 3");
  parse("1 + 2 * 3");
  parse("foo().bar.baz <- 1 + qux()");
  parse("a <- 1 * 2 + 3");
  parse("foo(0)[1 + 2] <- 3 + 4");
  parse_ast("a <- 1 * 2 + 3");

  compile("1 + 2 * 3 != 4");
  compile("f(1, 2 + g(4), 3) != x");
  compile("f(1, 2 + g(4), 3)");
  compile("x == y && f(1, 2, 3)");
  compile("x != y ? 1 : a != b ? 2 : 3");
  compile("(1 + 2).foo != 2");
  compile("(1 + 2).foofoofoofoo != 2");
  compile("1 == 1 && 2 != 2");
  compile("! (1 == 1 && 2 != 2)");
}
