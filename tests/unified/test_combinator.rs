use crate::util;
use expect_test::expect;

#[test]
fn test_select() {
  let mut out = String::new();

  util::dump(&mut out, "
    fun select(p, x, y) { p ? x : y }
    fun relu(x) { select(x >= 0, x, 0) + 0 }
  ");

  expect![[r#"
      === fun select : TypeScheme(0, Bool) ===
      %0 LABEL 3 : [Bool, Var(TypeVar(111)), Var(TypeVar(111))]
      %1 = GET 0 : Value Bool
      %2 = GET 1 : Value Var(TypeVar(111))
      %3 = GET 2 : Value Var(TypeVar(111))
      %4 COND %1
      %5 ==> GOTO %7
      %6 ==> GOTO %10
      %7 LABEL 0 : []
      %8 PUT 0 %3
      %9 RET
      %10 LABEL 0 : []
      %11 PUT 0 %2
      %12 RET
      === fun relu : TypeScheme(0, Bool) ===
      %13 LABEL 1 : [I64]
      %14 = GET 0 : Value I64
      %15 = 0 : Value I64
      %16 = %14 >= %15 : Value Bool
      %17 = 0 : Value I64
      %18 = CONST select : Value Fun([Bool, I64, I64], [I64])
      %19 PUT 0 %16
      %20 PUT 1 %14
      %21 PUT 2 %17
      %22 CALL %18
      %23 ==> GOTO %24
      %24 LABEL 1 : [I64]
      %25 = GET 0 : Value I64
      %26 = 0 : Value I64
      %27 = %25 + %26 : Value I64
      %28 PUT 0 %27
      %29 RET
  "#]].assert_eq(out.drain(..).as_ref());
}

#[test]
fn test_foo() {
  let mut out = String::new();

  util::dump(&mut out, "
    fun foo(x, f, g) {
      if f(x) {
        x
      } else {
        g(x)
      }
    }
  ");

  expect![[r#"
      === fun foo : TypeScheme(0, Bool) ===
      %0 LABEL 3 : [Var(TypeVar(111)), Fun([Var(TypeVar(111))], [Bool]), Fun([Var(TypeVar(111))], [Var(TypeVar(111))])]
      %1 = GET 0 : Value Var(TypeVar(111))
      %2 = GET 1 : Value Fun([Var(TypeVar(111))], [Bool])
      %3 = GET 2 : Value Fun([Var(TypeVar(111))], [Var(TypeVar(111))])
      %4 PUT 0 %1
      %5 CALL %2
      %6 ==> GOTO %7
      %7 LABEL 1 : [Bool]
      %8 = GET 0 : Value Bool
      %9 COND %8
      %10 ==> GOTO %12
      %11 ==> GOTO %15
      %12 LABEL 0 : []
      %13 PUT 0 %1
      %14 TAIL-CALL %3
      %15 LABEL 0 : []
      %16 PUT 0 %1
      %17 RET
  "#]].assert_eq(out.drain(..).as_ref());
}
