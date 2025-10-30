use crate::ast::Expr;
use crate::ast::Item;
use crate::ast::Stmt;
use crate::hir::Inst;
use crate::symbol::Symbol;
use foldhash::HashMap;
use foldhash::HashMapExt;
use std::iter::zip;

// TODO: consider special lowering for arguments to cond

pub fn compile<'a>(item_list: impl Iterator<Item = Item<'a>>) -> Vec<Inst> {
  let mut e = Env::new();
  let mut o = Out::new();

  for Item::Fundef(f) in item_list {
    let _ = o.emit(Inst::Entry(f.args.len() as u32));

    for x in f.args {
      let y = o.emit(Inst::Pop);
      if let Some(x) = x.name {
        put_referent(x, Referent::Let(y), &mut e.scopes);
      }
    }

    compile_block_tail(f.body, &mut e, &mut o);
  }

  return o.0;
}

enum What {
  NumPoints(usize),
  NumValues(usize),
}

enum Referent {
  Let(u32),
  Var(u32),
}

struct Point {
  index: u32,
  arity: Option<u32>,
}

#[derive(Clone, Copy)]
struct Label {
  index: u32,
  arity: u32,
}

struct Env {
  scopes: Scopes,
  loops: Loops,
  values: Vec<u32>,
  points: Vec<Point>,
}

impl Env {
  fn new() -> Self {
    Self {
      scopes: Scopes::new(),
      loops: Loops::new(),
      values: Vec::new(),
      points: Vec::new(),
    }
  }
}

struct Scopes {
  count: Vec<usize>,
  undo: Vec<(Symbol, Option<Referent>)>,
  table: HashMap<Symbol, Referent>,
}

impl Scopes {
  fn new() -> Self {
    Self {
      count: Vec::new(),
      undo: Vec::new(),
      table: HashMap::new()
    }
  }
}

struct Loops {
  labels: Vec<Label>,
  break_counts: Vec<Option<usize>>,
  break_points: Vec<Point>,
}

impl Loops {
  fn new() -> Self {
    Self {
      labels: Vec::new(),
      break_counts: Vec::new(),
      break_points: Vec::new(),
    }
  }
}

fn put<T>(x: T, y: &mut Vec<T>) {
  y.push(x);
}

fn pop<T>(x: &mut Vec<T>) -> T {
  return x.pop().unwrap();
}

fn pop_list<T>(n: usize, x: &mut Vec<T>) -> impl Iterator<Item = T> {
  return x.drain(x.len() - n ..);
}

fn put_referent(s: Symbol, x: Referent, t: &mut Scopes) {
  let y = t.table.insert(s, x);
  if let Some(n) = t.count.last_mut() {
    t.undo.push((s, y));
    *n += 1;
  }
}

fn get_referent(s: Symbol, t: &Scopes) -> Option<&Referent> {
  return t.table.get(&s);
}

fn put_loop(a: Label, t: &mut Loops) {
  t.labels.push(a);
  t.break_counts.push(Some(0));
}

fn pop_loop(t: &mut Loops, points: &mut Vec<Point>) -> usize {
  let _ = t.labels.pop().unwrap();
  let n = t.break_counts.pop().unwrap().unwrap();
  for i in pop_list(n, &mut t.break_points) {
    points.push(i);
  }
  return n;
}

fn put_loop_tail(a: Label, t: &mut Loops) {
  t.labels.push(a);
  t.break_counts.push(None);
}

fn pop_loop_tail(t: &mut Loops) {
  let _ = t.labels.pop().unwrap();
  let _ = t.break_counts.pop().unwrap();
}

fn put_scope(t: &mut Scopes) {
  t.count.push(0);
}

fn pop_scope(t: &mut Scopes) {
  for (s, x) in pop_list(pop(&mut t.count), &mut t.undo) {
    match x {
      None => {
        let _ = t.table.remove(&s);
      }
      Some(x) => {
        let _ = t.table.insert(s, x);
      }
    }
  }
}

fn put_break(i: Point, t: &mut Loops) {
  let n = t.break_counts.last_mut().unwrap().as_mut().unwrap();
  t.break_points.push(i);
  *n += 1;
}

struct Out(Vec<Inst>);

impl Out {
  fn new() -> Self {
    Self(Vec::new())
  }

