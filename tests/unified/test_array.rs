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
      FUN foo %0
      %0 LABEL 1 : [Array(I64)]
      %1 = GET 0 : Value Array(I64)
      %2 = 0 : Value I64
      %3 = 13 : Value I64
      %4 %1 [ %2 ] <- %3
      %5 = 1 : Value I64
      %6 = %1 [ %5 ] : Value I64
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
      FUN sum %0
      %0 LABEL 1 : [Array(I64)]
      %1 = GET 0 : Value Array(I64)
      %2 = 0 : Value I64
      %3 = LOCAL %2 : Local I64
      %4 = 0 : Value I64
      %5 = LOCAL %4 : Local I64
      %6 = CONST len : Value Fun([Array(I64)], None)
      %7 PUT 0 %1
      %8 CALL %6
      %9 ==> GOTO %10
      %10 LABEL 1 : [I64]
      %11 = GET 0 : Value I64
      %12 ==> GOTO %13
      %13 LABEL 0 : []
      %14 = [ %5 ] : Value I64
      %15 = %14 < %11 : Value Bool
      %16 = ! %15 : Value Bool
      %17 COND %16
      %18 ==> GOTO %22
      %19 ==> GOTO %20
      %20 LABEL 0 : []
      %21 ==> GOTO %29
      %22 LABEL 0 : []
      %23 = [ %3 ] : Value I64
      %24 = [ %5 ] : Value I64
      %25 = %1 [ %24 ] : Value I64
      %26 = %23 + %25 : Value I64
      %27 [ %3 ] <- %26
      %28 ==> GOTO %13
      %29 LABEL 0 : []
      %30 = [ %3 ] : Value I64
      %31 PUT 0 %30
      %32 RET
  "#]].assert_eq(out.drain(..).as_ref());
}
