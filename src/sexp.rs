pub enum Sexp {
  Atom(Box<[u8]>),
  List(Box<[Sexp]>),
}

impl Sexp {
  pub fn atom(x: &[u8]) -> Self {
    Self::Atom(Box::from(x))
  }

  pub fn from_array<const N: usize>(x: [Self; N]) -> Self {
    let x: Box<[Self]> = Box::from(x);
    Self::List(x)
  }
}

impl std::fmt::Display for Sexp {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Atom(x) => {
        write!(f, "{}", str::from_utf8(x).unwrap())
      }
      Self::List(x) => {
        write!(f, "(")?;
        for (i, y) in x.iter().enumerate() {
          if i != 0 {
            write!(f, " ")?;
          }
          write!(f, "{}", y)?;
        }
        write!(f, ")")?;
        Ok(())
      }
    }
  }
}
