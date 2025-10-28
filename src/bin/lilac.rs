#[allow(dead_code)]
fn parse(source: &str) {
  print!("{}\n", lilac::parse::parse_expr_sexp(source.as_bytes()));
}

#[allow(dead_code)]
fn compile(source: &str) {
  let mut store = oxcart::Store::new();
  let mut arena = store.arena();

  let item_list = lilac::ast::parse(source.as_bytes(), &mut arena);

  let code = lilac::compile::compile(item_list.into_iter());

  for (i, x) in code.iter().enumerate() {
    print!("%{} {}\n", i, x);
  }

  print!("\n");
}

fn main() {
  compile("
    fun foo(n) {
      var a = 1
      var b = 0
      var n = n
      loop {
        if n <= 0 { return b }
        let c = a + b
        a = b
        b = c
        n = n - 1
      }
    }

    fun bar(n) {
      baz(1, 0, n)
    }

    fun baz(a, b, n) {
      if n <= 0 {
        b
      } else {
        baz(b, a + b, n - 1)
      }
    }
  ");

  /*
  compile("
    fun foo(x, y) {
      let a = x + y
      let b = x - y
      f(a, b)
      let c, d = g(a, b)
      h(c - d)
    }
  ");

  compile("
    fun foo() {
      let x, y, z =
        loop {
          break 1, 2, 3
        }
      return x + y + z
    }");

  compile("
    fun foo(x, y) {
      let a, b = if x >= y { x, y } else { y, x }
      f(a - b)
    }");

  compile("
    fun foo(x, y) {
      let a = x + y
      let b = x - y
      let a, b = b, a
      f(a - b)
    }");

  compile("
    fun foo(x, y) {
      let a = x + y
      let b = x - y
      let a, b = b, a
      let a, b = b, a
      f(a - b)
    }");

  compile("
    fun foo(x, y) {
      let a = x + y
      let b = x - y
      let = g(1)
      let a, b = b, a
      f(a - b)
    }");

  compile("
    fun foo(x, y) {
      let _ = f(x + y)
      let _ = f(x - y)
    }");
  */
}
