//! pass 1
//!
//! source text -> linearized bytecode
//!
//! operates on a single file

use crate::ast::Expr;
use crate::ast::Item;
use crate::ast::Stmt;
use crate::parse_ast::parse;
use crate::buf::Buf;
use crate::ir1::Inst;
use crate::symbol::Symbol;
use oxcart::Arena;
use std::iter::zip;
use tangerine::map::HashMap;

// TODO: consider special lowering for arguments to cond

pub fn compile<'a>(source: &[u8], arena: &mut Arena<'a>) -> Box<[Inst]> {
  let item_list = parse(source, arena);

  let mut e = Env::new();
  let mut o = Out::new();

  for Item::Fundef(f) in item_list.iter() {
    put_scope(&mut e.scopes);
    let _ = o.emit(Inst::Entry(f.args.len() as u32));

    for x in f.args {
      let y = o.emit(Inst::Pop);
      if let Some(x) = x.name {
        put_referent(x, Referent::Let(y), &mut e.scopes);
      }
    }

    compile_block_tail(f.body, &mut e, &mut o);
    pop_scope(&mut e.scopes);
  }

  return o.0.iter().map(|inst| *inst).collect::<Box<[_]>>();
}

enum What {
  NumPoints(u32),
  NumValues(u32),
}

enum Referent {
  Let(u32),
  Var(u32),
}

enum LoopInfo {
  TopLevel,
  Tail { label: Label },
  NonTail { label: Label, n_breaks: u32 },
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
  scopes: ScopeStack,
  loops: LoopStack,
  values: Buf<u32>,
  points: Buf<Point>,
}

impl Env {
  fn new() -> Self {
    return Self {
      scopes: ScopeStack::new(),
      loops: LoopStack::new(),
      values: Buf::new(),
      points: Buf::new(),
    };
  }
}

struct ScopeStack {
  counts: Buf<u32>,
  undo: Buf<(Symbol, Option<Referent>)>,
  table: HashMap<Symbol, Referent>,
}

impl ScopeStack {
  fn new() -> Self {
    return Self {
      counts: Buf::new(),
      undo: Buf::new(),
      table: HashMap::new()
    };
  }
}

struct LoopStack {
  info: Buf<LoopInfo>,
  breaks: Buf<Point>,
}

impl LoopStack {
  fn new() -> Self {
    let mut t = Self {
      info: Buf::new(),
      breaks: Buf::new()
    };
    t.info.put(LoopInfo::TopLevel);
    return t;
  }
}

fn put_scope(t: &mut ScopeStack) {
  t.counts.put(0);
}

fn pop_scope(t: &mut ScopeStack) {
  for (s, x) in t.undo.pop_list(t.counts.pop()) {
    match x {
      None => {
        let _ = t.table.remove(s);
      }
      Some(x) => {
        let _ = t.table.insert(s, x);
      }
    }
  }
}

fn put_referent(s: Symbol, x: Referent, t: &mut ScopeStack) {
  let y = t.table.insert(s, x);
  let n = t.counts.top_mut();
  t.undo.put((s, y));
  *n += 1;
}

fn get_referent(s: Symbol, t: &ScopeStack) -> Option<&Referent> {
  return t.table.get(s);
}

fn put_loop(a: Label, t: &mut LoopStack) {
  t.info.put(LoopInfo::NonTail { label: a, n_breaks: 0 });
}

fn pop_loop(t: &mut LoopStack, points: &mut Buf<Point>) -> u32 {
  let LoopInfo::NonTail { n_breaks, .. } = t.info.pop() else { unreachable!() };
  for p in t.breaks.pop_list(n_breaks) {
    points.put(p);
  }
  return n_breaks;
}

fn put_loop_tail(a: Label, t: &mut LoopStack) {
  t.info.put(LoopInfo::Tail { label: a });
}

fn pop_loop_tail(t: &mut LoopStack) {
  let _ = t.info.pop();
}

struct Out(Buf<Inst>);

impl Out {
  fn new() -> Self {
    Self(Buf::new())
  }

  fn emit(&mut self, inst: Inst) -> u32 {
    let n = self.0.len();
    self.0.put(inst);
    return n;
  }

  fn emit_point(&mut self, arity: Option<u32>) -> Point {
    let i = self.emit(Inst::Goto(u32::MAX));
    return Point { index: i, arity };
  }

