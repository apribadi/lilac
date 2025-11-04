extern crate alloc;

use alloc::alloc::handle_alloc_error;
use core::alloc::Layout;
use core::iter::FusedIterator;
use core::marker::PhantomData;
use core::mem::needs_drop;
use core::ops::Index;
use core::ops::IndexMut;
use pop::ptr;

pub struct Buf<T> {
  ptr: ptr,
  cap: u32,
  len: u32,
  _phantom_data: PhantomData<T>,
}

unsafe impl<T: Send> Send for Buf<T> {
}

unsafe impl<T: Sync> Sync for Buf<T> {
}

#[inline(always)]
fn increment_size_class(n: usize) -> usize {
  debug_assert!(2 <= n && n <= isize::MAX as usize);
  let m = 2 * n - 1;
  let k = usize::BITS - 1 - m.leading_zeros();
  let a = 1 << k;
  let b = a >> 1;
  return a | b & m;
}

impl<T> Buf<T> {
  pub const fn new() -> Self {
    return Self {
      ptr: ptr::NULL,
      cap: if size_of::<T>() == 0 { u32::MAX } else { 0 },
      len: 0,
      _phantom_data: PhantomData,
    };
  }

  #[inline(always)]
  pub fn is_empty(&self) -> bool {
    return self.len == 0;
  }

  #[inline(always)]
  pub fn len(&self) -> u32 {
    return self.len;
  }

  #[inline(never)]
  #[cold]
  fn grow(old_p: ptr, old_c: u32) -> (ptr, u32) {
    assert!(size_of::<T>() != 0);

    let max_c =
      usize::min(
        u32::MAX as usize,
        isize::MAX as usize / size_of::<T>());

    if old_c == 0 {
      let new_c = 16;

      assert!(new_c <= max_c);

      let new_s = size_of::<T>() * new_c;
      let new_l = unsafe { Layout::from_size_align_unchecked(new_s, align_of::<T>()) };
      let new_p = unsafe { pop::alloc(new_l) };

      if new_p.is_null() {
        match handle_alloc_error(new_l) {
        }
      }

      return (new_p, new_c as u32);
    } else {
      let old_s = size_of::<T>() * old_c as usize;
      let old_l = unsafe { Layout::from_size_align_unchecked(old_s, align_of::<T>()) };
      let new_c = increment_size_class(old_s) / size_of::<T>();

      assert!(new_c <= max_c);

      let new_s = new_c * size_of::<T>();
      let new_l = unsafe { Layout::from_size_align_unchecked(new_s, align_of::<T>()) };
      let new_p = unsafe { pop::realloc(old_p, old_l, new_s) };

      if new_p.is_null () {
        match handle_alloc_error(new_l) {
        }
      }

      return (new_p, new_c as u32);
    }
  }

  #[inline(always)]
  pub fn put(&mut self, value: T) {
    let p = self.ptr;
    let c = self.cap;
    let n = self.len;

    if n == c {
      let (p, c) = Self::grow(p, c);
      self.ptr = p;
      self.cap = c;
      unsafe { (p + size_of::<T>() * n as usize).write(value) };
    } else {
      unsafe { (p + size_of::<T>() * n as usize).write(value) };
    }

    self.len = n + 1;
  }

  #[inline(always)]
  pub fn pop(&mut self) -> T {
    let p = self.ptr;
    let n = self.len;

    assert!(n != 0);

    self.len = n - 1;

    return unsafe { (p + size_of::<T>() * (n as usize - 1)).read::<T>() };
  }

  pub fn pop_list(&mut self, k: u32) -> PopList<'_, T> {
    let p = self.ptr;
    let n = self.len;

    assert!(k <= n);

    self.len = n - k;

    let p = p + size_of::<T>() * (n - k) as usize;

