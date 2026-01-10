use crate::util;
use expect_test::expect;

#[test]
fn test_fib_loop() {
  let mut out = String::new();

  util::dump(&mut out, "
    fun fib(n) {
      var a = 1
      var b = 0
      var n = n
      loop {
        if n == 0 { return b }
        let c = a + b
        a = b
        b = c
        n = n - 1
      }
    }
  ");

  expect![[r#"
      === fun fib : TypeScheme(0, Fun(Tuple([I64]), Tuple([I64]))) ===
      %0 LABEL 1 : (I64)
      %1 = GET 0 : Value I64
      %2 = 1 : Value I64
      %3 = LOCAL %2 : Local I64
      %4 = 0 : Value I64
      %5 = LOCAL %4 : Local I64
      %6 = LOCAL %1 : Local I64
      %7 ==> GOTO %8
      %8 LABEL 0 : ()
      %9 = [ %6 ] : Value I64
      %10 = 0 : Value I64
      %11 = %9 == %10 : Value Bool
      %12 COND %11
      %13 ==> GOTO %19
      %14 ==> GOTO %15
      %15 LABEL 0 : ()
      %16 = [ %5 ] : Value I64
      %17 PUT 0 %16
      %18 RET
      %19 LABEL 0 : ()
      %20 = [ %3 ] : Value I64
      %21 = [ %5 ] : Value I64
      %22 = %20 + %21 : Value I64
      %23 = [ %5 ] : Value I64
      %24 [ %3 ] <- %23
      %25 [ %5 ] <- %22
      %26 = [ %6 ] : Value I64
      %27 = 1 : Value I64
      %28 = %26 - %27 : Value I64
      %29 [ %6 ] <- %28
      %30 ==> GOTO %8
  "#]].assert_eq(out.drain(..).as_ref());
}

#[test]
fn test_fib_tailcall() {
  let mut out = String::new();

  util::dump(&mut out, "
    fun aux(a, b, n) {
      if n == 0 {
        b
      } else {
        aux(b, a + b, n - 1)
      }
    }

    fun fib(n) {
      aux(1, 0, n)
    }
  ");

  expect![[r#"
      === fun aux : TypeScheme(0, Fun(Tuple([I64, I64, I64]), Tuple([I64]))) ===
      %0 LABEL 3 : (I64, I64, I64)
      %1 = GET 0 : Value I64
      %2 = GET 1 : Value I64
      %3 = GET 2 : Value I64
      %4 = 0 : Value I64
      %5 = %3 == %4 : Value Bool
      %6 COND %5
      %7 ==> GOTO %9
      %8 ==> GOTO %18
      %9 LABEL 0 : ()
      %10 = %1 + %2 : Value I64
      %11 = 1 : Value I64
      %12 = %3 - %11 : Value I64
      %13 = CONST aux : Value Fun (I64, I64, I64) -> (I64)
      %14 PUT 0 %2
      %15 PUT 1 %10
      %16 PUT 2 %12
      %17 TAIL-CALL %13
      %18 LABEL 0 : ()
      %19 PUT 0 %2
      %20 RET
      === fun fib : TypeScheme(0, Fun(Tuple([I64]), Tuple([I64]))) ===
      %21 LABEL 1 : (I64)
      %22 = GET 0 : Value I64
      %23 = 1 : Value I64
      %24 = 0 : Value I64
      %25 = CONST aux : Value Fun (I64, I64, I64) -> (I64)
      %26 PUT 0 %23
      %27 PUT 1 %24
      %28 PUT 2 %22
      %29 TAIL-CALL %25
  "#]].assert_eq(out.drain(..).as_ref());
}