  fn emit(&mut self, inst: Inst) -> u32 {
    let n = self.0.len();
    assert!(n < u32::MAX as usize);
    self.0.push(inst);
    return n as u32;
  }

  fn emit_point(&mut self, arity: Option<usize>) -> Point {
    let i = self.emit(Inst::Goto(u32::MAX));
    let n = arity.map(|n| n as u32);
    return Point { index: i, arity: n };
  }

  fn emit_label(&mut self, arity: usize, ps: impl IntoIterator<Item = Point>) -> Label {
    let n = arity as u32;
    let a = self.emit(Inst::Label(n));
    let a = Label { index: a, arity: n};
    patch_point_list(a, ps, self);
    return a;
  }
}

fn patch_point_list(a: Label, ps: impl IntoIterator<Item = Point>, o: &mut Out) {
  for i in ps {
    if let Some(n) = i.arity && n != a.arity {
      // error, arity mismatch
      o.0[i.index as usize] = Inst::GotoStaticError;
    } else {
      o.0[i.index as usize] = Inst::Goto(a.index);
    }
  }
}

impl What {
  const NEVER: Self = What::NumPoints(0);

  const NIL: Self = What::NumValues(0);

  fn into_nil(self, e: &mut Env, o: &mut Out) {
    match self {
      What::NumPoints(n_points) => {
        let _ = o.emit_label(0, pop_list(n_points, &mut e.points));
      }
      What::NumValues(n_values) => {
        if n_values != 0 {
          // error, arity mismatch
          let _ = pop_list(n_values, &mut e.values);
          let _ = o.emit(Inst::GotoStaticError);
          let _ = o.emit(Inst::Label(0));
        }
      }
    }
  }

  fn into_value(self, e: &mut Env, o: &mut Out) -> u32 {
    match self {
      What::NumPoints(n_points) => {
        let _ = o.emit_label(1, pop_list(n_points, &mut e.points));
        let x = o.emit(Inst::Pop);
        return x;
      }
      What::NumValues(n_values) => {
        if n_values == 1 {
          return pop(&mut e.values);
        } else {
          // error, arity mismatch
          let _ = pop_list(n_values, &mut e.values);
          let _ = o.emit(Inst::GotoStaticError);
          let _ = o.emit(Inst::Label(1));
          let x = o.emit(Inst::Pop);
          return x;
        }
      }
    }
  }

  fn into_value_list(self, arity: usize, e: &mut Env, o: &mut Out) {
    match self {
      What::NumPoints(n_points) => {
        let _ = o.emit_label(arity, pop_list(n_points, &mut e.points));
        for _ in 0 .. arity {
          let x = o.emit(Inst::Pop);
          put(x, &mut e.values);
        }
      }
      What::NumValues(n_values) => {
        if arity != n_values {
          // error, arity mismatch
          let _ = pop_list(n_values, &mut e.values);
          let _ = o.emit(Inst::GotoStaticError);
          let _ = o.emit(Inst::Label(arity as u32));
          for _ in 0 .. arity {
            let x = o.emit(Inst::Pop);
            put(x, &mut e.values);
          }
        }
      }
    }
  }

