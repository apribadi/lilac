pub enum Sexp {
  Atom(Box<[u8]>),
  List(Box<[Sexp]>),
}

impl Sexp {
  pub fn atom(x: impl AsRef<[u8]>) -> Self {
    Self::Atom(Box::from(x.as_ref()))
  }

  pub fn list(x: impl IntoIterator<IntoIter: ExactSizeIterator<Item = Self>>) -> Self {
    Self::List(x.into_iter().collect())
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
