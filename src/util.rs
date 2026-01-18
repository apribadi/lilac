#[inline(always)]
pub(crate) fn map<T, U, A, B, F>(x: T, f: F) -> U
where
  T: IntoIterator<Item = A>,
  U: FromIterator<B>,
  F: FnMut(A) -> B,
{
  return x.into_iter().map(f).collect();
}

#[inline(always)]
pub(crate) fn copied<'a, T, U, A>(x: T) -> U
where
  T: IntoIterator<Item = &'a A>,
  U: FromIterator<A>,
  A: Copy + 'a,
{
  return x.into_iter().copied().collect();
}
