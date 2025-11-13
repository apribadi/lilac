use std::cell::Cell;
use std::ops::Index;
use std::ops::IndexMut;
use crate::buf::Buf;

pub struct UnionFind<T>(Buf<Node<T>>);

enum Node<T> {
  Data(T),
  Link(Cell<u32>),
}

impl<T> UnionFind<T> {
  pub fn new() -> Self {
    return Self(Buf::new());
  }

  pub fn put(&mut self, value: T) -> u32 {
    let n = self.0.len();
    self.0.put(Node::Data(value));
    return n;
  }

  unsafe fn find_unchecked(&self, index: u32) -> u32 {
    let mut i = index;

    while let Node::Link(a) = unsafe { self.0.get_unchecked(i) } {
      i = a.get();
    }

    return i;
  }

  pub fn union<F>(&mut self, a: u32, b: u32) -> (&mut T, Option<T>) {
    unimplemented!()
  }
}

impl<T> Index<u32> for UnionFind<T> {
  type Output = T;

  fn index(&self, index: u32) -> &T {
    unimplemented!()
  }
}

impl<T> IndexMut<u32> for UnionFind<T> {
  fn index_mut(&mut self, index: u32) -> &mut T {
    unimplemented!()
  }
}
