use crate::ast::Expr;
use crate::ast::Stmt;
use crate::symbol::Symbol;
use crate::uir::Inst;

struct Code(Vec<Inst>);

impl Code {
  fn new() -> Self {
    Self(Vec::new())
  }

  fn put(&mut self, inst: Inst) -> u32 {
    let n = self.0.len();
    self.0.push(inst);
    return n as u32;
  }

  fn set(&mut self, index: u32, inst: Inst) {
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


// compile_expr
//
// return either (N_VALUES usize | N_POINTS label)

pub fn compile<'a>(x: Expr<'a>) -> Vec<Inst> {
  let mut t = Env::new();
  let mut o = Code::new();

  compile_expr_tail(&mut t, &mut o, x);
  return o.0;
}

fn compile_put_seq(t: &mut Env, o: &mut Code, n: usize) {
  for x in t.pop_value_multi(n) {
    let _ = o.put(Inst::Put(x));
  }
}

fn compile_pop_seq(t: &mut Env, o: &mut Code, n: usize) {
  for _ in 0 .. n {
    t.put_value(o.put(Inst::Pop));
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
      let x = o.put(Inst::Undefined);
      for _ in 0 .. k {
        t.put_value(x);
      }
    }
  }
}

fn patch_point(t: &mut Env, o: &mut Code) {
  t.put_point(o.put(Inst::Jump(u32::MAX)));
}

fn update_patch_points<const N: usize>(t: &mut Env, o: &mut Code, labels: [u32; N]) {
  for (i, k) in t.pop_point_multi(N).enumerate() {
    o.set(k, Inst::Jump(labels[i]));
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
      let _ = o.put(Inst::Cond(x));
      patch_point(t, o);
      patch_point(t, o);
      let a = o.put(Inst::Label);
      let x = o.put(Inst::ConstBool(false));
      let _ = o.put(Inst::Put(x));
      patch_point(t, o);
      let b = o.put(Inst::Label);
      compile_expr_values_kont(t, o, y, 1);
      let c = o.put(Inst::Label);
      let x = o.put(Inst::Pop);
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
      let _ = o.put(Inst::Call(f));
      patch_point(t, o);
      let a = o.put(Inst::Label);
      compile_pop_seq(t, o, k);
      update_patch_points(t, o, [a]);
    }
    Expr::Field(&(x, s)) => {
      let x = compile_expr(t, o, x);
      let x = o.put(Inst::Field(x, Symbol::from_bytes(s)));
      into_values(t, o, x, k);
    }
    Expr::Index(&(x, i)) => {
      let x = compile_expr(t, o, x);
      let i = compile_expr(t, o, i);
      let x = o.put(Inst::Index(x, i));
      into_values(t, o, x, k);
    }
    Expr::Int(n) => {
      let x = o.put(Inst::ConstInt(n));
      into_values(t, o, x, k);
    }
    Expr::Loop(_) => {
      unimplemented!()
    }
    Expr::Op1(&(op, x)) => {
      let x = compile_expr(t, o, x);
      let x = o.put(Inst::Op1(op, x));
      into_values(t, o, x, k);
    }
    Expr::Op2(&(op, x, y)) => {
      let x = compile_expr(t, o, x);
      let y = compile_expr(t, o, y);
      let x = o.put(Inst::Op2(op, x, y));
      into_values(t, o, x, k);
    }
    Expr::Or(&(x, y)) => {
      let x = compile_expr(t, o, x);
      let _ = o.put(Inst::Cond(x));
      patch_point(t, o);
      patch_point(t, o);
      let a = o.put(Inst::Label);
      compile_expr_values_kont(t, o, y, 1);
      let b = o.put(Inst::Label);
      let x = o.put(Inst::ConstBool(true));
      let _ = o.put(Inst::Put(x));
      patch_point(t, o);
      let c = o.put(Inst::Label);
      let x = o.put(Inst::Pop);
      into_values(t, o, x, k);
      update_patch_points(t, o, [a, b, c, c]);
    }
    Expr::Ternary(&(p, x, y)) => {
      let p = compile_expr(t, o, p);
      let _ = o.put(Inst::Cond(p));
      patch_point(t, o);
      patch_point(t, o);
      let a = o.put(Inst::Label);
      compile_expr_values_kont(t, o, y, k);
      let b = o.put(Inst::Label);
      compile_expr_values_kont(t, o, x, k);
      let c = o.put(Inst::Label);
      compile_pop_seq(t, o, k);
      update_patch_points(t, o, [a, b, c, c]);
    }
    Expr::Undefined => {
      let x = o.put(Inst::Undefined);
      for _ in 0 .. k {
        t.put_value(x);
      }
    }
    Expr::Variable(symbol) => {
      // TODO: local scope
      let x = o.put(Inst::Global(Symbol::from_bytes(symbol)));
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
      let _ = o.put(Inst::Call(f));
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
      let _ = o.put(Inst::Cond(x));
      patch_point(t, o);
      patch_point(t, o);
      let a = o.put(Inst::Label);
      let x = o.put(Inst::ConstBool(false));
      let _ = o.put(Inst::Put(x));
      let _ = o.put(Inst::Ret);
      let b = o.put(Inst::Label);
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
      let _ = o.put(Inst::TailCall(f));
    }
    Expr::Loop(_) => {
      unimplemented!()
    }
    Expr::Or(&(x, y)) => {
      let x = compile_expr(t, o, x);
      let _ = o.put(Inst::Cond(x));
      patch_point(t, o);
      patch_point(t, o);
      let a = o.put(Inst::Label);
      compile_expr_tail(t, o, y);
      let b = o.put(Inst::Label);
      let x = o.put(Inst::ConstBool(true));
      let _ = o.put(Inst::Put(x));
      let _ = o.put(Inst::Ret);
      update_patch_points(t, o, [a, b]);
    }
    Expr::Ternary(&(p, x, y)) => {
      let p = compile_expr(t, o, p);
      let _ = o.put(Inst::Cond(p));
      patch_point(t, o);
      patch_point(t, o);
      let a = o.put(Inst::Label);
      compile_expr_tail(t, o, y);
      let b = o.put(Inst::Label);
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
      let _ = o.put(Inst::Put(x));
      let _ = o.put(Inst::Ret);
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
