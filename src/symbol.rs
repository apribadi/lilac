pub struct Symbol(pub u64);

const HASHER: foldhash::quality::FixedState =
  foldhash::quality::FixedState::with_seed(0);

impl Symbol {
  pub fn from_bytes(s: &[u8]) -> Self {
    Self(<foldhash::quality::FixedState as std::hash::BuildHasher>::hash_one(&HASHER, s))
  }
}

impl std::fmt::Display for Symbol {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Symbol({:#X})", self.0)
  }
}
