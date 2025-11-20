use crate::util;
use expect_test::expect;

#[test]
fn test_combinator() {
  let mut out = String::new();

  util::dump(&mut out, "
    fun select(p, x, y) { p ? x : y }
    fun max(x, y) { x >= y ? x : y }
    fun foo(x, y) { x >= 0 ? y : 0 }
  ");

  expect![[r#"
      %0 ENTRY 3
      %1 = POP : Value Bool
      %2 = POP : Value Abstract
      %3 = POP : Value Abstract
      %4 COND %1
      %5 ==> GOTO %7
      %6 ==> GOTO %10
      %7 LABEL 0
      %8 PUT %3
      %9 RET
      %10 LABEL 0
      %11 PUT %2
      %12 RET
      %13 ENTRY 2
      %14 = POP : Value I64
      %15 = POP : Value I64
      %16 = %14 >= %15 : Value Bool
      %17 COND %16
      %18 ==> GOTO %20
      %19 ==> GOTO %23
      %20 LABEL 0
      %21 PUT %15
      %22 RET
      %23 LABEL 0
      %24 PUT %14
      %25 RET
      %26 ENTRY 2
      %27 = POP : Value I64
      %28 = POP : Value Abstract
      %29 = 0 : Value I64
      %30 = %27 >= %29 : Value Bool
      %31 COND %30
      %32 ==> GOTO %34
      %33 ==> GOTO %38
      %34 LABEL 0
      %35 = 0 : Value I64
      %36 PUT %35
      %37 RET
      %38 LABEL 0
      %39 PUT %28
      %40 RET
  "#]].assert_eq(out.drain(..).as_ref());
}
