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
  const MAX_LEN: u32 = {
    if size_of::<T>() == 0 || isize::MAX as usize / size_of::<T>() > u32::MAX as usize {
      u32::MAX
    } else {
      (isize::MAX as usize / size_of::<T>()) as u32
    }
  };

  pub const EMPTY: Self = Self {
    ptr: ptr::NULL,
    len: 0,
    _phantom_data: PhantomData,
  };

  pub fn init(n: u32, f: impl FnMut(u32) -> T) -> Self {
    assert!(n <= Self::MAX_LEN);

    let p =
      if size_of::<T>() != 0 && n != 0 {
        unsafe { global::alloc_slice::<T>(n as usize) }
      } else {
        ptr::NULL
      };

    let mut a = p;
    let mut f = f;

    for i in 0 .. n {
      let x = f(i);
      unsafe { a.write(x) };
      a += 1;
    }

    return Self { ptr: p, len: n, _phantom_data: PhantomData };
  }

  pub fn new(iter: impl IntoIterator<IntoIter: ExactSizeIterator<Item = T>>) -> Self {
    let mut iter = iter.into_iter();
    let n = iter.len();

    assert!(n <= Self::MAX_LEN as usize);

    let r = Self::init(n as u32, |_| iter.next().unwrap());

    debug_assert!(iter.next().is_none());

    return r;
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

    self.ptr = ptr::NULL;
    self.len = 0;

    if needs_drop::<T>() {
      let mut a = p;
      let mut k = n;
      while k > 0 {
        unsafe { a.drop_in_place() };
        a = a + 1;
        k = k - 1;
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

impl<T> Default for Arr<T> {
  #[inline(always)]
  fn default() -> Self {
    return Self::EMPTY;
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

impl<'a, T> IntoIterator for &'a Arr<T> {
  type Item = &'a T;
  type IntoIter = Iter<'a, T>;

  fn into_iter(self) -> Self::IntoIter {
    return self.iter();
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
    let n = self.len;

    return (n as usize, Some(n as usize));
  }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {
  #[inline(always)]
  fn len(&self) -> usize {
    return self.len as usize;
  }
}

// TODO: remove?
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
