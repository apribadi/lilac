pub(crate) fn dump(out: &mut impl std::fmt::Write, source: &str) {
  let mut store = oxcart::Store::new();
  let mut arena = store.arena();

  let module = lilac::make_ast::parse(source.as_bytes(), &mut arena);
  let module = lilac::make_hir::compile(&module);

  let (environment, inst_types, solver) = lilac::typecheck::typecheck(&module);

  for f in module.decl.iter() {
    write!(out, "=== fun {} : {:?} ===\n", f.name, environment[f.name]).unwrap();

    for i in f.pos .. f.pos + f.len {
      let inst = module.code[i];
      match inst_types[i] {
        lilac::typecheck::InstType::Label(ref xs) => {
          let xs = xs.iter().map(|x| solver.resolve(*x)).collect::<Box<[_]>>();
          write!(out, "%{} {} : {:?}\n", i, inst, xs).unwrap();
        }
        lilac::typecheck::InstType::Local(x) => {
          write!(out, "%{} {} : Local {:?}\n", i, inst, solver.resolve(x)).unwrap();
        }
        lilac::typecheck::InstType::Nil => {
          write!(out, "%{} {}\n", i, inst).unwrap();
        }
        lilac::typecheck::InstType::Value(x) => {
          write!(out, "%{} {} : Value {:?}\n", i, inst, solver.resolve(x)).unwrap();
        }
      }
    }
  }
}

pub(crate) fn parse_sexp(out: &mut impl std::fmt::Write, source: &str) {
  for sexp in lilac::parse::parse_sexp(source.as_bytes()).iter() {
    write!(out, "{}", sexp).unwrap();
  }
}
