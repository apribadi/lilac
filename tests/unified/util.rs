pub(crate) fn dump(out: &mut impl std::fmt::Write, source: &str) {
  let mut store = oxcart::Store::new();
  let mut arena = store.arena();

  let module = lilac::make_ast::parse(source.as_bytes(), &mut arena);
  let module = lilac::make_uir::compile(&module);

  let (environment, solver) = lilac::typecheck::typecheck(&module);

  for f in module.decl.iter() {
    write!(out, "=== fun {} : {} ===\n", f.name, environment[f.name]).unwrap();

    for i in f.pos .. f.pos + f.len {
      let inst = module.code[i];
      match inst {
        | lilac::uir::Inst::GotoStaticError
        | lilac::uir::Inst::Put(..)
        | lilac::uir::Inst::Goto(..)
        | lilac::uir::Inst::Cond(..)
        | lilac::uir::Inst::Ret
        | lilac::uir::Inst::Call(..)
        | lilac::uir::Inst::TailCall(..)
        | lilac::uir::Inst::SetField(..)
        | lilac::uir::Inst::SetIndex(..)
        | lilac::uir::Inst::SetLocal(..) =>
          write!(out, "%{} {}\n", i, inst).unwrap(),
        | lilac::uir::Inst::Get(..)
        | lilac::uir::Inst::Const(..)
        | lilac::uir::Inst::ConstBool(..)
        | lilac::uir::Inst::ConstInt(..)
        | lilac::uir::Inst::Field(..)
        | lilac::uir::Inst::Index(..)
        | lilac::uir::Inst::GetLocal(..)
        | lilac::uir::Inst::Op1(..)
        | lilac::uir::Inst::Op2(..) => {
          let x = lilac::typevar::TypeVar(i);
          write!(out, "%{} {} : {}\n", i, inst, solver.resolve_value_type(x).unwrap()).unwrap();
        }
        | lilac::uir::Inst::Local(..) => {
          let x = lilac::typevar::TypeVar(i);
          write!(out, "%{} {} : Local {}\n", i, inst, solver.resolve_value_type(x).unwrap()).unwrap();
        }
        | lilac::uir::Inst::Label(_) => {
          let x = solver.resolve_tuple_type(lilac::typevar::TypeVar(i)).unwrap();
          write!(out, "%{} {} : {}\n", i, inst, x).unwrap();
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
