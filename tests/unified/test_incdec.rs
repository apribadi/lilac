use crate::util;
use expect_test::expect;

#[test]
fn test_loop() {
  let mut out = String::new();

  util::dump(&mut out, "
    fun foo(n) {
      var n = n
      return n
    }
  ");

  expect![[r#"
      === fun foo : Fun([Abstract], Some([Abstract])) ===
      %0 LABEL 1 : [Abstract]
      %1 = GET 0 : Value Abstract
      %2 = LOCAL %1 : Local Abstract
      %3 = [ %2 ] : Value Abstract
      %4 PUT 0 %3
      %5 RET
  "#]].assert_eq(out.drain(..).as_ref());
}
