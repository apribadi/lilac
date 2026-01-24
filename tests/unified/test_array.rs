use crate::util;
use expect_test::expect;

#[test]
fn test_array() {
  let mut out = String::new();

  util::dump(&mut out, "
    fun foo(x) {
      x[0] = 13
      x[1]
    }
  ");

  expect![[r#"
      === fun foo : Fun(Array[i64]) -> (i64) ===
      %0 LABEL 1 : (Array[i64])
      %1 = GET 0 : Array[i64]
      %2 = 0 : i64
      %3 = 13 : i64
      %4 %1 [ %2 ] <- %3
      %5 = 1 : i64
      %6 = %1 [ %5 ] : i64
      %7 PUT 0 %6
      %8 RET
  "#]].assert_eq(out.drain(..).as_ref());

  util::dump(&mut out, "
    fun sum(x) {
      var y = 0
      var i = 0
      let n = len(x)

      loop {
        if ! (i < n) {
          break
        }

        y = y + x[i]
      }

      return y
    }
  ");

  expect![[r#"
      === fun sum : Fun(Array[i64]) -> (i64) ===
      %0 LABEL 1 : (Array[i64])
      %1 = GET 0 : Array[i64]
      %2 = 0 : i64
      %3 = LOCAL %2 : Local i64
      %4 = 0 : i64
      %5 = LOCAL %4 : Local i64
      %6 = CONST len : Fun(Array[i64]) -> (i64)
      %7 PUT 0 %1
      %8 CALL %6
      %9 ==> GOTO %10
      %10 LABEL 1 : (i64)
      %11 = GET 0 : i64
      %12 ==> GOTO %13
      %13 LABEL 0 : ()
      %14 = [ %5 ] : i64
      %15 = %14 < %11 : bool
      %16 = ! %15 : bool
      %17 COND %16
      %18 ==> GOTO %22
      %19 ==> GOTO %20
      %20 LABEL 0 : ()
      %21 ==> GOTO %29
      %22 LABEL 0 : ()
      %23 = [ %3 ] : i64
      %24 = [ %5 ] : i64
      %25 = %1 [ %24 ] : i64
      %26 = %23 + %25 : i64
      %27 [ %3 ] <- %26
      %28 ==> GOTO %13
      %29 LABEL 0 : ()
      %30 = [ %3 ] : i64
      %31 PUT 0 %30
      %32 RET
  "#]].assert_eq(out.drain(..).as_ref());
}
