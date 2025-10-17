use oxcart::Store;
use oxcart::Arena;

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

fn parse_expr_ast<'a>(source: &str, arena: &mut Arena<'a>) -> lilac::ast::Expr<'a> {
  lilac::ast::parse_expr(source.as_bytes(), arena)
}

fn main() {
  let mut store = Store::new();
  let mut arena = store.arena();

  let e = parse_expr_ast("1 + 2 * 3 - 4", &mut arena);
  let e = lilac::compile::compile(e);

  for (i, x) in e.iter().enumerate() {
    print!("%{} {}\n", i, x);
  }

  /*
  print!("{:?}\n", parse_expr_ast("a + b * c - d", &mut arena));
  print!("{}\n", parse_stmt("let x = 1 + 2 * e"));
  print!("{}\n", parse_expr("x == y ? a != b ? 1 + 1 : 2 * 2 : 3 / 3"));
  print!("{}\n", parse_expr("z != - 2 * a[x.foo - 1] + 3 * ! - y.bar - 10"));
  print!("{}\n", parse_expr("a & b | c ^ d"));
  print!("{}\n", parse_expr("a || b && c ? 1 : 2"));
  print!("{}\n", parse_expr("1 +"));
  print!("{}\n", parse_expr("+ 1"));
  */
}
