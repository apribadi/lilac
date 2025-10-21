use crate::ast::Expr;
use crate::ast::Stmt;
use crate::symbol::Symbol;
use crate::uir::Inst;

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

  fn edit(&mut self, index: u32, inst: Inst) {
    self.0[index as usize] = inst;
  }
}

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

  fn put_value(&mut self, x: u32) {
    self.values.push(x)
  }

  fn pop_value(&mut self) -> u32 {
    return self.values.pop().unwrap();
  }

  fn pop_value_multi(&mut self, arity: usize) -> impl Iterator<Item = u32> {
    return self.values.drain(self.values.len() - arity ..);
  }

  fn put_point(&mut self, x: u32) {
    self.points.push(x)
  }

  fn pop_point(&mut self) -> u32 {
    return self.points.pop().unwrap();
  }

  fn pop_point_multi(&mut self, arity: usize) -> impl Iterator<Item = u32> {
    return self.points.drain(self.points.len() - arity ..);
  }
}

fn put_value(x: u32, e: &mut Env) {
  e.put_value(x);
}

fn pop_value(e: &mut Env) -> u32 {
  return e.pop_value();
}

fn pop_value_multi(n: usize, e: &mut Env) -> impl Iterator<Item = u32> {
  return e.pop_value_multi(n);
}

fn put_point(x: u32, e: &mut Env) {
  e.put_point(x);
}

fn pop_point(e: &mut Env) -> u32 {
  return e.pop_point();
}

fn pop_point_multi(n: usize, e: &mut Env) -> impl Iterator<Item = u32> {
  return e.pop_point_multi(n);
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
          o.edit(k, Inst::Jump(a));
        }
        return x;
      }
      What::NumValues(1) => {
        return pop_value(e);
      }
      _ => {
        // error
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
        put_point(patch_point(o), e);
        return 1;
      }
    }
  }
}

fn patch_point(o: &mut Out) -> u32 {
  return o.emit(Inst::Jump(u32::MAX));
}

fn edit_patch_point(i: u32, a: u32, o: &mut Out) {
  return o.edit(i, Inst::Jump(a));
}

fn compile_expr<'a>(x: Expr<'a>, e: &mut Env, o: &mut Out) -> What {
  match x {
    Expr::And(&(x, y)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let _ = o.emit(Inst::Cond(x));
      let i = patch_point(o);
      let j = patch_point(o);
      let a = o.emit(Inst::Label);
      let x = o.emit(Inst::ConstBool(false));
      let _ = o.emit(Inst::Put(x));
      put_point(patch_point(o), e);
      let b = o.emit(Inst::Label);
      let n = compile_expr(y, e, o).into_points(e, o);
      edit_patch_point(i, a, o);
      edit_patch_point(j, b, o);
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
      put_point(patch_point(o), e);
      return What::NumPoints(1);
    }
    Expr::Int(n) => {
      put_value(o.emit(Inst::ConstInt(n)), e);
      return What::NumValues(1);
    }
    Expr::Op2(&(f, x, y)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let y = compile_expr(y, e, o).into_value(e, o);
      put_value(o.emit(Inst::Op2(f, x, y)), e);
      return What::NumValues(1);
    }
    Expr::Ternary(&(p, x, y)) => {
      let p = compile_expr(p, e, o).into_value(e, o);
      let _ = o.emit(Inst::Cond(p));
      let i = patch_point(o);
      let j = patch_point(o);
      let a = o.emit(Inst::Label);
      let m = compile_expr(y, e, o).into_points(e, o);
      let b = o.emit(Inst::Label);
      let n = compile_expr(x, e, o).into_points(e, o);
      edit_patch_point(i, a, o);
      edit_patch_point(j, b, o);
      return What::NumPoints(m + n);
    }
    Expr::Variable(s) => {
      // TODO: look in symbol table for local variables
      put_value(o.emit(Inst::Global(Symbol::from_bytes(s))), e);
      return What::NumValues(1);
    }
    _ => unimplemented!(),
  }
}