  fn into_point_list(self, e: &mut Env, o: &mut Out) -> usize {
    match self {
      What::NumPoints(n_points) => {
        return n_points;
      }
      What::NumValues(n_values) => {
        for x in pop_list(n_values, &mut e.values) {
          let _ = o.emit(Inst::Put(x));
        }
        let p = o.emit_point(Some(n_values));
        put(p, &mut e.points);
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
      let p = o.emit_point(Some(0));
      let q = o.emit_point(Some(0));
      let _ = o.emit_label(0, [p]);
      let x = o.emit(Inst::ConstBool(false));
      let _ = o.emit(Inst::Put(x));
      let r = o.emit_point(Some(1));
      put(r, &mut e.points);
      let _ = o.emit_label(0, [q]);
      let n = compile_expr(y, e, o).into_point_list(e, o);
      return What::NumPoints(1 + n);
    }
    Expr::Bool(x) => {
      let x = o.emit(Inst::ConstBool(x));
      put(x, &mut e.values);
      return What::NumValues(1);
    }
    Expr::Call(&(f, xs)) => {
      let f = compile_expr(f, e, o).into_value(e, o);
      for &x in xs {
        let x = compile_expr(x, e, o).into_value(e, o);
        put(x, &mut e.values);
      }
      for x in pop_list(xs.len(), &mut e.values) {
        let _ = o.emit(Inst::Put(x));
      }
      let _ = o.emit(Inst::Call(f));
      let p = o.emit_point(None);
      put(p, &mut e.points);
      return What::NumPoints(1);
    }
    Expr::Field(&(x, s)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let x = o.emit(Inst::Field(x, s));
      put(x, &mut e.values);
      return What::NumValues(1);
    }
    Expr::If(&(x, ys)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let _ = o.emit(Inst::Cond(x));
      let p = o.emit_point(Some(0));
      let q = o.emit_point(Some(0));
      put(p, &mut e.points);
      let _ = o.emit_label(0, [q]);
      let n = compile_block(ys, e, o).into_point_list(e, o);
      return What::NumPoints(1 + n);
    }
    Expr::IfElse(&(x, ys, zs)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let _ = o.emit(Inst::Cond(x));
      let p = o.emit_point(Some(0));
      let q = o.emit_point(Some(0));
      let _ = o.emit_label(0, [p]);
      let m = compile_block(zs, e, o).into_point_list(e, o);
      let _ = o.emit_label(0, [q]);
      let n = compile_block(ys, e, o).into_point_list(e, o);
      return What::NumPoints(m + n);
    }
    Expr::Index(&(x, y)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let y = compile_expr(y, e, o).into_value(e, o);
      let x = o.emit(Inst::Index(x, y));
      put(x, &mut e.values);
      return What::NumValues(1);
    }
    Expr::Int(n) => {
      let x = o.emit(Inst::ConstInt(n));
      put(x, &mut e.values);
      return What::NumValues(1);
    }
    Expr::Loop(xs) => {
      let p = o.emit_point(Some(0));
      let a = o.emit_label(0, [p]);
      put_loop(a, &mut e.loops);
      let m = compile_block(xs, e, o).into_point_list(e, o);
      patch_point_list(a, pop_list(m, &mut e.points), o);
      let n = pop_loop(&mut e.loops, &mut e.points);
      return What::NumPoints(n);
    }
    Expr::Op1(&(f, x)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let x = o.emit(Inst::Op1(f, x));
      put(x, &mut e.values);
      return What::NumValues(1);
    }
    Expr::Op2(&(f, x, y)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let y = compile_expr(y, e, o).into_value(e, o);
      let x = o.emit(Inst::Op2(f, x, y));
      put(x, &mut e.values);
      return What::NumValues(1);
    }
    Expr::Or(&(x, y)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let _ = o.emit(Inst::Cond(x));
      let p = o.emit_point(Some(0));
      let q = o.emit_point(Some(0));
      let _ = o.emit_label(0, [p]);
      let n = compile_expr(y, e, o).into_point_list(e, o);
      let _ = o.emit_label(0, [q]);
      let x = o.emit(Inst::ConstBool(true));
      let _ = o.emit(Inst::Put(x));
      let r = o.emit_point(Some(1));
      put(r, &mut e.points);
      return What::NumPoints(n + 1);
    }
    Expr::Ternary(&(x, y, z)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let _ = o.emit(Inst::Cond(x));
      let p = o.emit_point(Some(0));
      let q = o.emit_point(Some(0));
      let _ = o.emit_label(0, [p]);
      let m = compile_expr(z, e, o).into_point_list(e, o);
      let _ = o.emit_label(0, [q]);
      let n = compile_expr(y, e, o).into_point_list(e, o);
      return What::NumPoints(m + n);
    }
    Expr::Undefined => {
      // error, evaluating undefined expression
      let _ = o.emit(Inst::GotoStaticError);
      let _ = o.emit(Inst::Label(1));
      let x = o.emit(Inst::Pop);
      put(x, &mut e.values);
      return What::NumValues(1);
    }
    Expr::Variable(s) => {
      match get_referent(s, &e.scopes) {
        None => {
          let x = o.emit(Inst::Const(s));
          put(x, &mut e.values);
        }
        Some(&Referent::Let(x)) => {
          put(x, &mut e.values);
        }
        Some(&Referent::Var(x)) => {
          let x = o.emit(Inst::Local(x));
          put(x, &mut e.values);
        }
      }
      return What::NumValues(1);
    }
  }
}

fn compile_expr_tail<'a>(x: Expr<'a>, e: &mut Env, o: &mut Out) {
  match x {
    Expr::And(&(x, y)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let _ = o.emit(Inst::Cond(x));
      let p = o.emit_point(Some(0));
      let q = o.emit_point(Some(0));
      let _ = o.emit_label(0, [p]);
      let x = o.emit(Inst::ConstBool(false));
      let _ = o.emit(Inst::Put(x));
      let _ = o.emit(Inst::Ret);
      let _ = o.emit_label(0, [q]);
      compile_expr_tail(y, e, o);
    }
    Expr::Call(&(f, xs)) => {
      let f = compile_expr(f, e, o).into_value(e, o);
      for &x in xs {
        let x = compile_expr(x, e, o).into_value(e, o);
        put(x, &mut e.values);
      }
      for x in pop_list(xs.len(), &mut e.values) {
        let _ = o.emit(Inst::Put(x));
      }
      let _ = o.emit(Inst::TailCall(f));
    }
    Expr::If(&(x, ys)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let _ = o.emit(Inst::Cond(x));
      let p = o.emit_point(Some(0));
      let q = o.emit_point(Some(0));
      let _ = o.emit_label(0, [p]);
      let _ = o.emit(Inst::Ret);
      let _ = o.emit_label(0, [q]);
      compile_block_tail(ys, e, o);
    }
    Expr::IfElse(&(x, ys, zs)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let _ = o.emit(Inst::Cond(x));
      let p = o.emit_point(Some(0));
      let q = o.emit_point(Some(0));
      let _ = o.emit_label(0, [p]);
      compile_block_tail(zs, e, o);
      let _ = o.emit_label(0, [q]);
      compile_block_tail(ys, e, o);
    }
    Expr::Loop(xs) => {
      let p = o.emit_point(Some(0));
      let a = o.emit_label(0, [p]);
      put_loop_tail(a, &mut e.loops);
      let n = compile_block(xs, e, o).into_point_list(e, o);
      patch_point_list(a, pop_list(n, &mut e.points), o);
      pop_loop_tail(&mut e.loops);
    }
    Expr::Or(&(x, y)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let _ = o.emit(Inst::Cond(x));
      let p = o.emit_point(Some(0));
      let q = o.emit_point(Some(0));
      let _ = o.emit_label(0, [p]);
      compile_expr_tail(y, e, o);
      let _ = o.emit_label(0, [q]);
      let x = o.emit(Inst::ConstBool(true));
      let _ = o.emit(Inst::Put(x));
      let _ = o.emit(Inst::Ret);
    }
    Expr::Ternary(&(x, y, z)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let _ = o.emit(Inst::Cond(x));
      let p = o.emit_point(Some(0));
      let q = o.emit_point(Some(0));
      let _ = o.emit_label(0, [p]);
      compile_expr_tail(z, e, o);
      let _ = o.emit_label(0, [q]);
      compile_expr_tail(y, e, o);
    }
    x @ (
      | Expr::Bool(..)
      | Expr::Field(..)
      | Expr::Index(..)
      | Expr::Int(..)
      | Expr::Op1(..)
      | Expr::Op2(..)
      | Expr::Undefined
      | Expr::Variable(..)
    ) => {
      let What::NumValues(1) = compile_expr(x, e, o) else { unreachable!() };
      let _ = o.emit(Inst::Put(pop(&mut e.values)));
      let _ = o.emit(Inst::Ret);
    }
  }
}

