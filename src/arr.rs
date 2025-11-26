use core::marker::PhantomData;
use core::mem::needs_drop;
use core::ops::Index;
use core::ops::IndexMut;
use crate::buf::Buf;
use pop::global;
use pop::ptr;

pub struct Arr<T> {
  ptr: ptr<T>,
  len: u32,
  _phantom_data: PhantomData<T>,
}

impl<T> Arr<T> {
  const MAX_LEN: usize = {
    if size_of::<T>() == 0 || isize::MAX as usize / size_of::<T>() > u32::MAX as usize {
      u32::MAX as usize
    } else {
      isize::MAX as usize / size_of::<T>()
    }
  };

  pub const EMPTY: Self = Self {
    ptr: ptr::null(),
    len: 0,
    _phantom_data: PhantomData,
  };

  pub fn new<U, V>(iter: U) -> Self
  where
    U: IntoIterator<IntoIter = V>,
    V: ExactSizeIterator<Item = T>
  {
    let mut iter = iter.into_iter();
    let n = iter.len();

    assert!(n <= Self::MAX_LEN);

    let p =
      if size_of::<T>() != 0 && n != 0 {
        unsafe { global::alloc_slice::<T>(n) }
      } else {
        ptr::null()
      };

    let n = n as u32;
    let mut a = p;

    for _ in 0 .. n {
      unsafe { a.write(iter.next().unwrap()); }
      a += 1;
    }

    debug_assert!(iter.next().is_none());

    return Self { ptr: p, len: n, _phantom_data: PhantomData };
  }

  #[inline(always)]
  pub fn len(&self) -> u32 {
    return self.len;
  }

  #[inline(always)]
  pub unsafe fn get_unchecked(&self, index: u32) -> &T {
    let p = self.ptr;
    let n = self.len;

    debug_assert!(index < n);

    return unsafe { (p + index).as_ref() }
  }

  #[inline(always)]
  pub unsafe fn get_unchecked_mut(&mut self, index: u32) -> &mut T {
    let p = self.ptr;
    let n = self.len;

    debug_assert!(index < n);

    return unsafe { (p + index).as_mut_ref() }
  }

  pub fn iter(&self) -> Iter<'_, T> {
    return Iter { ptr: self.ptr, len: self.len, _phantom_data: PhantomData };
  }
}

impl<T> Drop for Arr<T> {
  fn drop(&mut self) {
    let p = self.ptr;
    let n = self.len;

    self.ptr = ptr::null();
    self.len = 0;

    if needs_drop::<T>() {
      let mut a = p;
      let mut n = n;
      while n > 0 {
        unsafe { a.drop_in_place() };
        a = a + 1;
        n = n - 1;
      }
    }

    if size_of::<T>() != 0 && n != 0 {
      unsafe { global::dealloc_slice(p, n as usize) };
    }
  }
}

impl<T: Clone> Clone for Arr<T> {
  fn clone(&self) -> Self {
    return Self::new(self.iter().map(T::clone));
  }
}

impl<T> Index<u32> for Arr<T> {
  type Output = T;

  #[inline(always)]
  fn index(&self, index: u32) -> &Self::Output {
    let p = self.ptr;
    let n = self.len;

    assert!(index < n);

    return unsafe { (p + index).as_ref() }
  }
}

impl<T> IndexMut<u32> for Arr<T> {
  #[inline(always)]
  fn index_mut(&mut self, index: u32) -> &mut Self::Output {
    let p = self.ptr;
    let n = self.len;

    assert!(index < n);

    return unsafe { (p + index).as_mut_ref() }
  }
}

pub struct Iter<'a, T> {
  ptr: ptr<T>,
  len: u32,
  _phantom_data: PhantomData<&'a T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
  type Item = &'a T;

  #[inline(always)]
  fn next(&mut self) -> Option<Self::Item> {
    let p = self.ptr;
    let n = self.len;

    if n == 0 {
      return None;
    }

    self.ptr = p + 1;
    self.len = n - 1;

    return Some(unsafe { p.as_ref() });
  }

  #[inline(always)]
  fn size_hint(&self) -> (usize, Option<usize>) {
    let n = self.len as usize;

    return (n, Some(n));
  }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {
  #[inline(always)]
  fn len(&self) -> usize {
    return self.len as usize;
  }
}

impl<T> FromIterator<T> for Arr<T> {
  fn from_iter<U: IntoIterator<Item = T>>(iter: U) -> Self {
    let mut buf = Buf::new();
    for item in iter.into_iter() { buf.put(item); }
    return Arr::new(buf.drain());
  }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Arr<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_list().entries(self.iter()).finish()
  }
}
