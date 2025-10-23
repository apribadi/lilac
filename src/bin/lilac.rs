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
  parse("if x { let y = 1 y + 2 }");
  parse("if x { let y = 1 y + 2 } else { 4 }");

  compile("loop { break 1, 2, 3 }");

  compile("x ? 1 : y ? 2 : z ? 3 : 4");

  compile("1 + loop { 1 }");

  compile("1 + loop { return }");
}