fn compile_expr_tail<'a>(x: Expr<'a>, e: &mut Env, o: &mut Out) {
  match x {
    Expr::And(&(x, y)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let _ = o.emit(Inst::Cond(x));
      let i = patch_point(o);
      let j = patch_point(o);
      let a = o.emit(Inst::Label);
      let x = o.emit(Inst::ConstBool(false));
      let _ = o.emit(Inst::Put(x));
      let _ = o.emit(Inst::Ret);
      let b = o.emit(Inst::Label);
      compile_expr_tail(y, e, o);
      edit_patch_point(i, a, o);
      edit_patch_point(j, b, o);
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
    Expr::Ternary(&(p, x, y)) => {
      let p = compile_expr(p, e, o).into_value(e, o);
      let _ = o.emit(Inst::Cond(p));
      let i = patch_point(o);
      let j = patch_point(o);
      let a = o.emit(Inst::Label);
      compile_expr_tail(y, e, o);
      let b = o.emit(Inst::Label);
      compile_expr_tail(x, e, o);
      edit_patch_point(i, a, o);
      edit_patch_point(j, b, o);
    }
    x => {
      match compile_expr(x, e, o) {
        What::NumPoints(_) => {
          unreachable!()
        }
        What::NumValues(n) => {
          for x in pop_value_multi(n, e) {
            let _ = o.emit(Inst::Put(x));
          }
          let _ = o.emit(Inst::Ret);
        }
      }
    }
  }
}

/*
fn compile_put_seq(t: &mut Env, o: &mut Code, n: usize) {
  for x in t.pop_value_multi(n) {
    let _ = o.emit(Inst::Put(x));
  }
}

fn compile_pop_seq(t: &mut Env, o: &mut Code, n: usize) {
  for _ in 0 .. n {
    t.put_value(o.emit(Inst::Pop));
  }
}

fn into_values(t: &mut Env, o: &mut Code, x: u32, k: usize) {
  match k {
    0 => {
      // TODO: error dropped value
    }
    1 => {
      t.put_value(x);
    }
    _ => {
      // TODO: error arity mismatch
      let x = o.emit(Inst::Undefined);
      for _ in 0 .. k {
        t.put_value(x);
      }
    }
  }
}

fn patch_point(t: &mut Env, o: &mut Code) {
  t.put_point(o.emit(Inst::Jump(u32::MAX)));
}

fn update_patch_points<const N: usize>(t: &mut Env, o: &mut Code, labels: [u32; N]) {
  for (i, k) in t.pop_point_multi(N).enumerate() {
    o.edit(k, Inst::Jump(labels[i]));
  }
}

fn compile_expr<'a>(t: &mut Env, o: &mut Code, x: Expr<'a>) -> u32 {
  compile_expr_values(t, o, x, 1);
  return t.pop_value();
}

fn compile_expr_values<'a>(t: &mut Env, o: &mut Code, x: Expr<'a>, k: usize) {
  match x {
    Expr::And(&(x, y)) => {
      let x = compile_expr(t, o, x);
      let _ = o.emit(Inst::Cond(x));
      patch_point(t, o);
      patch_point(t, o);
      let a = o.emit(Inst::Label);
      let x = o.emit(Inst::ConstBool(false));
      let _ = o.emit(Inst::Put(x));
      patch_point(t, o);
      let b = o.emit(Inst::Label);
      compile_expr_values_kont(t, o, y, 1);
      let c = o.emit(Inst::Label);
      let x = o.emit(Inst::Pop);
      into_values(t, o, x, k);
      update_patch_points(t, o, [a, b, c, c]);
    }
    Expr::Call(&(f, x)) => {
      let n = x.len();
      let f = compile_expr(t, o, f);
      for &x in x {
        compile_expr_values(t, o, x, 1);
      }
      compile_put_seq(t, o, n);
      let _ = o.emit(Inst::Call(f));
      patch_point(t, o);
      let a = o.emit(Inst::Label);
      compile_pop_seq(t, o, k);
      update_patch_points(t, o, [a]);
    }
    Expr::Field(&(x, s)) => {
      let x = compile_expr(t, o, x);
      let x = o.emit(Inst::Field(x, Symbol::from_bytes(s)));
      into_values(t, o, x, k);
    }
    Expr::Index(&(x, i)) => {
      let x = compile_expr(t, o, x);
      let i = compile_expr(t, o, i);
      let x = o.emit(Inst::Index(x, i));
      into_values(t, o, x, k);
    }
    Expr::Int(n) => {
      let x = o.emit(Inst::ConstInt(n));
      into_values(t, o, x, k);
    }
    Expr::Loop(_) => {
      unimplemented!()
    }
    Expr::Op1(&(op, x)) => {
      let x = compile_expr(t, o, x);
      let x = o.emit(Inst::Op1(op, x));
      into_values(t, o, x, k);
    }
    Expr::Op2(&(op, x, y)) => {
      let x = compile_expr(t, o, x);
      let y = compile_expr(t, o, y);
      let x = o.emit(Inst::Op2(op, x, y));
      into_values(t, o, x, k);
    }
    Expr::Or(&(x, y)) => {
      let x = compile_expr(t, o, x);
      let _ = o.emit(Inst::Cond(x));
      patch_point(t, o);
      patch_point(t, o);
      let a = o.emit(Inst::Label);
      compile_expr_values_kont(t, o, y, 1);
      let b = o.emit(Inst::Label);
      let x = o.emit(Inst::ConstBool(true));
      let _ = o.emit(Inst::Put(x));
      patch_point(t, o);
      let c = o.emit(Inst::Label);
      let x = o.emit(Inst::Pop);
      into_values(t, o, x, k);
      update_patch_points(t, o, [a, b, c, c]);
    }
    Expr::Ternary(&(p, x, y)) => {
      let p = compile_expr(t, o, p);
      let _ = o.emit(Inst::Cond(p));
      patch_point(t, o);
      patch_point(t, o);
      let a = o.emit(Inst::Label);
      compile_expr_values_kont(t, o, y, k);
      let b = o.emit(Inst::Label);
      compile_expr_values_kont(t, o, x, k);
      let c = o.emit(Inst::Label);
      compile_pop_seq(t, o, k);
      update_patch_points(t, o, [a, b, c, c]);
    }
    Expr::Undefined => {
      let x = o.emit(Inst::Undefined);
      for _ in 0 .. k {
        t.put_value(x);
      }
    }
    Expr::Variable(symbol) => {
      // TODO: local scope
      let x = o.emit(Inst::Global(Symbol::from_bytes(symbol)));
      into_values(t, o, x, k);
    }
  }
}

fn compile_expr_values_kont<'a>(t: &mut Env, o: &mut Code, x: Expr<'a>, k: usize) {
  match x {
    Expr::Call(&(f, x)) => {
      let n = x.len();
      let f = compile_expr(t, o, f);
      for &x in x {
        compile_expr_values(t, o, x, 1);
      }
      compile_put_seq(t, o, n);
      let _ = o.emit(Inst::Call(f));
      patch_point(t, o);
    }
    x @ (
      | Expr::And(_)
      | Expr::Field(_)
      | Expr::Index(_)
      | Expr::Int(_)
      | Expr::Loop(_)
      | Expr::Op1(_)
      | Expr::Op2(_)
      | Expr::Or(_)
      | Expr::Ternary(_)
      | Expr::Undefined
      | Expr::Variable(_)
    ) => {
      compile_expr_values(t, o, x, k);
      compile_put_seq(t, o, k);
      patch_point(t, o);
    }
  }
}

fn compile_expr_tail<'a>(t: &mut Env, o: &mut Code, x: Expr<'a>) {
  match x {
    Expr::And(&(x, y)) => {
      let x = compile_expr(t, o, x);
      let _ = o.emit(Inst::Cond(x));
      patch_point(t, o);
      patch_point(t, o);
      let a = o.emit(Inst::Label);
      let x = o.emit(Inst::ConstBool(false));
      let _ = o.emit(Inst::Put(x));
      let _ = o.emit(Inst::Ret);
      let b = o.emit(Inst::Label);
      compile_expr_tail(t, o, y);
      update_patch_points(t, o, [a, b]);
    }
    Expr::Call(&(f, x)) => {
      let n = x.len();
      let f = compile_expr(t, o, f);
      for &x in x {
        compile_expr_values(t, o, x, 1);
      }
      compile_put_seq(t, o, n);
      let _ = o.emit(Inst::TailCall(f));
    }
    Expr::Loop(_) => {
      unimplemented!()
    }
    Expr::Or(&(x, y)) => {
      let x = compile_expr(t, o, x);
      let _ = o.emit(Inst::Cond(x));
      patch_point(t, o);
      patch_point(t, o);
      let a = o.emit(Inst::Label);
      compile_expr_tail(t, o, y);
      let b = o.emit(Inst::Label);
      let x = o.emit(Inst::ConstBool(true));
      let _ = o.emit(Inst::Put(x));
      let _ = o.emit(Inst::Ret);
      update_patch_points(t, o, [a, b]);
    }
    Expr::Ternary(&(p, x, y)) => {
      let p = compile_expr(t, o, p);
      let _ = o.emit(Inst::Cond(p));
      patch_point(t, o);
      patch_point(t, o);
      let a = o.emit(Inst::Label);
      compile_expr_tail(t, o, y);
      let b = o.emit(Inst::Label);
      compile_expr_tail(t, o, x);
      update_patch_points(t, o, [a, b]);
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
      let x = compile_expr(t, o, x);
      let _ = o.emit(Inst::Put(x));
      let _ = o.emit(Inst::Ret);
    }
  }
}

/*
fn compile_stmt_tail<'a>(t: &mut Env, o: &mut Code, x: Stmt<'a>) {
  let _ = t;
  let _ = o;
  let _ = x;
}
*/
*/
