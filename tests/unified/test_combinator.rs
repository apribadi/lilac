use crate::util;
use expect_test::expect;

#[test]
fn test_id() {
  let mut out = String::new();

  util::dump(&mut out, "
    fun id(x) { return x }
    fun id3(x, y, z) { return id(x), id(y), id(z) }
  ");

  expect![[r#"
      === fun id : forall '0 . Fun('0) -> ('0) ===
      %0 LABEL 1 : ('0)
      %1 = GET 0 : '0
      %2 PUT 0 %1
      %3 RET
      === fun id3 : forall '0 '1 '2 . Fun('0, '1, '2) -> ('0, '1, '2) ===
      %4 LABEL 3 : ('0, '1, '2)
      %5 = GET 0 : '0
      %6 = GET 1 : '1
      %7 = GET 2 : '2
      %8 = CONST id : Fun('0) -> ('0)
      %9 PUT 0 %5
      %10 CALL %8
      %11 ==> GOTO %12
      %12 LABEL 1 : ('0)
      %13 = GET 0 : '0
      %14 = CONST id : Fun('1) -> ('1)
      %15 PUT 0 %6
      %16 CALL %14
      %17 ==> GOTO %18
      %18 LABEL 1 : ('1)
      %19 = GET 0 : '1
      %20 = CONST id : Fun('2) -> ('2)
      %21 PUT 0 %7
      %22 CALL %20
      %23 ==> GOTO %24
      %24 LABEL 1 : ('2)
      %25 = GET 0 : '2
      %26 PUT 0 %13
      %27 PUT 1 %19
      %28 PUT 2 %25
      %29 RET
  "#]].assert_eq(out.drain(..).as_ref());
}

#[test]
fn test_apply() {
  let mut out = String::new();

  util::dump(&mut out, "
    fun apply1(f, x) { f(x) }
    fun apply2(f, x, y) { f(x, y) }
    fun id(x) { x }
    fun flip(x, y) { y, x }
    fun foo(x) { apply1(id, x) }
    fun bar(x, y) { apply2(flip, x, y) }
    fun baz(x) { apply2(apply1, id, x) }
  ");

  expect![[r#"
      === fun apply1 : forall '0 '1 . Fun(Fun('0) -> '1, '0) -> '1 ===
      %0 LABEL 2 : (Fun('0) -> '1, '0)
      %1 = GET 0 : Fun('0) -> '1
      %2 = GET 1 : '0
      %3 PUT 0 %2
      %4 TAIL-CALL %1
      === fun apply2 : forall '0 '1 '2 . Fun(Fun('0, '1) -> '2, '0, '1) -> '2 ===
      %5 LABEL 3 : (Fun('0, '1) -> '2, '0, '1)
      %6 = GET 0 : Fun('0, '1) -> '2
      %7 = GET 1 : '0
      %8 = GET 2 : '1
      %9 PUT 0 %7
      %10 PUT 1 %8
      %11 TAIL-CALL %6
      === fun id : forall '0 . Fun('0) -> ('0) ===
      %12 LABEL 1 : ('0)
      %13 = GET 0 : '0
      %14 PUT 0 %13
      %15 RET
      === fun flip : forall '0 '1 . Fun('0, '1) -> ('1, '0) ===
      %16 LABEL 2 : ('0, '1)
      %17 = GET 0 : '0
      %18 = GET 1 : '1
      %19 PUT 0 %18
      %20 PUT 1 %17
      %21 RET
      === fun foo : forall '0 . Fun('0) -> ('0) ===
      %22 LABEL 1 : ('0)
      %23 = GET 0 : '0
      %24 = CONST id : Fun('0) -> ('0)
      %25 = CONST apply1 : Fun(Fun('0) -> ('0), '0) -> ('0)
      %26 PUT 0 %24
      %27 PUT 1 %23
      %28 TAIL-CALL %25
      === fun bar : forall '0 '1 . Fun('0, '1) -> ('1, '0) ===
      %29 LABEL 2 : ('0, '1)
      %30 = GET 0 : '0
      %31 = GET 1 : '1
      %32 = CONST flip : Fun('0, '1) -> ('1, '0)
      %33 = CONST apply2 : Fun(Fun('0, '1) -> ('1, '0), '0, '1) -> ('1, '0)
      %34 PUT 0 %32
      %35 PUT 1 %30
      %36 PUT 2 %31
      %37 TAIL-CALL %33
      === fun baz : forall '0 . Fun('0) -> ('0) ===
      %38 LABEL 1 : ('0)
      %39 = GET 0 : '0
      %40 = CONST apply1 : Fun(Fun('0) -> ('0), '0) -> ('0)
      %41 = CONST id : Fun('0) -> ('0)
      %42 = CONST apply2 : Fun(Fun(Fun('0) -> ('0), '0) -> ('0), Fun('0) -> ('0), '0) -> ('0)
      %43 PUT 0 %40
      %44 PUT 1 %41
      %45 PUT 2 %39
      %46 TAIL-CALL %42
  "#]].assert_eq(out.drain(..).as_ref());
}

#[test]
fn test_select() {
  let mut out = String::new();

  util::dump(&mut out, "
    fun select(p, x, y) { p ? x : y }
    fun relu(x) { select(x >= 0, x, 0) + 0 }
  ");

  expect![[r#"
      === fun select : forall '0 . Fun(Bool, '0, '0) -> ('0) ===
      %0 LABEL 3 : (Bool, '0, '0)
      %1 = GET 0 : Bool
      %2 = GET 1 : '0
      %3 = GET 2 : '0
      %4 COND %1
      %5 ==> GOTO %7
      %6 ==> GOTO %10
      %7 LABEL 0 : ()
      %8 PUT 0 %3
      %9 RET
      %10 LABEL 0 : ()
      %11 PUT 0 %2
      %12 RET
      === fun relu : Fun(I64) -> (I64) ===
      %13 LABEL 1 : (I64)
      %14 = GET 0 : I64
      %15 = 0 : I64
      %16 = %14 >= %15 : Bool
      %17 = 0 : I64
      %18 = CONST select : Fun(Bool, I64, I64) -> (I64)
      %19 PUT 0 %16
      %20 PUT 1 %14
      %21 PUT 2 %17
      %22 CALL %18
      %23 ==> GOTO %24
      %24 LABEL 1 : (I64)
      %25 = GET 0 : I64
      %26 = 0 : I64
      %27 = %25 + %26 : I64
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
      === fun foo : forall '0 . Fun('0, Fun('0) -> (Bool), Fun('0) -> ('0)) -> ('0) ===
      %0 LABEL 3 : ('0, Fun('0) -> (Bool), Fun('0) -> ('0))
      %1 = GET 0 : '0
      %2 = GET 1 : Fun('0) -> (Bool)
      %3 = GET 2 : Fun('0) -> ('0)
      %4 PUT 0 %1
      %5 CALL %2
      %6 ==> GOTO %7
      %7 LABEL 1 : (Bool)
      %8 = GET 0 : Bool
      %9 COND %8
      %10 ==> GOTO %12
      %11 ==> GOTO %15
      %12 LABEL 0 : ()
      %13 PUT 0 %1
      %14 TAIL-CALL %3
      %15 LABEL 0 : ()
      %16 PUT 0 %1
      %17 RET
  "#]].assert_eq(out.drain(..).as_ref());
}
