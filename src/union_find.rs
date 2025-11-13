use std::cell::Cell;
use std::mem::replace;
use std::ops::Index;
use std::ops::IndexMut;
use crate::buf::Buf;

pub struct UnionFind<T>(Buf<Node<T>>);

#[derive(Debug)]
enum Node<T> {
  Link(Cell<u32>),
  Root(T),
}

impl<T> Node<T> {
  unsafe fn root_unchecked(self) -> T {
    if let Node::Root(value) = self {
      return value;
    }

    unsafe { std::hint::unreachable_unchecked() };
  }

  unsafe fn root_unchecked_ref(&self) -> &T {
    if let Node::Root(value) = self {
      return value;
    }

    unsafe { std::hint::unreachable_unchecked() };
  }

  unsafe fn root_unchecked_mut_ref(&mut self) -> &mut T {
    if let Node::Root(value) = self {
      return value;
    }

    unsafe { std::hint::unreachable_unchecked() };
  }
}

impl<T> UnionFind<T> {
  pub fn new() -> Self {
    return Self(Buf::new());
  }

  pub fn put(&mut self, value: T) -> u32 {
    let n = self.0.len();
    self.0.put(Node::Root(value));
    return n;
  }

  unsafe fn find_unchecked(&self, index: u32) -> u32 {
    debug_assert!(index < self.0.len());

    // path splitting

    if let Node::Link(a) = unsafe { self.0.get_unchecked(index) } {
      let mut a = a;
      let mut i = a.get();

      while let Node::Link(b) = unsafe { self.0.get_unchecked(i) } {
        i = b.get();
        a.set(i);
        a = b;
      }

      return i;
    }

    return index;
  }

  unsafe fn data_unchecked(&self, index: u32) -> &T {
    debug_assert!(index < self.0.len());

    return unsafe { self.0.get_unchecked(index).root_unchecked_ref() };
  }

  unsafe fn data_unchecked_mut(&mut self, index: u32) -> &mut T {
    debug_assert!(index < self.0.len());

    return unsafe { self.0.get_unchecked_mut(index).root_unchecked_mut_ref() };
  }

  /// Returns a mutable reference to the value for `index`'s equivalence class.
  /// If `other` is in a different equivalence class, combines the two
  /// eqivalence classes and removes and returns the old value for `other`'s
  /// equivalence class.

  pub fn union(&mut self, index: u32, other: u32) -> (&mut T, Option<T>) {
    let n = self.0.len();

    assert!(index < n && other < n);

    // index by rank - all links point to lower indices

    let i = unsafe { self.find_unchecked(index) };
    let j = unsafe { self.find_unchecked(other) };

    if i == j {
      return (unsafe { self.data_unchecked_mut(i) }, None);
    } else if i < j {
      let a = replace(unsafe { self.0.get_unchecked_mut(j) }, Node::Link(Cell::new(i)));
      return (unsafe { self.data_unchecked_mut(i) }, Some(unsafe { a.root_unchecked() }));
    } else {
      let a = replace(unsafe { self.0.get_unchecked_mut(i) }, Node::Link(Cell::new(j)));
      let a = replace(unsafe { self.0.get_unchecked_mut(j) }, a);
      return (unsafe { self.data_unchecked_mut(j) }, Some(unsafe { a.root_unchecked() }));
    }
  }
}

impl<T> Index<u32> for UnionFind<T> {
  type Output = T;

  fn index(&self, index: u32) -> &T {
    assert!(index < self.0.len());

    return unsafe { self.data_unchecked(self.find_unchecked(index)) };
  }
}

impl<T> IndexMut<u32> for UnionFind<T> {
  fn index_mut(&mut self, index: u32) -> &mut T {
    assert!(index < self.0.len());

    return unsafe { self.data_unchecked_mut(self.find_unchecked(index)) };
  }
}

pub fn foo(t: &mut UnionFind<u32>, i: u32) -> &mut u32 {
  return &mut t[i];
}

impl<T: std::fmt::Display> std::fmt::Display for UnionFind<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    for i in 0 .. self.0.len() {
      write!(f, "{}: ", i)?;
      match &self.0[i] {
        Node::Link(a) => write!(f, "=> {}\n", a.get())?,
        Node::Root(a) => write!(f, "{}\n", a)?,
      }
    }

    return Ok(());
  }
}
