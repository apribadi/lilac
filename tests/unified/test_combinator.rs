use crate::util;
use expect_test::expect;

#[test]
fn test_combinator() {
  let mut out = String::new();

  util::dump(&mut out, "
    fun select(p, x, y) { p ? x : y }
  ");

  expect![[r#"
      %0 ENTRY 3 : [Bool, Abstract, Abstract] -> Some([Abstract])
      %1 = POP : Value Bool
      %2 = POP : Value Abstract
      %3 = POP : Value Abstract
      %4 COND %1
      %5 ==> GOTO %7
      %6 ==> GOTO %10
      %7 LABEL 0 : []
      %8 PUT %3
      %9 RET
      %10 LABEL 0 : []
      %11 PUT %2
      %12 RET
  "#]].assert_eq(out.drain(..).as_ref());

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
      %0 ENTRY 3 : [Abstract, Fun([Abstract], None), Fun([Abstract], None)] -> Some([Abstract])
      %1 = POP : Value Abstract
      %2 = POP : Value Fun([Abstract], None)
      %3 = POP : Value Fun([Abstract], None)
      %4 PUT %1
      %5 CALL %2
      %6 ==> GOTO %7
      %7 LABEL 1 : [Bool]
      %8 = POP : Value Bool
      %9 COND %8
      %10 ==> GOTO %12
      %11 ==> GOTO %15
      %12 LABEL 0 : []
      %13 PUT %1
      %14 TAIL-CALL %3
      %15 LABEL 0 : []
      %16 PUT %1
      %17 RET
  "#]].assert_eq(out.drain(..).as_ref());
}
