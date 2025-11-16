use std::iter::zip;

pub(crate) fn dump(out: &mut impl std::fmt::Write, source: &str) {
  let mut store = oxcart::Store::new();
  let mut arena = store.arena();

  let code = lilac::compile_pass1::compile(source.as_bytes(), &mut arena);

  let (typing, valtypes) = lilac::typecheck::typecheck(&code);

  for (i, (inst, ty)) in zip(code.iter(), typing.iter()).enumerate() {
    match ty {
      lilac::typecheck::InstType::Nil => {
        write!(out, "%{} {}\n", i, inst).unwrap();
      }
      lilac::typecheck::InstType::Value(ty) => {
        write!(out, "%{} {} : Value {:?}\n", i, inst, valtypes[*ty]).unwrap();
      }
      lilac::typecheck::InstType::Local(ty) => {
        write!(out, "%{} {} : Local {:?}\n", i, inst, valtypes[*ty]).unwrap();
      }
    }
  }
}
