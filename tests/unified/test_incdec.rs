use crate::util;
use expect_test::expect;

#[test]
fn test_loop() {
  let mut out = String::new();

  util::dump(&mut out, "
    fun foo(n) {
      var n = n
      n += 2
      n -= 2
      return n
    }
  ");

  expect![[r#"
      === fun foo : Fun([I64], Some([I64])) ===
      %0 LABEL 1 : [I64]
      %1 = GET 0 : Value I64
      %2 = LOCAL %1 : Local I64
      %3 = [ %2 ] : Value I64
      %4 = 2 : Value I64
      %5 = %3 + %4 : Value I64
      %6 [ %2 ] <- %5
      %7 = [ %2 ] : Value I64
      %8 = 2 : Value I64
      %9 = %7 - %8 : Value I64
      %10 [ %2 ] <- %9
      %11 = [ %2 ] : Value I64
      %12 PUT 0 %11
      %13 RET
  "#]].assert_eq(out.drain(..).as_ref());
}