fn compile_stmt<'a>(x: Stmt<'a>, e: &mut Env, o: &mut Out) -> What {
  match x {
    Stmt::ExprList(xs) => {
      return compile_expr_list(xs, e, o);
    }
    Stmt::Break(xs) => {
      match e.loops.break_counts.last() {
        None => {
          // error, break is not inside loop
          let _ = o.emit(Inst::GotoStaticError);
        }
        Some(None) => {
          compile_expr_list_tail(xs, e, o);
        }
        Some(Some(_)) => {
          let n = compile_expr_list(xs, e, o).into_point_list(e, o);
          for i in pop_list(n, &mut e.points) {
            put_break(i, &mut e.loops);
          }
        }
      }
      return What::NEVER;
    }
    Stmt::Continue => {
      match e.loops.labels.last() {
        None => {
          // error, break is not inside loop
          let _ = o.emit(Inst::GotoStaticError);
        }
        Some(a) => {
          // NB: all loop headers have arity zero
          let _ = o.emit(Inst::Goto(a.index));
        }
      }
      return What::NEVER;
    }
    Stmt::Let(xs, ys) => {
      // NB: we do the bindings from left to right, so later bindings shadow
      // earlier ones.
      compile_expr_list(ys, e, o).into_value_list(xs.len(), e, o);
      for (&x, y) in zip(xs, pop_list(xs.len(), &mut e.values)) {
        if let Some(x) = x.name {
          put_referent(x, Referent::Let(y), &mut e.scopes);
        }
      }
      return What::NIL;
    }
    Stmt::Return(xs) => {
      compile_expr_list_tail(xs, e, o);
      return What::NEVER;
    }
    Stmt::Set(s, x) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      if let Some(&Referent::Var(y)) = get_referent(s, &e.scopes) {
        let _ = o.emit(Inst::SetLocal(y, x));
      } else {
        // error, symbol does not refer to local variable
        let _ = o.emit(Inst::GotoStaticError);
        let _ = o.emit(Inst::Label(0));
      }
      return What::NIL;
    }
    Stmt::SetField(x, s, y) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let y = compile_expr(y, e, o).into_value(e, o);
      let _ = o.emit(Inst::SetField(x, s, y));
      return What::NIL;
    }
    Stmt::SetIndex(x, y, z) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let y = compile_expr(y, e, o).into_value(e, o);
      let z = compile_expr(z, e, o).into_value(e, o);
      let _ = o.emit(Inst::SetIndex(x, y, z));
      return What::NIL;
    }
    Stmt::Var(s, x) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let x = o.emit(Inst::DefLocal(x));
      put_referent(s, Referent::Var(x), &mut e.scopes);
      return What::NIL;
    }
  }
}

