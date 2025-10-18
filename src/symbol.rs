pub struct Symbol(pub u64);

const HASHER: foldhash::quality::FixedState =
  foldhash::quality::FixedState::with_seed(0);

impl Symbol {
  pub fn from_bytes(s: &[u8]) -> Self {
    if s.len() <= 8 {
      let mut buf = [0u8; 8];
      buf[.. s.len()].copy_from_slice(s);
      return Self(u64::from_le_bytes(buf));
    } else {
      let n = <foldhash::quality::FixedState as std::hash::BuildHasher>::hash_one(&HASHER, s);
      return Self(n | 1 << 63);
    }
  }
}

impl std::fmt::Display for Symbol {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if self.0 as isize >= 0 {
      let buf = self.0.to_le_bytes();
      let mut i = 0;
      while i < 8 && buf[i] != 0 { i += 1; }
      write!(f, "{}", str::from_utf8(&buf[.. i]).unwrap())
    } else {
      write!(f, "Symbol({:#X})", self.0)
    }
  }
}
