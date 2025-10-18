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
  compile("f(1, 2, 3) != x");
  compile("f(1, 2, 3)");
  compile("x == y && f(1, 2, 3)");
  compile("x != y ? 1 : a != b ? 2 : 3");
  compile("(1 + 2).foo != 2");
  compile("(1 + 2).foofoofoofoo != 2");
  compile("1 == 1 && 2 != 2");
  compile("! (1 == 1 && 2 != 2)");
}

/*
fn parse_stmt(source: &str) -> lilac::sexp::Sexp {
  lilac::parse::parse_stmt(
      &mut lilac::lexer::Lexer::new(source.as_bytes()),
      &mut lilac::parse::EmitSexp
    )
}

fn parse_expr(source: &str) -> lilac::sexp::Sexp {
  lilac::parse::parse_expr(
      &mut lilac::lexer::Lexer::new(source.as_bytes()),
      &mut lilac::parse::EmitSexp
    )
}
*/

