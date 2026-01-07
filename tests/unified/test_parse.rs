use crate::util;
use expect_test::expect;

#[test]
fn test_parse() {
  let mut out = String::new();

  util::parse_sexp(&mut out, "
    fun foo(n) {
      var n = n
      let _ = n ++
      let _ = n --
    }
  ");

  expect!["(fun foo (n) ((var n n) (let (_) ((%post ++ n))) (let (_) ((%post -- n)))))"].assert_eq(out.drain(..).as_ref());
}
