use crate::util;
use expect_test::expect;

#[test]
fn test_fib() {
  let mut out = String::new();

  util::dump(&mut out, "
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

  expect![[r#"
      %0 ENTRY 1
      %1 = POP : Abstract
      %2 = 1 : I64
      %3 = DEF-LOCAL %2
      %4 = 0 : I64
      %5 = DEF-LOCAL %4
      %6 = DEF-LOCAL %1
      %7 ==> GOTO %8
      %8 LABEL 0
      %9 = [ %6 ] : Abstract
      %10 = 0 : I64
      %11 = %9 <= %10 : Abstract
      %12 COND %11
      %13 ==> GOTO %19
      %14 ==> GOTO %15
      %15 LABEL 0
      %16 = [ %5 ] : Abstract
      %17 PUT %16
      %18 RET
      %19 LABEL 0
      %20 = [ %3 ] : Abstract
      %21 = [ %5 ] : Abstract
      %22 = %20 + %21 : Abstract
      %23 = [ %5 ] : Abstract
      %24 [ %3 ] <- %23
      %25 [ %5 ] <- %22
      %26 = [ %6 ] : Abstract
      %27 = 1 : I64
      %28 = %26 - %27 : Abstract
      %29 [ %6 ] <- %28
      %30 ==> GOTO %8
  "#]].assert_eq(out.drain(..).as_ref());

  util::dump(&mut out, "
    fun fib(n) {
      aux(1, 0, n)
    }

    fun aux(a, b, n) {
      if n <= 0 {
        b
      } else {
        aux(b, a + b, n - 1)
      }
    }
  ");

  expect![[r#"
      %0 ENTRY 1
      %1 = POP : Abstract
      %2 = CONST aux : Abstract
      %3 = 1 : I64
      %4 = 0 : I64
      %5 PUT %3
      %6 PUT %4
      %7 PUT %1
      %8 TAIL-CALL %2
      %9 ENTRY 3
      %10 = POP : Abstract
      %11 = POP : Abstract
      %12 = POP : Abstract
      %13 = 0 : I64
      %14 = %12 <= %13 : Abstract
      %15 COND %14
      %16 ==> GOTO %18
      %17 ==> GOTO %27
      %18 LABEL 0
      %19 = CONST aux : Abstract
      %20 = %10 + %11 : Abstract
      %21 = 1 : I64
      %22 = %12 - %21 : Abstract
      %23 PUT %11
      %24 PUT %20
      %25 PUT %22
      %26 TAIL-CALL %19
      %27 LABEL 0
      %28 PUT %11
      %29 RET
  "#]].assert_eq(out.drain(..).as_ref());
}
