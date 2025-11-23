struct Enumerate<T> {
  iter: T,
  index: u32,
}

impl<T: Iterator> Iterator for Enumerate<T> {
  type Item = (u32, <T as Iterator>::Item);

  #[inline(always)]
  fn next(&mut self) -> Option<Self::Item> {
    match self.iter.next() {
      None => {
        return None;
      }
      Some(a) => {
        let i = self.index;
        self.index = i + 1;
        return Some((i, a));
      }
    }
  }
}

pub fn enumerate<T: Iterator>(iter: T) -> impl Iterator<Item = (u32, T::Item)> {
  return Enumerate { iter: iter.into_iter(), index: 0 };
}