  fn emit_label(&mut self, arity: u32, ps: impl IntoIterator<Item = Point>) -> Label {
    let a = self.emit(Inst::Label(arity));
    let a = Label { index: a, arity };
    patch_point_list(a, ps, self);
    return a;
  }
}

fn patch_point_list(a: Label, ps: impl IntoIterator<Item = Point>, o: &mut Out) {
  for i in ps {
    if let Some(n) = i.arity && n != a.arity {
      // error, arity mismatch
      o.0[i.index] = Inst::GotoStaticError;
    } else {
      o.0[i.index] = Inst::Goto(a.index);
    }
  }
}

impl What {
  const NEVER: Self = What::NumPoints(0);

  const NIL: Self = What::NumValues(0);

  fn into_nil(self, e: &mut Env, o: &mut Out) {
    match self {
      What::NumPoints(n_points) => {
        let _ = o.emit_label(0, e.points.pop_list(n_points));
      }
      What::NumValues(n_values) => {
        if n_values != 0 {
          // error, arity mismatch
          let _ = e.values.pop_list(n_values);
          let _ = o.emit(Inst::GotoStaticError);
          let _ = o.emit(Inst::Label(0));
        }
      }
    }
  }

  fn into_value(self, e: &mut Env, o: &mut Out) -> u32 {
    match self {
      What::NumPoints(n_points) => {
        let _ = o.emit_label(1, e.points.pop_list(n_points));
        let x = o.emit(Inst::Pop);
        return x;
      }
      What::NumValues(n_values) => {
        if n_values == 1 {
          return e.values.pop();
        } else {
          // error, arity mismatch
          let _ = e.values.pop_list(n_values);
          let _ = o.emit(Inst::GotoStaticError);
          let _ = o.emit(Inst::Label(1));
          let x = o.emit(Inst::Pop);
          return x;
        }
      }
    }
  }

  fn into_value_list(self, arity: u32, e: &mut Env, o: &mut Out) {
    match self {
      What::NumPoints(n_points) => {
        let _ = o.emit_label(arity, e.points.pop_list(n_points));
        for _ in 0 .. arity {
          let x = o.emit(Inst::Pop);
          e.values.put(x);
        }
      }
      What::NumValues(n_values) => {
        if arity != n_values {
          // error, arity mismatch
          let _ = e.values.pop_list(n_values);
          let _ = o.emit(Inst::GotoStaticError);
          let _ = o.emit(Inst::Label(arity));
          for _ in 0 .. arity {
            let x = o.emit(Inst::Pop);
            e.values.put(x);
          }
        }
      }
    }
  }

