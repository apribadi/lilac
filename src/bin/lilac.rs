#[allow(dead_code)]
fn parse(source: &str) {
  print!("{}\n", lilac::parse::parse_expr_sexp(source.as_bytes()));
}

#[allow(dead_code)]
fn compile(source: &str) {
  let mut store = oxcart::Store::new();
  let mut arena = store.arena();

  let items = lilac::ast::parse(source.as_bytes(), &mut arena);

  for lilac::ast::Item::Fundef(f) in items {
    let code = lilac::compile::compile(f);

    for (i, x) in code.iter().enumerate() {
      print!("%{} {}\n", i, x);
    }

    print!("\n");
  }
}

fn main() {
  compile("
    fun {
      var n = n
      var a = 1
      var b = 0
      loop {
        if n == 0 { return b }
        let c = a + b
        a <- b
        b <- c
        n <- n - 1
      }
    }
  ");

  compile("
    fun { loop { break 1, 2, 3 } }
    fun { loop { continue } }
    fun { loop { return 1, 2, 3 } }
    fun { 1 + 2 }
    fun { let z = x >= y ? x : y return f(z)}
    ");
  /*



  /*
  compile("if x { let y = 1 f(y) }");
  compile("if x { let y = 1 y + 2 } else { 4 }");

  compile("loop { break 1, 2, 3 }");

  compile("x ? 1 : y ? 2 : z ? 3 : 4");

  compile("1 + loop { 1 }");

  compile("1 + loop { return }");
  */
  */
}
