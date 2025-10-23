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
  compile("loop { 1 1 }");
  compile("loop { break }");
  compile("loop { continue }");
  compile("loop { return }");

  compile("
    loop {
      if x {
        break 1
      } else
        return 2
      }
    }");

  compile("
    if n == 0 {
      return 0
    } else {
      var n = n
      var a = 1
      var b = 0
      loop {
        let c = a + b
        a <- b
        b <- c
        n <- n - 1
        if n == 0 { return b }
      }
    }
  ");


  /*
  compile("if x { let y = 1 f(y) }");
  compile("if x { let y = 1 y + 2 } else { 4 }");

  compile("loop { break 1, 2, 3 }");

  compile("x ? 1 : y ? 2 : z ? 3 : 4");

  compile("1 + loop { 1 }");

  compile("1 + loop { return }");
  */
}
