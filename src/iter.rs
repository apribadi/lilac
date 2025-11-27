struct Enumerate<T> {
  iter: T,
  count: u32,
}

impl<T: Iterator> Iterator for Enumerate<T> {
  type Item = (u32, <T as Iterator>::Item);

  #[inline(always)]
  fn next(&mut self) -> Option<Self::Item> {
    let a = self.iter.next()?;
    let i = self.count;
    self.count = i + 1;
    return Some((i, a));
  }
}

#[inline(always)]
pub fn enumerate<T: IntoIterator>(iter: T) -> impl Iterator<Item = (u32, T::Item)> {
  return Enumerate { iter: iter.into_iter(), count: 0 };
}
