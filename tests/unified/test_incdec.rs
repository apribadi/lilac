use crate::util;
use expect_test::expect;

#[test]
fn test_loop() {
  let mut out = String::new();

  util::dump(&mut out, "
    fun foo(n) {
      var n = n
      let a = n ++
      let b = n --
      let c = ++ n
      let d = -- n
      return a + b + c + d
    }
  ");

  expect![[r#"
      === fun foo : Fun(I64) -> (I64) ===
      %0 LABEL 1 : (I64)
      %1 = GET 0 : I64
      %2 = LOCAL %1 : Local I64
      %3 = [ %2 ] : I64
      %4 = ++ %3 : I64
      %5 [ %2 ] <- %4
      %6 = [ %2 ] : I64
      %7 = -- %6 : I64
      %8 [ %2 ] <- %7
      %9 = [ %2 ] : I64
      %10 = ++ %9 : I64
      %11 [ %2 ] <- %10
      %12 = [ %2 ] : I64
      %13 = -- %12 : I64
      %14 [ %2 ] <- %13
      %15 = %3 + %6 : I64
      %16 = %15 + %10 : I64
      %17 = %16 + %13 : I64
      %18 PUT 0 %17
      %19 RET
  "#]].assert_eq(out.drain(..).as_ref());
}
