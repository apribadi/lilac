use std::num::NonZeroU32;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TypeVar(pub u32);

unsafe impl tangerine::key::IntoKey for TypeVar {
  type Key = NonZeroU32;

  #[inline(always)]
  fn inject(Self(n): Self) -> Self::Key {
    return NonZeroU32::new(n.wrapping_add(1)).unwrap();
  }

  #[inline(always)]
  unsafe fn project(n: Self::Key) -> Self {
    return Self(n.get().wrapping_sub(1));
  }
}
