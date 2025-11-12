use std::cell::Cell;
use std::ops::Index;
use std::ops::IndexMut;
use crate::buf::Buf;

pub struct UnionFind<T> {
  buf: Buf<Node<T>>,
}

enum Node<T> {
  Data(T),
  Indirect(Cell<u32>),
}

impl<T> UnionFind<T> {
  pub fn add(&mut self, value: T) -> u32 {
    unimplemented!()
  }

  pub fn get(&self, index: u32) -> &T {
    unimplemented!()
  }

  pub fn get_mut(&mut self, index: u32) -> &mut T {
    unimplemented!()
  }

  pub fn union<F>(&mut self, a: u32, b: u32, f: F)
  where
    F: FnMut(&mut T, &mut T) -> Option<T>
  {
    unimplemented!()
  }
}

impl<T> Index<u32> for UnionFind<T> {
  type Output = T;

  fn index(&self, index: u32) -> &T {
    return self.get(index);
  }
}

impl<T> IndexMut<u32> for UnionFind<T> {
  fn index_mut(&mut self, index: u32) -> &mut T {
    return self.get_mut(index);
  }
}