  fn into_point_list(self, e: &mut Env, o: &mut Out) -> u32 {
    match self {
      What::NumPoints(n_points) => {
        return n_points;
      }
      What::NumValues(n_values) => {
        for x in e.values.pop_list(n_values) {
          let _ = o.emit(Inst::Put(x));
        }
        let p = o.emit_point(Some(n_values));
        e.points.put(p);
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
      e.points.put(r);
      let _ = o.emit_label(0, [q]);
      let n = compile_expr(y, e, o).into_point_list(e, o);
      return What::NumPoints(1 + n);
    }
    Expr::Bool(x) => {
      let x = o.emit(Inst::ConstBool(x));
      e.values.put(x);
      return What::NumValues(1);
    }
    Expr::Call(&(f, xs)) => {
      let n = xs.len() as u32;
      let f = compile_expr(f, e, o).into_value(e, o);
      for &x in xs {
        let x = compile_expr(x, e, o).into_value(e, o);
        e.values.put(x);
      }
      for x in e.values.pop_list(n) {
        let _ = o.emit(Inst::Put(x));
      }
      let _ = o.emit(Inst::Call(f));
      let p = o.emit_point(None);
      e.points.put(p);
      return What::NumPoints(1);
    }
    Expr::Field(&(x, s)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let x = o.emit(Inst::Field(x, s));
      e.values.put(x);
      return What::NumValues(1);
    }
    Expr::If(&(x, ys)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let _ = o.emit(Inst::Cond(x));
      let p = o.emit_point(Some(0));
      let q = o.emit_point(Some(0));
      e.points.put(p);
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
      e.values.put(x);
      return What::NumValues(1);
    }
    Expr::Int(n) => {
      let x = o.emit(Inst::ConstInt(n));
      e.values.put(x);
      return What::NumValues(1);
    }
    Expr::Loop(xs) => {
      let p = o.emit_point(Some(0));
      let a = o.emit_label(0, [p]);
      put_loop(a, &mut e.loops);
      let m = compile_block(xs, e, o).into_point_list(e, o);
      patch_point_list(a, e.points.pop_list(m), o);
      let n = pop_loop(&mut e.loops, &mut e.points);
      return What::NumPoints(n);
    }
    Expr::Op1(&(f, x)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let x = o.emit(Inst::Op1(f, x));
      e.values.put(x);
      return What::NumValues(1);
    }
    Expr::Op2(&(f, x, y)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let y = compile_expr(y, e, o).into_value(e, o);
      let x = o.emit(Inst::Op2(f, x, y));
      e.values.put(x);
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
      e.points.put(r);
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
      e.values.put(x);
      return What::NumValues(1);
    }
    Expr::Variable(s) => {
      match get_referent(s, &e.scopes) {
        None => {
          let x = o.emit(Inst::Const(s));
          e.values.put(x);
        }
        Some(&Referent::Let(x)) => {
          e.values.put(x);
        }
        Some(&Referent::Var(x)) => {
          let x = o.emit(Inst::Local(x));
          e.values.put(x);
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
      let n = xs.len() as u32;
      let f = compile_expr(f, e, o).into_value(e, o);
      for &x in xs {
        let x = compile_expr(x, e, o).into_value(e, o);
        e.values.put(x);
      }
      for x in e.values.pop_list(n) {
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
      patch_point_list(a, e.points.pop_list(n), o);
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
      let _ = o.emit(Inst::Put(e.values.pop()));
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
      match e.loops.info.top() {
        LoopInfo::TopLevel => {
          // error, break is not inside loop
          let _ = o.emit(Inst::GotoStaticError);
        }
        LoopInfo::NonTail { .. } => {
          let n = compile_expr_list(xs, e, o).into_point_list(e, o);
          let LoopInfo::NonTail { n_breaks, .. } = e.loops.info.top_mut() else { unreachable!() };
          for p in e.points.pop_list(n) {
            e.loops.breaks.put(p);
            *n_breaks += 1;
          }
        }
        LoopInfo::Tail { .. } => {
          compile_expr_list_tail(xs, e, o);
        }
      }
      return What::NEVER;
    }
    Stmt::Continue => {
      match e.loops.info.top() {
        LoopInfo::TopLevel => {
          // error, break is not inside loop
          let _ = o.emit(Inst::GotoStaticError);
        }
        LoopInfo::NonTail { label, .. } | LoopInfo::Tail { label } => {
          // NB: all loop headers have arity zero
          let _ = o.emit(Inst::Goto(label.index));
        }
      }
      return What::NEVER;
    }
    Stmt::Let(xs, ys) => {
      let n = xs.len() as u32;
      // NB: we do the bindings from left to right, so later bindings shadow
      // earlier ones.
      compile_expr_list(ys, e, o).into_value_list(n, e, o);
      for (&x, y) in zip(xs, e.values.pop_list(n)) {
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
    Stmt::While(x, ys) => {
      let p = o.emit_point(Some(0));
      let a = o.emit_label(0, [p]);
      put_loop(a, &mut e.loops);
      let x = compile_expr(x, e, o).into_value(e, o);
      let _ = o.emit(Inst::Cond(x));
      let q = o.emit_point(Some(0));
      e.points.put(q);
      let r = o.emit_point(Some(0));
      let _ = o.emit_label(0, [r]);
      let m = compile_block(ys, e, o).into_point_list(e, o);
      patch_point_list(a, e.points.pop_list(m), o);
      let n = pop_loop(&mut e.loops, &mut e.points);
      return What::NumPoints(1 + n);
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
      | Stmt::While(..)
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
      let n = xs.len() as u32;
      for &x in xs {
        let x = compile_expr(x, e, o).into_value(e, o);
        e.values.put(x);
      }
      return What::NumValues(n);
    }
  }
}

fn compile_expr_list_tail<'a>(xs: &'a [Expr<'a>], e: &mut Env, o: &mut Out) {
  match xs {
    &[x] => {
      compile_expr_tail(x, e, o);
    }
    xs => {
      let n = xs.len() as u32;
      for &x in xs {
        let x = compile_expr(x, e, o).into_value(e, o);
        e.values.put(x);
      }
      for x in e.values.pop_list(n) {
        let _ = o.emit(Inst::Put(x));
      }
      let _ = o.emit(Inst::Ret);
    }
  }
}
