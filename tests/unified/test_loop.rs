use crate::util;
use expect_test::expect;

#[test]
fn test_loop() {
  let mut out = String::new();

  util::dump(&mut out, "
    fun foo() { loop { break 1 } }
    fun bar() { loop { continue } }
    fun baz() { loop { return 1 } }
    fun qux() { let x, y = loop { break 1, 2 } return x + y }
  ");

  expect![[r#"
      === fun foo : Fun() -> (i64) ===
      %0 LABEL 0 : ()
      %1 ==> GOTO %2
      %2 LABEL 0 : ()
      %3 = 1 : i64
      %4 PUT 0 %3
      %5 RET
      === fun bar : forall '0 . Fun() -> '0 ===
      %6 LABEL 0 : ()
      %7 ==> GOTO %8
      %8 LABEL 0 : ()
      %9 ==> GOTO %8
      === fun baz : Fun() -> (i64) ===
      %10 LABEL 0 : ()
      %11 ==> GOTO %12
      %12 LABEL 0 : ()
      %13 = 1 : i64
      %14 PUT 0 %13
      %15 RET
      === fun qux : Fun() -> (i64) ===
      %16 LABEL 0 : ()
      %17 ==> GOTO %18
      %18 LABEL 0 : ()
      %19 = 1 : i64
      %20 = 2 : i64
      %21 PUT 0 %19
      %22 PUT 1 %20
      %23 ==> GOTO %24
      %24 LABEL 2 : (i64, i64)
      %25 = GET 0 : i64
      %26 = GET 1 : i64
      %27 = %25 + %26 : i64
      %28 PUT 0 %27
      %29 RET
  "#]].assert_eq(out.drain(..).as_ref());
}
