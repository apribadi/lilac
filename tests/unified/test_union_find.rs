use expect_test::expect;
use lilac::union_find::UnionFind;

fn init(n: u32, x: impl IntoIterator<Item = (u32, u32)>) -> UnionFind<String> {
  let mut t = UnionFind::new();

  for i in 0 .. n {
    assert!(i == t.put(format!("{}", i)));
  }

  for (a, b) in x {
    match t.union(a, b) {
      (_, None) => {}
      (a, Some(b)) => { *a = format!("({} + {})", *a, b); }
    }
  }

  return t;
}

#[test]
fn test_1() {
  let t = init(2, [(0, 1)]);

  expect![[r#"
      0: (0 + 1)
      1: => 0
  "#]].assert_eq(&t.to_string());
}

#[test]
fn test_2() {
  let t = init(2, [(1, 0)]);

  expect![[r#"
      0: (1 + 0)
      1: => 0
  "#]].assert_eq(&t.to_string());
}

#[test]
fn test_3() {
  let t =
    init(10, [
      (0, 1),
      (1, 2),
      (2, 3),
      (3, 4),
      (4, 5),
      (5, 6),
      (6, 7),
      (7, 8),
      (8, 9),
    ]);

  expect![[r#"
      0: (((((((((0 + 1) + 2) + 3) + 4) + 5) + 6) + 7) + 8) + 9)
      1: => 0
      2: => 0
      3: => 0
      4: => 0
      5: => 0
      6: => 0
      7: => 0
      8: => 0
      9: => 0
  "#]].assert_eq(&t.to_string());
}

#[test]
fn test_4() {
  let t =
    init(10, [
      (9, 8),
      (8, 7),
      (7, 6),
      (6, 5),
      (5, 4),
      (4, 3),
      (3, 2),
      (2, 1),
      (1, 0),
    ]);

  expect![[r#"
      0: (((((((((9 + 8) + 7) + 6) + 5) + 4) + 3) + 2) + 1) + 0)
      1: => 0
      2: => 1
      3: => 2
      4: => 3
      5: => 4
      6: => 5
      7: => 6
      8: => 7
      9: => 8
  "#]].assert_eq(&t.to_string());

  let _ = &t[9];

  expect![[r#"
      0: (((((((((9 + 8) + 7) + 6) + 5) + 4) + 3) + 2) + 1) + 0)
      1: => 0
      2: => 0
      3: => 1
      4: => 2
      5: => 3
      6: => 4
      7: => 5
      8: => 6
      9: => 7
  "#]].assert_eq(&t.to_string());

  let _ = &t[8];
  let _ = &t[9];

  expect![[r#"
      0: (((((((((9 + 8) + 7) + 6) + 5) + 4) + 3) + 2) + 1) + 0)
      1: => 0
      2: => 0
      3: => 0
      4: => 0
      5: => 1
      6: => 2
      7: => 3
      8: => 4
      9: => 5
  "#]].assert_eq(&t.to_string());
}

#[test]
fn test_5() {
  let t =
    init(10, [
      (0, 1),
      (2, 3),
      (4, 5),
      (6, 7),
      (8, 9),
      (9, 0),
    ]);

  expect![[r#"
      0: ((8 + 9) + (0 + 1))
      1: => 0
      2: (2 + 3)
      3: => 2
      4: (4 + 5)
      5: => 4
      6: (6 + 7)
      7: => 6
      8: => 0
      9: => 8
  "#]].assert_eq(&t.to_string());
}
