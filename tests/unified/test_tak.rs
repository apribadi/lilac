use crate::util;
use expect_test::expect;

#[test]
fn test_tak() {
  let mut out = String::new();

  util::dump(&mut out, "
    fun tak(x, y, z) {
      if y < x {
        return tak(
          tak(x - 1, y, z),
          tak(y - 1, z, x),
          tak(z - 1, x, y)
        )
      } else {
        return z
      }
    }
  ");

  expect![[r#"
      === fun tak : TypeScheme(0, Bool) ===
      %0 LABEL 3 : [I64, I64, I64]
      %1 = GET 0 : Value I64
      %2 = GET 1 : Value I64
      %3 = GET 2 : Value I64
      %4 = %2 < %1 : Value Bool
      %5 COND %4
      %6 ==> GOTO %8
      %7 ==> GOTO %11
      %8 LABEL 0 : []
      %9 PUT 0 %3
      %10 RET
      %11 LABEL 0 : []
      %12 = 1 : Value I64
      %13 = %1 - %12 : Value I64
      %14 = CONST tak : Value Fun([I64, I64, I64], [I64])
      %15 PUT 0 %13
      %16 PUT 1 %2
      %17 PUT 2 %3
      %18 CALL %14
      %19 ==> GOTO %20
      %20 LABEL 1 : [I64]
      %21 = GET 0 : Value I64
      %22 = 1 : Value I64
      %23 = %2 - %22 : Value I64
      %24 = CONST tak : Value Fun([I64, I64, I64], [I64])
      %25 PUT 0 %23
      %26 PUT 1 %3
      %27 PUT 2 %1
      %28 CALL %24
      %29 ==> GOTO %30
      %30 LABEL 1 : [I64]
      %31 = GET 0 : Value I64
      %32 = 1 : Value I64
      %33 = %3 - %32 : Value I64
      %34 = CONST tak : Value Fun([I64, I64, I64], [I64])
      %35 PUT 0 %33
      %36 PUT 1 %1
      %37 PUT 2 %2
      %38 CALL %34
      %39 ==> GOTO %40
      %40 LABEL 1 : [I64]
      %41 = GET 0 : Value I64
      %42 = CONST tak : Value Fun([I64, I64, I64], [I64])
      %43 PUT 0 %21
      %44 PUT 1 %31
      %45 PUT 2 %41
      %46 TAIL-CALL %42
  "#]].assert_eq(out.drain(..).as_ref());
}
