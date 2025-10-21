use crate::ast::Expr;
use crate::ast::Stmt;
use crate::symbol::Symbol;
use crate::uir::Inst;

struct Env {
  values: Vec<u32>,
  points: Vec<u32>,
  // breaks: Vec<usize>,
}

impl Env {
  fn new() -> Self {
    Self {
      values: Vec::new(),
      points: Vec::new(),
      // breaks: Vec::new(),
    }
  }
}

fn put_value(x: u32, e: &mut Env) {
  e.values.push(x);
}

fn pop_value(e: &mut Env) -> u32 {
  return e.values.pop().unwrap();
}

fn pop_values(n: usize, e: &mut Env) -> impl Iterator<Item = u32> {
  return e.values.drain(e.values.len() - n ..);
}

fn put_point(x: u32, e: &mut Env) {
  e.points.push(x);
}

#[allow(dead_code)]
fn pop_point(e: &mut Env) -> u32 {
  return e.points.pop().unwrap();
}

fn pop_points(n: usize, e: &mut Env) -> impl Iterator<Item = u32> {
  return e.points.drain(e.points.len() - n ..);
}

/*
fn put_break_scope(e: &mut Env) {
  e.breaks.push(0);
}

fn pop_break_scope(e: &mut Env) -> impl Iterator<Item = u32> {
  let n = e.breaks.pop().unwrap();
  return e.points.drain(e.points.len() - n ..);
}

fn put_break(a: u32, e: &mut Env) {
  e.breaks.last_mut().unwrap() += 1;
  e.points.push(a);
}
*/

struct Out(Vec<Inst>);

impl Out {
  fn new() -> Self {
    Self(Vec::new())
  }

  fn emit(&mut self, inst: Inst) -> u32 {
    let n = self.0.len();
    self.0.push(inst);
    return n as u32;
  }

  fn emit_patch_point(&mut self) -> u32 {
    return self.emit(Inst::Jump(u32::MAX));
  }

  fn edit(&mut self, index: u32, inst: Inst) {
    self.0[index as usize] = inst;
  }

  fn edit_patch_point(&mut self, index: u32, label: u32) {
    self.edit(index, Inst::Jump(label));
  }
}

pub fn compile<'a>(x: Expr<'a>) -> Vec<Inst> {
  let mut e = Env::new();
  let mut o = Out::new();

  compile_expr_tail(x, &mut e, &mut o);
  return o.0;
}

enum What {
  NumPoints(usize),
  NumValues(usize),
}

impl What {
  fn into_nil(self, e: &mut Env, o: &mut Out) {
    match self {
      What::NumPoints(n) => {
        let a = o.emit(Inst::Label);
        for k in pop_points(n, e) {
          o.edit_patch_point(k, a);
        }
      }
      What::NumValues(0) => {
      }
      _ => {
        // arity error
        unimplemented!()
      }
    }
  }

  fn into_value(self, e: &mut Env, o: &mut Out) -> u32 {
    match self {
      What::NumPoints(n) => {
        let a = o.emit(Inst::Label);
        let x = o.emit(Inst::Pop);
        for k in pop_points(n, e) {
          o.edit_patch_point(k, a);
        }
        return x;
      }
      What::NumValues(1) => {
        return pop_value(e);
      }
      _ => {
        // arity error
        unimplemented!()
      }
    }
  }

  fn into_points(self, e: &mut Env, o: &mut Out) -> usize {
    match self {
      What::NumPoints(n) => {
        return n;
      }
      What::NumValues(n) => {
        for x in pop_values(n, e) {
          let _ = o.emit(Inst::Put(x));
        }
        put_point(o.emit_patch_point(), e);
        return 1;
      }
    }
  }
}