    return PopList { ptr: p, len: k, _phantom_data: PhantomData };
  }

  #[inline(always)]
  pub fn last(&self) -> &T {
    let p = self.ptr;
    let n = self.len;

    assert!(n != 0);

    return unsafe { (p + size_of::<T>() * (n as usize - 1)).as_ref::<T>() };
  }

  #[inline(always)]
  pub fn last_mut(&mut self) -> &mut T {
    let p = self.ptr;
    let n = self.len;

    assert!(n != 0);

    return unsafe { (p + size_of::<T>() * (n as usize - 1)).as_mut_ref::<T>() };
  }

  pub fn reset(&mut self) {
    let p = self.ptr;
    let c = self.cap;
    let n = self.len;

    self.ptr = ptr::NULL;
    self.cap = if size_of::<T>() == 0 { u32::MAX } else { 0 };
    self.len = 0;

    if needs_drop::<T>() {
      let mut a = p;
      let mut n = n;
      while n > 0 {
        unsafe { a.drop_in_place::<T>() };
        a = a + size_of::<T>();
        n = n - 1;
      }
    }

    if size_of::<T>() != 0 && c != 0 {
      let size = size_of::<T>() * c as usize;
      let layout = unsafe { Layout::from_size_align_unchecked(size, align_of::<T>()) };
      unsafe { pop::dealloc(p, layout) };
    }
  }

  pub fn iter(&self) -> Iter<'_, T> {
    return Iter { ptr: self.ptr, len: self.len, _phantom_data: PhantomData };
  }
}

impl<T> Drop for Buf<T> {
  fn drop(&mut self) {
    self.reset();
  }
}

impl<T> Index<u32> for Buf<T> {
  type Output = T;

  #[inline(always)]
  fn index(&self, index: u32) -> &Self::Output {
    let p = self.ptr;
    let n = self.len;

    assert!(index < n);

    return unsafe { (p + size_of::<T>() * index as usize).as_ref::<T>() }
  }
}

impl<T> IndexMut<u32> for Buf<T> {
  #[inline(always)]
  fn index_mut(&mut self, index: u32) -> &mut Self::Output {
    let p = self.ptr;
    let n = self.len;

    assert!(index < n);

    return unsafe { (p + size_of::<T>() * index as usize).as_mut_ref::<T>() }
  }
}

pub struct Iter<'a, T> {
  ptr: ptr,
  len: u32,
  _phantom_data: PhantomData<&'a T>,
}

pub struct PopList<'a, T> {
  ptr: ptr,
  len: u32,
  _phantom_data: PhantomData<&'a mut T>,
}

impl<'a, T> Drop for PopList<'a, T> {
  fn drop(&mut self) {
    if needs_drop::<T>() {
      for _ in self {
      }
    }
  }
}

impl<'a, T> Iterator for Iter<'a, T> {
  type Item = &'a T;

  #[inline(always)]
  fn next(&mut self) -> Option<&'a T> {
    let p = self.ptr;
    let n = self.len;

    if n == 0 {
      return None;
    }

    self.ptr = p + size_of::<T>();
    self.len = n - 1;

    return Some(unsafe { p.as_ref::<T>() });
  }

  #[inline(always)]
  fn size_hint(&self) -> (usize, Option<usize>) {
    let n = self.len;

    return (n as usize, Some(n as usize));
  }
}

impl<'a, T> Iterator for PopList<'a, T> {
  type Item = T;

  #[inline(always)]
  fn next(&mut self) -> Option<T> {
    let p = self.ptr;
    let n = self.len;

    if n == 0 {
      return None;
    }

    self.ptr = p + size_of::<T>();
    self.len = n - 1;

    return Some(unsafe { p.read::<T>() });

  }

  #[inline(always)]
  fn size_hint(&self) -> (usize, Option<usize>) {
    let n = self.len;

    return (n as usize, Some(n as usize));
  }
}

impl<'a, T> ExactSizeIterator for PopList<'a, T> {
  #[inline(always)]
  fn len(&self) -> usize {
    return self.len as usize;
  }
}

impl<'a, T> FusedIterator for PopList<'a, T> {
}
