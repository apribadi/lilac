fn parse(source: &str) {
  print!("{}\n", lilac::parse::parse_block_sexp(source.as_bytes()));
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
  parse("{ 1 2 3 4 }");
  parse(
    "\
{
var x = 1 * 2 + 3
f(x + 1)
g()
return 1, 2
}
");

  compile("x == y && f(z + 1)");
}

  /*
  parse("1 * 2 + 3");
  parse("1 + 2 * 3");
  parse("foo().bar.baz <- 1 + qux()");
  parse("a <- 1 * 2 + 3");
  parse("foo(0).bar[1 + 2] <- 3 + 4");

  compile("1 + 2 * 3 != 4");
  compile("f(1, 2 + g(4), 3) != x");
  compile("f(1, 2 + g(4), 3)");
  compile("x == y && f(1, 2, 3)");
  compile("x != y ? 1 : a != b ? 2 : 3");
  compile("(1 + 2).foo != 2");
  compile("(1 + 2).foofoofoofoo != 2");
  compile("1 == 1 && 2 != 2");
  compile("! (1 == 1 && 2 != 2)");
  */
