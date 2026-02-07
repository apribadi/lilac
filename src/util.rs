pub const fn usize_u32_saturating_cast(x: usize) -> u32 {
  if (x as u32) as usize == x { x as u32 } else { u32::MAX }
}

#[inline(always)]
pub fn enumerate<T: IntoIterator>(iter: T) -> impl Iterator<Item = (u32, T::Item)> {
  Enumerate { iter: iter.into_iter(), count: 0 }
}

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
    Some((i, a))
  }
}
