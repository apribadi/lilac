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
}

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

fn compile_values_from_value(t: &mut Env, o: &mut Code, x: u32, k: usize) {
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

fn compile_patch_point(t: &mut Env, o: &mut Code) {
  t.put_point(o.put(Inst::Jump(u32::MAX)));
}

fn resolve_patch_point(t: &mut Env, o: &mut Code, a: u32) {
  o.set(t.pop_point(), Inst::Jump(a));
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
      compile_patch_point(t, o);
      compile_patch_point(t, o);
      let a = o.put(Inst::Label);
      let z = o.put(Inst::ConstBool(false));
      let _ = o.put(Inst::Put(z));
      compile_patch_point(t, o);
      let b = o.put(Inst::Label);
      let y = compile_expr(t, o, y); // compile_expr_kont ??
      let _ = o.put(Inst::Put(y));
      compile_patch_point(t, o);
      let c = o.put(Inst::Label);
      resolve_patch_point(t, o, c);
      resolve_patch_point(t, o, c);
      resolve_patch_point(t, o, b);
      resolve_patch_point(t, o, a);
      let x = o.put(Inst::Pop);
      compile_values_from_value(t, o, x, k);
    }
    Expr::Call(&(f, x)) => {
      let n = x.len();
      let f = compile_expr(t, o, f);
      for &x in x {
        compile_expr_values(t, o, x, 1);
      }
      compile_put_seq(t, o, n);
      let _ = o.put(Inst::Call(f));
      compile_patch_point(t, o);
      let a = o.put(Inst::Label);
      resolve_patch_point(t, o, a);
      compile_pop_seq(t, o, k);
    }
    Expr::Field(&(x, s)) => {
      let x = compile_expr(t, o, x);
      let x = o.put(Inst::Field(x, Symbol::from_bytes(s)));
      compile_values_from_value(t, o, x, k);
    }
    Expr::Index(&(x, i)) => {
      let x = compile_expr(t, o, x);
      let i = compile_expr(t, o, i);
      let x = o.put(Inst::Index(x, i));
      compile_values_from_value(t, o, x, k);
    }
    Expr::Int(n) => {
      let x = o.put(Inst::ConstInt(n));
      compile_values_from_value(t, o, x, k);
    }
    Expr::Loop(_) => {
      unimplemented!()
    }
    Expr::Op1(&(op, x)) => {
      let x = compile_expr(t, o, x);
      let x = o.put(Inst::Op1(op, x));
      compile_values_from_value(t, o, x, k);
    }
    Expr::Op2(&(op, x, y)) => {
      let x = compile_expr(t, o, x);
      let y = compile_expr(t, o, y);
      let x = o.put(Inst::Op2(op, x, y));
      compile_values_from_value(t, o, x, k);
    }
    Expr::Or(&(x, y)) => {
      let x = compile_expr(t, o, x);
      let _ = o.put(Inst::Cond(x));
      compile_patch_point(t, o);
      compile_patch_point(t, o);
      let a = o.put(Inst::Label);
      let y = compile_expr(t, o, y); // compile_expr_kont??
      let _ = o.put(Inst::Put(y));
      compile_patch_point(t, o);
      let b = o.put(Inst::Label);
      let z = o.put(Inst::ConstBool(true));
      let _ = o.put(Inst::Put(z));
      compile_patch_point(t, o);
      let c = o.put(Inst::Label);
      resolve_patch_point(t, o, c);
      resolve_patch_point(t, o, c);
      resolve_patch_point(t, o, b);
      resolve_patch_point(t, o, a);
      let x = o.put(Inst::Pop);
      compile_values_from_value(t, o, x, k);
    }
    Expr::Ternary(&(p, x, y)) => {
      let p = compile_expr(t, o, p);
      let _ = o.put(Inst::Cond(p));
      compile_patch_point(t, o);
      compile_patch_point(t, o);
      let a = o.put(Inst::Label);
      compile_expr_values(t, o, x, k); // compile_expr_kont
      compile_put_seq(t, o, k);
      compile_patch_point(t, o);
      let b = o.put(Inst::Label);
      compile_expr_values(t, o, y, k); // compile_expr_kont
      compile_put_seq(t, o, k);
      compile_patch_point(t, o);
      let c = o.put(Inst::Label);
      resolve_patch_point(t, o, c);
      resolve_patch_point(t, o, c);
      resolve_patch_point(t, o, b);
      resolve_patch_point(t, o, a);
      compile_pop_seq(t, o, k);
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
      compile_values_from_value(t, o, x, k);
    }
  }
}

fn compile_expr_tail<'a>(t: &mut Env, o: &mut Code, x: Expr<'a>) {
  match x {
    Expr::And(&(x, y)) => {
      let x = compile_expr(t, o, x);
      let _ = o.put(Inst::Cond(x));
      compile_patch_point(t, o);
      compile_patch_point(t, o);
      let a = o.put(Inst::Label);
      let z = o.put(Inst::ConstBool(false));
      let _ = o.put(Inst::Put(z));
      let _ = o.put(Inst::Ret);
      let b = o.put(Inst::Label);
      compile_expr_tail(t, o, y);
      resolve_patch_point(t, o, b);
      resolve_patch_point(t, o, a);
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
      let _ = compile_stmt_tail;
      unimplemented!()
    }
    Expr::Or(&(x, y)) => {
      let x = compile_expr(t, o, x);
      let _ = o.put(Inst::Cond(x));
      compile_patch_point(t, o);
      compile_patch_point(t, o);
      let a = o.put(Inst::Label);
      compile_expr_tail(t, o, y);
      let b = o.put(Inst::Label);
      let z = o.put(Inst::ConstBool(true));
      let _ = o.put(Inst::Put(z));
      let _ = o.put(Inst::Ret);
      resolve_patch_point(t, o, b);
      resolve_patch_point(t, o, a);
    }
    Expr::Ternary(&(p, x, y)) => {
      let p = compile_expr(t, o, p);
      let _ = o.put(Inst::Cond(p));
      compile_patch_point(t, o);
      compile_patch_point(t, o);
      let a = o.put(Inst::Label);
      compile_expr_tail(t, o, x);
      let b = o.put(Inst::Label);
      compile_expr_tail(t, o, y);
      resolve_patch_point(t, o, b);
      resolve_patch_point(t, o, a);
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

fn compile_stmt_tail<'a>(t: &mut Env, o: &mut Code, x: Stmt<'a>) {
  let _ = t;
  let _ = o;
  let _ = x;
}
