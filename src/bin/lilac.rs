#[allow(dead_code)]
fn parse(source: &str) {
  let mut buf = lilac::parse_sexp::parse(source.as_bytes());
  for f in buf.pop_list(buf.len()) {
    print!("{}\n", f);
  }
}

#[allow(dead_code)]
fn compile(source: &str) {
  let mut store = oxcart::Store::new();
  let module = lilac::compile_pass1::compile(source.as_bytes(), &mut store.arena());

  for (i, x) in module.code.iter().enumerate() {
    print!("%{} {}\n", i, x);
  }

  print!("\n");
}

fn main() {
  compile("
    fun fib(n) {
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
