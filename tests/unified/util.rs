use std::iter::zip;

pub(crate) fn dump(out: &mut impl std::fmt::Write, source: &str) {
  let mut store = oxcart::Store::new();
  let mut arena = store.arena();

  let module = lilac::compile_pass1::compile(source.as_bytes(), &mut arena);

  let (typemap, solver) = lilac::typecheck::typecheck(&module);

  for (i, (&inst, insttype)) in zip(module.code.iter(), typemap.insts()).enumerate() {
    match insttype {
      lilac::typecheck::InstType::Entry(xs, y) => {
        let xs = xs.iter().map(|x| solver.resolve(*x)).collect::<Box<[_]>>();
        let y = solver.resolve_ret(*y);
        write!(out, "%{} {} : {:?} -> {:?}\n", i, inst, xs, y).unwrap();
      }
      lilac::typecheck::InstType::Label(xs) => {
        let xs = xs.iter().map(|x| solver.resolve(*x)).collect::<Box<[_]>>();
        write!(out, "%{} {} : {:?}\n", i, inst, xs).unwrap();
      }
      lilac::typecheck::InstType::Local(x) => {
        write!(out, "%{} {} : Local {:?}\n", i, inst, solver.resolve(*x)).unwrap();
      }
      lilac::typecheck::InstType::Nil => {
        write!(out, "%{} {}\n", i, inst).unwrap();
      }
      lilac::typecheck::InstType::Value(x) => {
        write!(out, "%{} {} : Value {:?}\n", i, inst, solver.resolve(*x)).unwrap();
      }
    }
  }
}
