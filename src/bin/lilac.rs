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
  compile("1 + 2 * 3 != 4");
  /*
  compile("f(1, 2 + g(4), 3) != x");
  compile("f(1, 2 + g(4), 3)");
  compile("x == y && f(1, 2, 3)");
  compile("x != y ? 1 : a != b ? 2 : 3");
  compile("(1 + 2).foo != 2");
  compile("(1 + 2).foofoofoofoo != 2");
  compile("1 == 1 && 2 != 2");
  compile("! (1 == 1 && 2 != 2)");
  */
}

/*

fn main() {
  print!("{}\n", lilac::parse::parse_expr_sexp("x == y && f(1 + 2 * 3, 1 * 2 + 3)"));
  print!("{}\n", lilac::parse::parse_stmt_sexp("let foo = x == y && f(1 + 2 * 3, 1 * 2 + 3)"));
}
*/
