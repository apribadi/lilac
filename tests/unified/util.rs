pub(crate) fn dump(out: &mut impl std::fmt::Write, source: &str) {
  let mut store = oxcart::Store::new();
  let mut arena = store.arena();

  let code = lilac::compile_pass1::compile(source.as_bytes(), &mut arena);

  for (i, inst) in code.iter().enumerate() {
    write!(out, "%{} {}\n", i, inst).unwrap();
  }
}
