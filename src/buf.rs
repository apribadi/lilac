extern crate alloc;

use alloc::alloc::alloc;
use alloc::alloc::dealloc;
use alloc::alloc::handle_alloc_error;
use core::alloc::Layout;
use core::iter::FusedIterator;
use core::marker::PhantomData;
use core::mem::needs_drop;
use core::num::NonZeroU32;
use core::ops::Index;
use core::ops::IndexMut;
use pop::ptr;

#[derive(Clone, Copy)]
pub struct Idx(NonZeroU32);

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
  pub fn len(&self) -> u32 {
    return self.len;
  }

  // const MAX_CAP: u32 = isize::MAX as usize / size_of::<T>();

  #[inline(never)]
  #[cold]
  fn grow(old_p: ptr, old_c: u32) -> (ptr, u32) {
    assert!(size_of::<T>() != 0);

    unimplemented!()
    /*

    // TODO: check max capacity

    if old_c == 0 {
      let new_c = 16;
      let new_s = new_c as usize * size_of::<T>();
      let new_l = unsafe { Layout::from_size_align_unchecked(new_s, align_of::<T>()) };
      let new_p = unsafe { alloc(new_l) } as *mut T;
      if new_p.is_null() {
        match handle_alloc_error(new_l) { /* ! */ }
      }
      return (new_p, new_c);
    } else {
      let old_s = old_c as usize * size_of::<T>();
      let old_l = unsafe { Layout::from_size_align_unchecked(old_s, align_of::<T>()) };
      let new_c = old_c * 2;
      let new_s = new_c as usize * size_of::<T>();
      let new_l = unsafe { Layout::from_size_align_unchecked(new_s, align_of::<T>()) };
      let new_p = unsafe { alloc(new_l) } as *mut T;
      if new_p.is_null () {
        match handle_alloc_error(new_l) { /* ! */ }
      }
      unsafe { ptr::copy_nonoverlapping(old_p, new_p, old_c as usize) };
      unsafe { dealloc(old_p as *mut u8, old_l) };
      return (new_p, new_c);
    }
  */
  }

  #[inline(always)]
  pub fn put(&mut self, value: T) -> Idx {
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

    return Idx(unsafe { NonZeroU32::new_unchecked(n + 1) });
  }

  #[inline(always)]
  pub fn pop(&mut self) -> T {
    let p = self.ptr;
    let n = self.len;

    assert!(n != 0);

    self.len = n - 1;

    return unsafe { (p + size_of::<T>() * (n as usize - 1)).read::<T>() };
  }

  pub fn pop_multi(&mut self, k: u32) -> PopMulti<'_, T> {
    let p = self.ptr;
    let n = self.len;

    assert!(k <= n);

    self.len = n - k;

    let p = p + size_of::<T>() * (n - k) as usize;

    return PopMulti { ptr: p, len: k, _phantom_data: PhantomData };
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
}

impl<T> Drop for Buf<T> {
  fn drop(&mut self) {
    self.reset();
  }
}

impl<T> Index<Idx> for Buf<T> {
  type Output = T;

  #[inline(always)]
  fn index(&self, index: Idx) -> &Self::Output {
    let p = self.ptr;
    let n = self.len;
    let i = index.0.get();

    assert!(i <= n);

    return unsafe { (p + size_of::<T>() * (i - 1) as usize).as_ref::<T>() }
  }
}

impl<T> IndexMut<Idx> for Buf<T> {
  #[inline(always)]
  fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
    let p = self.ptr;
    let n = self.len;
    let i = index.0.get();

    assert!(i <= n);

    return unsafe { (p + size_of::<T>() * (i - 1) as usize).as_mut_ref::<T>() }
  }
}

pub struct PopMulti<'a, T> {
  ptr: ptr,
  len: u32,
  _phantom_data: PhantomData<&'a mut T>,
}

impl<'a, T> Drop for PopMulti<'a, T> {
  fn drop(&mut self) {
    if needs_drop::<T>() {
      let mut a = self.ptr;
      let mut n = self.len;
      while n > 0 {
        unsafe { a.drop_in_place::<T>() };
        a = a + size_of::<T>();
        n = n - 1;
      }
    }
  }
}

impl<'a, T> Iterator for PopMulti<'a, T> {
  type Item = T;

  #[inline(always)]
  fn next(&mut self) -> Option<T> {
    let p = self.ptr;
    let n = self.len;

    if n == 0 {
      return None;
    }

    let value = unsafe { p.read::<T>() };

    self.ptr = p + size_of::<T>();
    self.len = n - 1;

    return Some(value);
  }

  #[inline(always)]
  fn size_hint(&self) -> (usize, Option<usize>) {
    let n = self.len;

    return (n as usize, Some(n as usize));
  }
}

impl<'a, T> FusedIterator for PopMulti<'a, T> {
}

impl<'a, T> ExactSizeIterator for PopMulti<'a, T> {
  #[inline(always)]
  fn len(&self) -> usize {
    return self.len as usize;
  }
}
