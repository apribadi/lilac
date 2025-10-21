#[allow(dead_code)]
fn parse(source: &str) {
  print!("{}\n", lilac::parse::parse_expr_sexp(source.as_bytes()));
}

#[allow(dead_code)]
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
  compile("
    1 +
      loop {
        f(1)
        g(x + y)
        h()
      }
  ");

  /*
  compile("x == y && f(z + 1)");
  compile("g(x == y && f(z + 1))");
  compile("x != y ? 1 : a != b ? 2 : 3");
  compile("1 + (x != y ? 1 : a != b ? 2 : 3)");
  */
}
