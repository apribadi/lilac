use std::cell::Cell;
use std::hint::unreachable_unchecked;
use std::mem::replace;
use std::ops::Index;
use std::ops::IndexMut;
use crate::buf::Buf;

pub struct UnionFind<T>(Buf<Node<T>>);

enum Node<T> {
  Link(Cell<u32>),
  Root(T),
}

impl<T> Node<T> {
  unsafe fn root_unchecked(self) -> T {
    let Node::Root(value) = self else { unsafe { unreachable_unchecked() } };
    return value;
  }

  unsafe fn root_unchecked_ref(&self) -> &T {
    let Node::Root(value) = self else { unsafe { unreachable_unchecked() } };
    return value;
  }

  unsafe fn root_unchecked_mut_ref(&mut self) -> &mut T {
    let Node::Root(value) = self else { unsafe { unreachable_unchecked() } };
    return value;
  }
}

impl<T> UnionFind<T> {
  pub fn new() -> Self {
    return Self(Buf::new());
  }

  pub fn len(&self) -> u32 {
    return self.0.len();
  }

  pub fn put(&mut self, value: T) {
    self.0.put(Node::Root(value));
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

  /// Returns a mutable reference to the value for `index`'s equivalence class.
  /// If `other` is in a different equivalence class, combines the two
  /// eqivalence classes and removes and returns the old value for `other`'s
  /// equivalence class.

  pub fn union(&mut self, index: u32, other: u32) -> (&mut T, Option<T>) {
    let n = self.0.len();

    assert!(index < n && other < n);

    let i = unsafe { self.find_unchecked(index) };
    let j = unsafe { self.find_unchecked(other) };

    // index by rank - all links point to lower indices

    if i == j {
      return (unsafe { self.get_root_unchecked_mut(i) }, None);
    } else if i < j {
      let a = replace(unsafe { self.0.get_unchecked_mut(j) }, Node::Link(Cell::new(i)));
      return (unsafe { self.get_root_unchecked_mut(i) }, Some(unsafe { a.root_unchecked() }));
    } else {
      let a = replace(unsafe { self.0.get_unchecked_mut(i) }, Node::Link(Cell::new(j)));
      let a = replace(unsafe { self.0.get_unchecked_mut(j) }, a);
      return (unsafe { self.get_root_unchecked_mut(j) }, Some(unsafe { a.root_unchecked() }));
    }
  }

  pub fn is_equivalent(&self, index: u32, other: u32) -> bool {
    let n = self.0.len();

    assert!(index < n && other < n);

    let i = unsafe { self.find_unchecked(index) };
    let j = unsafe { self.find_unchecked(other) };

    return i == j;
  }

  unsafe fn get_root_unchecked(&self, index: u32) -> &T {
    debug_assert!(index < self.0.len());

    return unsafe { self.0.get_unchecked(index).root_unchecked_ref() };
  }

  unsafe fn get_root_unchecked_mut(&mut self, index: u32) -> &mut T {
    debug_assert!(index < self.0.len());

    return unsafe { self.0.get_unchecked_mut(index).root_unchecked_mut_ref() };
  }
}

impl<T> Index<u32> for UnionFind<T> {
  type Output = T;

  fn index(&self, index: u32) -> &T {
    assert!(index < self.0.len());

    return unsafe { self.get_root_unchecked(self.find_unchecked(index)) };
  }
}

impl<T> IndexMut<u32> for UnionFind<T> {
  fn index_mut(&mut self, index: u32) -> &mut T {
    assert!(index < self.0.len());

    return unsafe { self.get_root_unchecked_mut(self.find_unchecked(index)) };
  }
}

impl<T: std::fmt::Display> std::fmt::Display for UnionFind<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    for i in 0 .. self.0.len() {
      match &self.0[i] {
        Node::Link(a) => write!(f, "{}: => {}\n", i, a.get())?,
        Node::Root(a) => write!(f, "{}: {}\n", i, a)?,
      }
    }
    return Ok(());
  }
}
