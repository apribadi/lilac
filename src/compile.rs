use crate::ast::Expr;
use crate::symbol::Symbol;
use crate::uir::Inst;

struct Env {
  values: Vec<u32>,
  points: Vec<u32>,
}

impl Env {
  fn new() -> Self {
    Self {
      values: Vec::new(),
      points: Vec::new(),
    }
  }
}

fn put_value(x: u32, e: &mut Env) {
  e.values.push(x);
}

fn pop_value(e: &mut Env) -> u32 {
  return e.values.pop().unwrap();
}

fn pop_value_multi(n: usize, e: &mut Env) -> impl Iterator<Item = u32> {
  return e.values.drain(e.values.len() - n ..);
}

fn put_point(x: u32, e: &mut Env) {
  e.points.push(x);
}

#[allow(dead_code)]
fn pop_point(e: &mut Env) -> u32 {
  return e.points.pop().unwrap();
}

fn pop_point_multi(n: usize, e: &mut Env) -> impl Iterator<Item = u32> {
  return e.points.drain(e.points.len() - n ..);
}

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

  // let _ = compile_expr(x, &mut e, &mut o).into_value(&mut e, &mut o);
  compile_expr_tail(x, &mut e, &mut o);
  return o.0;
}

// compile_expr
//
// return either (N_VALUES usize | N_POINTS label)

enum What {
  NumPoints(usize),
  NumValues(usize),
}

impl What {
  fn into_value(self, e: &mut Env, o: &mut Out) -> u32 {
    match self {
      What::NumPoints(n) => {
        let a = o.emit(Inst::Label);
        let x = o.emit(Inst::Pop);
        for k in pop_point_multi(n, e) {
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
        for x in pop_value_multi(n, e) {
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
    Expr::Call(&(f, x)) => {
      let n = x.len();
      let f = compile_expr(f, e, o).into_value(e, o);
      for y in x {
        put_value(compile_expr(*y, e, o).into_value(e, o), e);
      }
      for y in pop_value_multi(n, e) {
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
    Expr::Loop(_) => {
      unimplemented!()
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
      for y in x {
        put_value(compile_expr(*y, e, o).into_value(e, o), e);
      }
      for y in pop_value_multi(n, e) {
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
