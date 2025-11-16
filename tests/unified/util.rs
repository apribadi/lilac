use std::iter::zip;

pub(crate) fn dump(out: &mut impl std::fmt::Write, source: &str) {
  let mut store = oxcart::Store::new();
  let mut arena = store.arena();

  let code = lilac::compile_pass1::compile(source.as_bytes(), &mut arena);

  let (typing, valtypes) = lilac::typecheck::typecheck(&code);

  for (i, (inst, ty)) in zip(code.iter(), typing.iter()).enumerate() {
    match ty {
      lilac::typecheck::Typing::Nil => {
        write!(out, "%{} {}\n", i, inst).unwrap();
      }
      lilac::typecheck::Typing::Val(ty) => {
        write!(out, "%{} {} : Val {:?}\n", i, inst, valtypes[*ty]).unwrap();
      }
      lilac::typecheck::Typing::Var(ty) => {
        write!(out, "%{} {} : Var {:?}\n", i, inst, valtypes[*ty]).unwrap();
      }
    }
  }
}
