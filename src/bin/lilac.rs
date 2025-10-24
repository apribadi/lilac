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
    fun fib(n) {
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

  /*
  compile("
    fun foo() { loop { break 1, 2, 3 } }
    fun bar() { loop { continue } }
    fun baz() { loop { return 1, 2, 3 } }
    fun qux() { let z = x >= y ? x : y return f(z)}
  ");

  compile("fun foo(x) { f(x + 1) }");
  compile("fun foo(x) { return f(x + 1) }");
  compile("fun foo(x) { loop { break f(x + 1) } }");
  compile("fun foo(x) { loop { loop { return f(x + 1) } } }");
  compile("fun foo(x) { if x != 0 { f(x) } }");
  compile("fun foo(x, y) { x >= y ? x : y }");
  compile("fun foo(x, y) { f(x >= y ? x : y) }");
  */

  compile("
    fun foo(x, y) {
      let a = x + y
      let b = x - y
      f(a, b)
      let c, d = g(a, b)
      h(c - d)
    }
  ");
}
