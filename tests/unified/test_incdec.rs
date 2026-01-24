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
      === fun foo : Fun(i64) -> (i64) ===
      %0 LABEL 1 : (i64)
      %1 = GET 0 : i64
      %2 = LOCAL %1 : Local i64
      %3 = [ %2 ] : i64
      %4 = ++ %3 : i64
      %5 [ %2 ] <- %4
      %6 = [ %2 ] : i64
      %7 = -- %6 : i64
      %8 [ %2 ] <- %7
      %9 = [ %2 ] : i64
      %10 = ++ %9 : i64
      %11 [ %2 ] <- %10
      %12 = [ %2 ] : i64
      %13 = -- %12 : i64
      %14 [ %2 ] <- %13
      %15 = %3 + %6 : i64
      %16 = %15 + %10 : i64
      %17 = %16 + %13 : i64
      %18 PUT 0 %17
      %19 RET
  "#]].assert_eq(out.drain(..).as_ref());
}
