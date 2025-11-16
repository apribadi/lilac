use std::iter::zip;

pub(crate) fn dump(out: &mut impl std::fmt::Write, source: &str) {
  let mut store = oxcart::Store::new();
  let mut arena = store.arena();

  let code = lilac::compile_pass1::compile(source.as_bytes(), &mut arena);

  let (typemap, solver) = lilac::typecheck::typecheck(&code);

  for (i, (&inst, insttype)) in zip(code.iter(), typemap.insts()).enumerate() {
    match insttype {
      lilac::typecheck::InstType::Nil => {
        write!(out, "%{} {}\n", i, inst).unwrap();
      }
      lilac::typecheck::InstType::Value(x) => {
        write!(out, "%{} {} : Value {:?}\n", i, inst, solver.valtype(x)).unwrap();
      }
      lilac::typecheck::InstType::Local(x) => {
        write!(out, "%{} {} : Local {:?}\n", i, inst, solver.valtype(x)).unwrap();
      }
    }
  }
}
