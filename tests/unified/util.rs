pub(crate) fn dump(out: &mut impl std::fmt::Write, source: &str) {
  let mut store = oxcart::Store::new();
  let mut arena = store.arena();

  let module = lilac::make_ast::parse(source.as_bytes(), &mut arena);
  let module = lilac::make_hir::compile(&module);

  let (environment, solver) = lilac::typecheck::typecheck(&module);

  for f in module.decl.iter() {
    write!(out, "=== fun {} : {} ===\n", f.name, environment[f.name]).unwrap();

    for i in f.pos .. f.pos + f.len {
      let inst = module.code[i];
      match inst {
        | lilac::hir::Inst::GotoStaticError
        | lilac::hir::Inst::Put(..)
        | lilac::hir::Inst::Goto(..)
        | lilac::hir::Inst::Cond(..)
        | lilac::hir::Inst::Ret
        | lilac::hir::Inst::Call(..)
        | lilac::hir::Inst::TailCall(..)
        | lilac::hir::Inst::SetField(..)
        | lilac::hir::Inst::SetIndex(..)
        | lilac::hir::Inst::SetLocal(..) =>
          write!(out, "%{} {}\n", i, inst).unwrap(),
        | lilac::hir::Inst::Get(..)
        | lilac::hir::Inst::Const(..)
        | lilac::hir::Inst::ConstBool(..)
        | lilac::hir::Inst::ConstInt(..)
        | lilac::hir::Inst::Field(..)
        | lilac::hir::Inst::Index(..)
        | lilac::hir::Inst::GetLocal(..)
        | lilac::hir::Inst::Op1(..)
        | lilac::hir::Inst::Op2(..) => {
          let x = lilac::typevar::TypeVar(i);
          write!(out, "%{} {} : Value {}\n", i, inst, solver.resolve_value_type(x)).unwrap();
        }
        | lilac::hir::Inst::Local(..) => {
          let x = lilac::typevar::TypeVar(i);
          write!(out, "%{} {} : Local {}\n", i, inst, solver.resolve_value_type(x)).unwrap();
        }
        | lilac::hir::Inst::Label(_) => {
          let x = solver.resolve_tuple_type(lilac::typevar::TypeVar(i));
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
