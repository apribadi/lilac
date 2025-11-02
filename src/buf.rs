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
use core::ptr;

#[derive(Clone, Copy)]
pub struct Idx(NonZeroU32);

pub struct Buf<T> {
  ptr: pop::ptr,
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
      ptr: pop::ptr::NULL,
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
  fn grow(old_p: *mut T, old_c: u32) -> (*mut T, u32) {
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
    let p = self.ptr.as_mut_ptr::<T>();
    let c = self.cap;
    let n = self.len;

    if n == c {
      let (p, c) = Self::grow(p, c);
      self.ptr = pop::ptr::from(p);
      self.cap = c;
      unsafe { ptr::write(p.wrapping_add(n as usize), value) };
    } else {
      unsafe { ptr::write(p.wrapping_add(n as usize), value) };
    }

    self.len = n + 1;

    return Idx(unsafe { NonZeroU32::new_unchecked(n + 1) });
  }

  #[inline(always)]
  pub fn pop(&mut self) -> T {
    let p = self.ptr.as_mut_ptr::<T>();
    let n = self.len;

    assert!(n != 0);

    self.len = n - 1;

    return unsafe { ptr::read(p.wrapping_add((n - 1) as usize)) };
  }

  pub fn pop_multi(&mut self, k: u32) -> PopMulti<'_, T> {
    let p = self.ptr.as_mut_ptr::<T>();
    let n = self.len;

    assert!(k <= n);

    self.len = n - k;

    return PopMulti { ptr: pop::ptr::from(p.wrapping_add((n - k) as usize)), len: k, _phantom_data: PhantomData };
  }

  pub fn reset(&mut self) {
    let p = self.ptr.as_mut_ptr::<T>();
    let c = self.cap;
    let n = self.len;

    self.ptr = pop::ptr::NULL;
    self.cap = if size_of::<T>() == 0 { u32::MAX } else { 0 };
    self.len = 0;

    if needs_drop::<T>() {
      let mut a = p;
      let mut n = n;
      while n > 0 {
        unsafe { ptr::drop_in_place(a) };
        a = a.wrapping_add(1);
        n = n - 1;
      }
    }

    if size_of::<T>() != 0 && c != 0 {
      let size = c as usize * size_of::<T>();
      let layout = unsafe { Layout::from_size_align_unchecked(size, align_of::<T>()) };
      unsafe { dealloc(p as *mut u8, layout) };
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
    let p = self.ptr.as_const_ptr::<T>();
    let n = self.len;
    let i = index.0.get();

    assert!(i <= n);

    return unsafe { &*p.wrapping_add((i - 1) as usize) };
  }
}

impl<T> IndexMut<Idx> for Buf<T> {
  #[inline(always)]
  fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
    let p = self.ptr.as_mut_ptr::<T>();
    let n = self.len;
    let i = index.0.get();

    assert!(i <= n);

    return unsafe { &mut *p.wrapping_add((i - 1) as usize) };
  }
}

pub struct PopMulti<'a, T> {
  ptr: pop::ptr,
  len: u32,
  _phantom_data: PhantomData<&'a mut T>,
}

impl<'a, T> Drop for PopMulti<'a, T> {
  fn drop(&mut self) {
    if needs_drop::<T>() {
      let mut a = self.ptr.as_mut_ptr::<T>();
      let mut n = self.len;
      while n > 0 {
        unsafe { ptr::drop_in_place(a) };
        a = a.wrapping_add(1);
        n = n - 1;
      }
    }
  }
}

impl<'a, T> Iterator for PopMulti<'a, T> {
  type Item = T;

  #[inline(always)]
  fn next(&mut self) -> Option<T> {
    let p = self.ptr.as_const_ptr::<T>();
    let n = self.len;

    if n == 0 {
      return None;
    }

    let value = unsafe { ptr::read(p) };

    self.ptr = pop::ptr::from(p.wrapping_add(1));
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
