use std::num::NonZeroU64;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Symbol(pub NonZeroU64);

const HASHER: foldhash::quality::FixedState =
  foldhash::quality::FixedState::with_seed(0);

impl Symbol {
  pub fn from_bytes(s: &[u8]) -> Self {
    if s.len() == 0 {
      panic!();
    } else if s.len() <= 8 {
      let mut buf = [0u8; 8];
      buf[.. s.len()].copy_from_slice(s);
      let n = u64::from_le_bytes(buf);
      let n = NonZeroU64::new(n).unwrap();
      return Self(n);
    } else {
      let n = <foldhash::quality::FixedState as std::hash::BuildHasher>::hash_one(&HASHER, s);
      let n = n | 1 << 63;
      let n = NonZeroU64::new(n).unwrap();
      return Self(n);
    }
  }

  pub fn from_str(s: &str) -> Self {
    return Self::from_bytes(s.as_bytes());
  }
}

unsafe impl tangerine::key::IntoKey for Symbol {
  type Key = NonZeroU64;

  #[inline(always)]
  fn inject(Self(n): Self) -> Self::Key {
    return n;
  }

  #[inline(always)]
  unsafe fn project(n: Self::Key) -> Self {
    return Self(n);
  }
}

impl std::fmt::Display for Symbol {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let n = self.0.get();
    if n as isize >= 0 {
      let buf = n.to_le_bytes();
      let mut i = 0;
      while i < 8 && buf[i] != 0 { i += 1; }
      write!(f, "{}", str::from_utf8(&buf[.. i]).unwrap())
    } else {
      write!(f, "Symbol({:#X})", n)
    }
  }
}
