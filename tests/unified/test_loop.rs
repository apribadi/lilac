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
      %0 ENTRY 0
      %1 ==> GOTO %2
      %2 LABEL 0
      %3 = 1 : Val I64
      %4 PUT %3
      %5 RET
      %6 ENTRY 0
      %7 ==> GOTO %8
      %8 LABEL 0
      %9 ==> GOTO %8
      %10 ENTRY 0
      %11 ==> GOTO %12
      %12 LABEL 0
      %13 = 1 : Val I64
      %14 PUT %13
      %15 RET
      %16 ENTRY 0
      %17 ==> GOTO %18
      %18 LABEL 0
      %19 = 1 : Val I64
      %20 = 2 : Val I64
      %21 PUT %19
      %22 PUT %20
      %23 ==> GOTO %24
      %24 LABEL 2
      %25 = POP : Val Abstract
      %26 = POP : Val Abstract
      %27 = %25 + %26 : Val Abstract
      %28 PUT %27
      %29 RET
  "#]].assert_eq(out.drain(..).as_ref());
}