fn compile_stmt_tail<'a>(x: Stmt<'a>, e: &mut Env, o: &mut Out) {
  match x {
    Stmt::ExprList(xs) => {
      compile_expr_list_tail(xs, e, o);
    }
    x @ (
      | Stmt::Break(..)
      | Stmt::Continue
      | Stmt::Return(..)
    ) => {
      let What::NumPoints(0) = compile_stmt(x, e, o) else { unreachable!() };
    }
    x @ (
      | Stmt::Let(..)
      | Stmt::Set(..)
      | Stmt::SetField(..)
      | Stmt::SetIndex(..)
      | Stmt::Var(..)
    ) => {
      let What::NumValues(0) = compile_stmt(x, e, o) else { unreachable!() };
      let _ = o.emit(Inst::Ret);
    }
  }
}

fn compile_block<'a>(xs: &'a [Stmt<'a>], e: &mut Env, o: &mut Out) -> What {
  match xs.split_last() {
    None => {
      return What::NIL;
    }
    Some((&y, xs)) => {
      put_scope(&mut e.scopes);
      for &x in xs {
        compile_stmt(x, e, o).into_nil(e, o);
      }
      let w = compile_stmt(y, e, o);
      pop_scope(&mut e.scopes);
      return w;
    }
  }
}

fn compile_block_tail<'a>(xs: &'a [Stmt<'a>], e: &mut Env, o: &mut Out) {
  match xs.split_last() {
    None => {
      let _ = o.emit(Inst::Ret);
    }
    Some((&y, xs)) => {
      put_scope(&mut e.scopes);
      for &x in xs {
        compile_stmt(x, e, o).into_nil(e, o);
      }
      compile_stmt_tail(y, e, o);
      pop_scope(&mut e.scopes);
    }
  }
}

fn compile_expr_list<'a>(xs: &'a [Expr<'a>], e: &mut Env, o: &mut Out) -> What {
  match xs {
    &[x] => {
      return compile_expr(x, e, o);
    }
    xs => {
      for &x in xs {
        let x = compile_expr(x, e, o).into_value(e, o);
        put(x, &mut e.values);
      }
      return What::NumValues(xs.len());
    }
  }
}

fn compile_expr_list_tail<'a>(xs: &'a [Expr<'a>], e: &mut Env, o: &mut Out) {
  match xs {
    &[x] => {
      compile_expr_tail(x, e, o);
    }
    xs => {
      for &x in xs {
        let x = compile_expr(x, e, o).into_value(e, o);
        put(x, &mut e.values);
      }
      for x in pop_list(xs.len(), &mut e.values) {
        let _ = o.emit(Inst::Put(x));
      }
      let _ = o.emit(Inst::Ret);
    }
  }
}
