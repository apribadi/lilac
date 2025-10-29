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

struct LoopInfo {
  top: Label,
  bot: Option<usize>,
}

struct Env {
  scopes: Scopes,
  loops: Vec<LoopInfo>,
  breaks: Vec<Point>,
  values: Vec<u32>,
  points: Vec<Point>,
}

impl Env {
  fn new() -> Self {
    Self {
      scopes: Scopes::new(),
      loops: Vec::new(),
      breaks: Vec::new(),
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

fn put_loop(a: Label, e: &mut Env) {
  e.loops.push(LoopInfo { top: a, bot: Some(0) });
}

fn pop_loop(e: &mut Env) -> usize {
  let n = pop(&mut e.loops).bot.unwrap();
  for i in pop_list(n, &mut e.breaks) {
    e.points.push(i);
  }
  return n;
}

fn put_loop_tail(a: Label, e: &mut Env) {
  e.loops.push(LoopInfo { top: a, bot: None });
}

fn pop_loop_tail(e: &mut Env) {
  let _ = e.loops.pop();
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

fn put_break_point(i: Point, e: &mut Env) {
  let n = e.loops.last_mut().unwrap().bot.as_mut().unwrap();
  e.breaks.push(i);
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

  fn emit_label(&mut self, arity: usize) -> Label {
    let n = arity as u32;
    let a = self.emit(Inst::Label(n));
    return Label { index: a, arity: n};
  }

  fn emit_point(&mut self, arity: Option<usize>) -> Point {
    let i = self.emit(Inst::Goto(u32::MAX));
    let n = arity.map(|n| n as u32);
    return Point { index: i, arity: n };
  }

  fn emit_label_and_patch_point_list(
      &mut self,
      arity: usize,
      points: impl IntoIterator<Item = Point>
    ) -> Label {
    let a = self.emit_label(arity);
    patch_point_list(a, points, self);
    return a;
  }
}

fn patch_point_list(a: Label, points: impl IntoIterator<Item = Point>, o: &mut Out) {
  for i in points {
    match i.arity {
      Some(n) if n != a.arity => {
        // error, arity mismatch
        o.0[i.index as usize] = Inst::GotoStaticError;
      }
      _ => {
        o.0[i.index as usize] = Inst::Goto(a.index);
      }
    }
  }
}

impl What {
  const NEVER: Self = What::NumPoints(0);

  const NIL: Self = What::NumValues(0);

  fn into_nil(self, e: &mut Env, o: &mut Out) {
    match self {
      What::NumPoints(n) => {
        let _ = o.emit_label_and_patch_point_list(0, pop_list(n, &mut e.points));
      }
      What::NumValues(n) => {
        if n != 0 {
          // error, arity mismatch
          let _ = pop_list(n, &mut e.values);
          let _ = o.emit(Inst::GotoStaticError);
          let _ = o.emit(Inst::Label(0));
        }
      }
    }
  }

  fn into_value(self, e: &mut Env, o: &mut Out) -> u32 {
    match self {
      What::NumPoints(n) => {
        let _ = o.emit_label_and_patch_point_list(1, pop_list(n, &mut e.points));
        let x = o.emit(Inst::Pop);
        return x;
      }
      What::NumValues(n) => {
        if n == 1 {
          return pop(&mut e.values);
        } else {
          // error, arity mismatch
          let _ = pop_list(n, &mut e.values);
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
      What::NumPoints(n) => {
        let _ = o.emit_label_and_patch_point_list(arity, pop_list(n, &mut e.points));
        for _ in 0 .. arity {
          let x = o.emit(Inst::Pop);
          put(x, &mut e.values);
        }
      }
      What::NumValues(n) => {
        if arity != n {
          // error, arity mismatch
          let _ = pop_list(n, &mut e.values);
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
      What::NumPoints(n) => {
        return n;
      }
      What::NumValues(n) => {
        for x in pop_list(n, &mut e.values) {
          let _ = o.emit(Inst::Put(x));
        }
        let i = o.emit_point(Some(n));
        put(i, &mut e.points);
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
      let i = o.emit_point(Some(0));
      let j = o.emit_point(Some(0));
      let _ = o.emit_label_and_patch_point_list(0, [i]);
      let x = o.emit(Inst::ConstBool(false));
      let _ = o.emit(Inst::Put(x));
      let k = o.emit_point(Some(1));
      put(k, &mut e.points);
      let _ = o.emit_label_and_patch_point_list(0, [j]);
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
      let i = o.emit_point(None);
      put(i, &mut e.points);
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
      let i = o.emit_point(Some(0));
      let j = o.emit_point(Some(0));
      put(i, &mut e.points);
      let _ = o.emit_label_and_patch_point_list(0, [j]);
      let n = compile_block(ys, e, o).into_point_list(e, o);
      return What::NumPoints(1 + n);
    }
    Expr::IfElse(&(x, ys, zs)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let _ = o.emit(Inst::Cond(x));
      let i = o.emit_point(Some(0));
      let j = o.emit_point(Some(0));
      let _ = o.emit_label_and_patch_point_list(0, [i]);
      let m = compile_block(zs, e, o).into_point_list(e, o);
      let _ = o.emit_label_and_patch_point_list(0, [j]);
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
      let i = o.emit_point(Some(0));
      let a = o.emit_label_and_patch_point_list(0, [i]);
      put_loop(a, e);
      let n = compile_block(xs, e, o).into_point_list(e, o);
      patch_point_list(a, pop_list(n, &mut e.points), o);
      let m = pop_loop(e);
      return What::NumPoints(m);
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
      let i = o.emit_point(Some(0));
      let j = o.emit_point(Some(0));
      let _ = o.emit_label_and_patch_point_list(0, [i]);
      let n = compile_expr(y, e, o).into_point_list(e, o);
      let _ = o.emit_label_and_patch_point_list(0, [j]);
      let x = o.emit(Inst::ConstBool(true));
      let _ = o.emit(Inst::Put(x));
      let k = o.emit_point(Some(1));
      put(k, &mut e.points);
      return What::NumPoints(n + 1);
    }
    Expr::Ternary(&(x, y, z)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let _ = o.emit(Inst::Cond(x));
      let i = o.emit_point(Some(0));
      let j = o.emit_point(Some(0));
      let _ = o.emit_label_and_patch_point_list(0, [i]);
      let m = compile_expr(z, e, o).into_point_list(e, o);
      let _ = o.emit_label_and_patch_point_list(0, [j]);
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
      let i = o.emit_point(Some(0));
      let j = o.emit_point(Some(0));
      let _ = o.emit_label_and_patch_point_list(0, [i]);
      let x = o.emit(Inst::ConstBool(false));
      let _ = o.emit(Inst::Put(x));
      let _ = o.emit(Inst::Ret);
      let _ = o.emit_label_and_patch_point_list(0, [j]);
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
      let i = o.emit_point(Some(0));
      let j = o.emit_point(Some(0));
      let _ = o.emit_label_and_patch_point_list(0, [i]);
      let _ = o.emit(Inst::Ret);
      let _ = o.emit_label_and_patch_point_list(0, [j]);
      compile_block_tail(ys, e, o);
    }
    Expr::IfElse(&(x, ys, zs)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let _ = o.emit(Inst::Cond(x));
      let i = o.emit_point(Some(0));
      let j = o.emit_point(Some(0));
      let _ = o.emit_label_and_patch_point_list(0, [i]);
      compile_block_tail(zs, e, o);
      let _ = o.emit_label_and_patch_point_list(0, [j]);
      compile_block_tail(ys, e, o);
    }
    Expr::Loop(xs) => {
      let i = o.emit_point(Some(0));
      let a = o.emit_label_and_patch_point_list(0, [i]);
      put_loop_tail(a, e);
      let n = compile_block(xs, e, o).into_point_list(e, o);
      patch_point_list(a, pop_list(n, &mut e.points), o);
      pop_loop_tail(e);
    }
    Expr::Or(&(x, y)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let _ = o.emit(Inst::Cond(x));
      let i = o.emit_point(Some(0));
      let j = o.emit_point(Some(0));
      let _ = o.emit_label_and_patch_point_list(0, [i]);
      compile_expr_tail(y, e, o);
      let _ = o.emit_label_and_patch_point_list(0, [j]);
      let x = o.emit(Inst::ConstBool(true));
      let _ = o.emit(Inst::Put(x));
      let _ = o.emit(Inst::Ret);
    }
    Expr::Ternary(&(x, y, z)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let _ = o.emit(Inst::Cond(x));
      let i = o.emit_point(Some(0));
      let j = o.emit_point(Some(0));
      let _ = o.emit_label_and_patch_point_list(0, [i]);
      compile_expr_tail(z, e, o);
      let _ = o.emit_label_and_patch_point_list(0, [j]);
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
      match e.loops.last() {
        None => {
          // error, break is not inside loop
          let _ = o.emit(Inst::GotoStaticError);
        }
        Some(LoopInfo { bot: None, .. }) => {
          compile_expr_list_tail(xs, e, o);
        }
        Some(LoopInfo { bot: Some(_), .. }) => {
          let n = compile_expr_list(xs, e, o).into_point_list(e, o);
          for _ in 0 .. n {
            let i = pop(&mut e.points);
            put_break_point(i, e);
          }
        }
      }
      return What::NEVER;
    }
    Stmt::Continue => {
      match e.loops.last() {
        None => {
          // error, break is not inside loop
          let _ = o.emit(Inst::GotoStaticError);
        }
        Some(loop_info) => {
          // NB: all loop headers have arity zero
          let _ = o.emit(Inst::Goto(loop_info.top.index));
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
      match get_referent(s, &e.scopes) {
        Some(&Referent::Var(y)) => {
          let _ = o.emit(Inst::SetLocal(y, x));
          return What::NIL;
        }
        _ => {
          // error, symbol does not refer to local variable
          let _ = o.emit(Inst::GotoStaticError);
          let _ = o.emit(Inst::Label(0));
          return What::NIL;
        }
      }
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
