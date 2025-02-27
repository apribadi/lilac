// pub(crate) use oxcart::Store;
pub(crate) use oxcart::Arena;

pub(crate) use pop::ptr;
pub(crate) use core::iter::zip;
// pub(crate) use core::convert::identity;

pub(crate) use crate::byte_slice::*;
pub(crate) use crate::buf::Buf;


/*
pub(crate) fn singleton<T>(x: &T) -> &[T] {
  core::slice::from_ref(x)
}
*/

/*
pub(crate) fn map<'a, T, U>(
    arena: &mut Arena<'a>,
    x: impl ExactSizeIterator<Item = T>,
    f: impl FnMut(T) -> U
  ) -> &'a mut [U]
{
  arena.slice_from_iter(x.map(f))
}
*/