fn compile_expr<'a>(x: Expr<'a>, e: &mut Env, o: &mut Out) -> What {
  match x {
    Expr::And(&(x, y)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let _ = o.emit(Inst::Cond(x));
      let i = o.emit_patch_point();
      let j = o.emit_patch_point();
      let a = o.emit(Inst::Label);
      let x = o.emit(Inst::ConstBool(false));
      let _ = o.emit(Inst::Put(x));
      put_point(o.emit_patch_point(), e);
      let b = o.emit(Inst::Label);
      let n = compile_expr(y, e, o).into_points(e, o);
      o.edit_patch_point(i, a);
      o.edit_patch_point(j, b);
      return What::NumPoints(1 + n);
    }
    Expr::Bool(x) => {
      put_value(o.emit(Inst::ConstBool(x)), e);
      return What::NumValues(1);
    }
    Expr::Call(&(f, x)) => {
      let n = x.len();
      let f = compile_expr(f, e, o).into_value(e, o);
      for &y in x.iter() {
        put_value(compile_expr(y, e, o).into_value(e, o), e);
      }
      for y in pop_values(n, e) {
        let _ = o.emit(Inst::Put(y));
      }
      let _ = o.emit(Inst::Call(f));
      put_point(o.emit_patch_point(), e);
      return What::NumPoints(1);
    }
    Expr::Field(&(x, s)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      put_value(o.emit(Inst::Field(x, Symbol::from_bytes(s))), e);
      return What::NumValues(1);
    }
    Expr::Index(&(x, y)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let y = compile_expr(y, e, o).into_value(e, o);
      put_value(o.emit(Inst::Index(x, y)), e);
      return What::NumValues(1);
    }
    Expr::Int(n) => {
      put_value(o.emit(Inst::ConstInt(n)), e);
      return What::NumValues(1);
    }
    Expr::Loop(x) => {
      // TODO:
      //
      // while compiling break statements, we need to push their patch point to
      // the stack and increment n_breaks
      //
      // the break target needs to be scoped lexically, and in particular be
      // available in sub-trees
      //
      // try to just handle returns first

      let mut n_breaks = 0;
      let a = o.emit(Inst::Label);
      for &y in x.iter() { compile_stmt(y, e, o).into_nil(e, o); }
      let _ = o.emit(Inst::Jump(a));

      return What::NumPoints(n_breaks);
    }
    Expr::Op1(&(f, x)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      put_value(o.emit(Inst::Op1(f, x)), e);
      return What::NumValues(1);
    }
    Expr::Op2(&(f, x, y)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let y = compile_expr(y, e, o).into_value(e, o);
      put_value(o.emit(Inst::Op2(f, x, y)), e);
      return What::NumValues(1);
    }
    Expr::Or(&(x, y)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let _ = o.emit(Inst::Cond(x));
      let i = o.emit_patch_point();
      let j = o.emit_patch_point();
      let a = o.emit(Inst::Label);
      let n = compile_expr(y, e, o).into_points(e, o);
      let b = o.emit(Inst::Label);
      let x = o.emit(Inst::ConstBool(true));
      let _ = o.emit(Inst::Put(x));
      put_point(o.emit_patch_point(), e);
      o.edit_patch_point(i, a);
      o.edit_patch_point(j, b);
      return What::NumPoints(n + 1);
    }
    Expr::Ternary(&(p, x, y)) => {
      let p = compile_expr(p, e, o).into_value(e, o);
      let _ = o.emit(Inst::Cond(p));
      let i = o.emit_patch_point();
      let j = o.emit_patch_point();
      let a = o.emit(Inst::Label);
      let m = compile_expr(y, e, o).into_points(e, o);
      let b = o.emit(Inst::Label);
      let n = compile_expr(x, e, o).into_points(e, o);
      o.edit_patch_point(i, a);
      o.edit_patch_point(j, b);
      return What::NumPoints(m + n);
    }
    Expr::Undefined => {
      put_value(o.emit(Inst::Undefined), e);
      return What::NumValues(1);
    }
    Expr::Variable(s) => {
      // TODO: look in symbol table for local variables
      put_value(o.emit(Inst::Global(Symbol::from_bytes(s))), e);
      return What::NumValues(1);
    }
  }
}

fn compile_expr_tail<'a>(x: Expr<'a>, e: &mut Env, o: &mut Out) {
  match x {
    Expr::And(&(x, y)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let _ = o.emit(Inst::Cond(x));
      let i = o.emit_patch_point();
      let j = o.emit_patch_point();
      let a = o.emit(Inst::Label);
      let x = o.emit(Inst::ConstBool(false));
      let _ = o.emit(Inst::Put(x));
      let _ = o.emit(Inst::Ret);
      let b = o.emit(Inst::Label);
      compile_expr_tail(y, e, o);
      o.edit_patch_point(i, a);
      o.edit_patch_point(j, b);
    }
    Expr::Call(&(f, x)) => {
      let n = x.len();
      let f = compile_expr(f, e, o).into_value(e, o);
      for &y in x.iter() {
        put_value(compile_expr(y, e, o).into_value(e, o), e);
      }
      for y in pop_values(n, e) {
        let _ = o.emit(Inst::Put(y));
      }
      let _ = o.emit(Inst::TailCall(f));
    }
    Expr::Loop(_) => {
      unimplemented!()
    }
    Expr::Or(&(x, y)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let _ = o.emit(Inst::Cond(x));
      let i = o.emit_patch_point();
      let j = o.emit_patch_point();
      let a = o.emit(Inst::Label);
      compile_expr_tail(y, e, o);
      let b = o.emit(Inst::Label);
      let x = o.emit(Inst::ConstBool(true));
      let _ = o.emit(Inst::Put(x));
      let _ = o.emit(Inst::Ret);
      o.edit_patch_point(i, a);
      o.edit_patch_point(j, b);
    }
    Expr::Ternary(&(p, x, y)) => {
      let p = compile_expr(p, e, o).into_value(e, o);
      let _ = o.emit(Inst::Cond(p));
      let i = o.emit_patch_point();
      let j = o.emit_patch_point();
      let a = o.emit(Inst::Label);
      compile_expr_tail(y, e, o);
      let b = o.emit(Inst::Label);
      compile_expr_tail(x, e, o);
      o.edit_patch_point(i, a);
      o.edit_patch_point(j, b);
    }
    x @ (
      | Expr::Bool(_)
      | Expr::Field(_)
      | Expr::Index(_)
      | Expr::Int(_)
      | Expr::Op1(_)
      | Expr::Op2(_)
      | Expr::Undefined
      | Expr::Variable(_)
    ) => {
      match compile_expr(x, e, o) {
        What::NumValues(1) => {
          let _ = o.emit(Inst::Put(pop_value(e)));
          let _ = o.emit(Inst::Ret);
        }
        _ => unreachable!()
      }
    }
  }
}

fn compile_stmt<'a>(x: Stmt<'a>, e: &mut Env, o: &mut Out) -> What {
  let _ = e;
  let _ = o;
  match x {
    Stmt::Expr(x) => {
      return compile_expr(x, e, o);
    }
    _ => unimplemented!(),
  }
}
