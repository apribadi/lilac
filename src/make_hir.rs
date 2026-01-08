//! lowering pass
//!
//! abstract syntax tree -> linearized bytecode + item references
//!
//! operates on a single source file

use crate::arr::Arr;
use crate::ast::Expr;
use crate::ast::Stmt;
use crate::ast;
use crate::buf::Buf;
use crate::hir::Fun;
use crate::hir::Inst;
use crate::hir::Module;
use crate::iter::enumerate;
use crate::symbol::Symbol;
use std::iter::zip;
use tangerine::map::HashMap;

// TODO: consider special lowering for arguments to cond

pub fn compile<'a>(item_list: &Arr<ast::Item<'a>>) -> Module {
  let mut ctx = Ctx::new();
  let mut out = Out::new();

  for ast::Item::Fun(f) in item_list.iter() {
    let pos = out.code.len();
    put_scope(&mut ctx.scopes);
    let _ = out.emit(Inst::Label(f.args.len() as u32));

    for (i, x) in enumerate(f.args.iter()) {
      let y = out.emit(Inst::Get(i));
      if let Some(x) = x.name {
        put_referent(x, Referent::Value(y), &mut ctx.scopes);
      }
    }

    compile_block_tail(f.body, &mut ctx, &mut out);
    pop_scope(&mut ctx.scopes);
    out.decl.put(Fun { name: f.name, pos, len: out.code.len() - pos });
  }

  return
    Module {
      code: out.code.drain().collect(),
      decl: out.decl.drain().collect(),
    };
}

enum What {
  NumPoints(u32),
  NumValues(u32),
}

enum Referent {
  Local(u32),
  Value(u32),
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

struct Ctx {
  scopes: ScopeStack,
  loops: LoopStack,
  values: Buf<u32>,
  points: Buf<Point>,
}

impl Ctx {
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
        t.table.remove(s);
      }
      Some(x) => {
        t.table.insert(s, x);
      }
    }
  }
}

fn put_referent(s: Symbol, x: Referent, t: &mut ScopeStack) {
  let y = t.table.get_and_insert(s, x);
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

struct Out {
  code: Buf<Inst>,
  decl: Buf<Fun>,
}

impl Out {
  fn new() -> Self {
    Self {
      code: Buf::new(),
      decl: Buf::new(),
    }
  }

  fn emit(&mut self, inst: Inst) -> u32 {
    let n = self.code.len();
    self.code.put(inst);
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

fn patch_point_list(a: Label, ps: impl IntoIterator<Item = Point>, out: &mut Out) {
  for i in ps {
    if let Some(n) = i.arity && n != a.arity {
      // error, arity mismatch
      out.code[i.index] = Inst::GotoStaticError;
    } else {
      out.code[i.index] = Inst::Goto(a.index);
    }
  }
}

impl What {
  const NEVER: Self = What::NumPoints(0);

  const NIL: Self = What::NumValues(0);

  fn into_nil(self, ctx: &mut Ctx, out: &mut Out) {
    match self {
      What::NumPoints(n_points) => {
        let _ = out.emit_label(0, ctx.points.pop_list(n_points));
      }
      What::NumValues(n_values) => {
        if n_values != 0 {
          // error, arity mismatch
          let _ = ctx.values.pop_list(n_values);
          let _ = out.emit(Inst::GotoStaticError);
          let _ = out.emit(Inst::Label(0));
        }
      }
    }
  }

  fn into_value(self, ctx: &mut Ctx, out: &mut Out) -> u32 {
    match self {
      What::NumPoints(n_points) => {
        let _ = out.emit_label(1, ctx.points.pop_list(n_points));
        let x = out.emit(Inst::Get(0));
        return x;
      }
      What::NumValues(n_values) => {
        if n_values == 1 {
          return ctx.values.pop();
        } else {
          // error, arity mismatch
          let _ = ctx.values.pop_list(n_values);
          let _ = out.emit(Inst::GotoStaticError);
          let _ = out.emit(Inst::Label(1));
          let x = out.emit(Inst::Get(0));
          return x;
        }
      }
    }
  }

  fn into_value_list(self, arity: u32, ctx: &mut Ctx, out: &mut Out) {
    match self {
      What::NumPoints(n_points) => {
        let _ = out.emit_label(arity, ctx.points.pop_list(n_points));
        for i in 0 .. arity {
          let x = out.emit(Inst::Get(i));
          ctx.values.put(x);
        }
      }
      What::NumValues(n_values) => {
        if arity != n_values {
          // error, arity mismatch
          let _ = ctx.values.pop_list(n_values);
          let _ = out.emit(Inst::GotoStaticError);
          let _ = out.emit(Inst::Label(arity));
          for i in 0 .. arity {
            let x = out.emit(Inst::Get(i));
            ctx.values.put(x);
          }
        }
      }
    }
  }

  fn into_point_list(self, ctx: &mut Ctx, out: &mut Out) -> u32 {
    match self {
      What::NumPoints(n_points) => {
        return n_points;
      }
      What::NumValues(n_values) => {
        for (i, x) in enumerate(ctx.values.pop_list(n_values)) {
          let _ = out.emit(Inst::Put(i, x));
        }
        let p = out.emit_point(Some(n_values));
        ctx.points.put(p);
        return 1;
      }
    }
  }
}

fn compile_expr<'a>(x: &Expr<'a>, ctx: &mut Ctx, out: &mut Out) -> What {
  match *x {
    Expr::And(&(ref x, ref y)) => {
      let x = compile_expr(x, ctx, out).into_value(ctx, out);
      let _ = out.emit(Inst::Cond(x));
      let p = out.emit_point(Some(0));
      let q = out.emit_point(Some(0));
      let _ = out.emit_label(0, [p]);
      let x = out.emit(Inst::ConstBool(false));
      let _ = out.emit(Inst::Put(0, x));
      let r = out.emit_point(Some(1));
      ctx.points.put(r);
      let _ = out.emit_label(0, [q]);
      let n = compile_expr(y, ctx, out).into_point_list(ctx, out);
      return What::NumPoints(1 + n);
    }
    Expr::Bool(x) => {
      let x = out.emit(Inst::ConstBool(x));
      ctx.values.put(x);
      return What::NumValues(1);
    }
    Expr::Call(&(ref f, ref xs)) => {
      let n = xs.len() as u32;
      for x in xs.iter() {
        let x = compile_expr(x, ctx, out).into_value(ctx, out);
        ctx.values.put(x);
      }
      let f = compile_expr(f, ctx, out).into_value(ctx, out); // NB: evaluate *after* args
      for (i, x) in enumerate(ctx.values.pop_list(n)) {
        let _ = out.emit(Inst::Put(i, x));
      }
      let _ = out.emit(Inst::Call(f));
      let p = out.emit_point(None);
      ctx.points.put(p);
      return What::NumPoints(1);
    }
    Expr::Field(&(ref x, s)) => {
      let x = compile_expr(x, ctx, out).into_value(ctx, out);
      let x = out.emit(Inst::Field(x, s));
      ctx.values.put(x);
      return What::NumValues(1);
    }
    Expr::If(&(ref x, ref ys)) => {
      let x = compile_expr(x, ctx, out).into_value(ctx, out);
      let _ = out.emit(Inst::Cond(x));
      let p = out.emit_point(Some(0));
      let q = out.emit_point(Some(0));
      ctx.points.put(p);
      let _ = out.emit_label(0, [q]);
      let n = compile_block(ys, ctx, out).into_point_list(ctx, out);
      return What::NumPoints(1 + n);
    }
    Expr::IfElse(&(ref x, ref ys, ref zs)) => {
      let x = compile_expr(x, ctx, out).into_value(ctx, out);
      let _ = out.emit(Inst::Cond(x));
      let p = out.emit_point(Some(0));
      let q = out.emit_point(Some(0));
      let _ = out.emit_label(0, [p]);
      let m = compile_block(zs, ctx, out).into_point_list(ctx, out);
      let _ = out.emit_label(0, [q]);
      let n = compile_block(ys, ctx, out).into_point_list(ctx, out);
      return What::NumPoints(m + n);
    }
    Expr::Index(&(ref x, ref y)) => {
      let x = compile_expr(x, ctx, out).into_value(ctx, out);
      let y = compile_expr(y, ctx, out).into_value(ctx, out);
      let x = out.emit(Inst::Index(x, y));
      ctx.values.put(x);
      return What::NumValues(1);
    }
    Expr::Int(n) => {
      let x = out.emit(Inst::ConstInt(n));
      ctx.values.put(x);
      return What::NumValues(1);
    }
    Expr::Loop(xs) => {
      let p = out.emit_point(Some(0));
      let a = out.emit_label(0, [p]);
      put_loop(a, &mut ctx.loops);
      let m = compile_block(xs, ctx, out).into_point_list(ctx, out);
      patch_point_list(a, ctx.points.pop_list(m), out);
      let n = pop_loop(&mut ctx.loops, &mut ctx.points);
      return What::NumPoints(n);
    }
    Expr::Op1(&(f, ref x)) => {
      let x = compile_expr(x, ctx, out).into_value(ctx, out);
      let x = out.emit(Inst::Op1(f, x));
      ctx.values.put(x);
      return What::NumValues(1);
    }
    Expr::Op2(&(f, ref x, ref y)) => {
      let x = compile_expr(x, ctx, out).into_value(ctx, out);
      let y = compile_expr(y, ctx, out).into_value(ctx, out);
      let x = out.emit(Inst::Op2(f, x, y));
      ctx.values.put(x);
      return What::NumValues(1);
    }
    Expr::Or(&(ref x, ref y)) => {
      let x = compile_expr(x, ctx, out).into_value(ctx, out);
      let _ = out.emit(Inst::Cond(x));
      let p = out.emit_point(Some(0));
      let q = out.emit_point(Some(0));
      let _ = out.emit_label(0, [p]);
      let n = compile_expr(y, ctx, out).into_point_list(ctx, out);
      let _ = out.emit_label(0, [q]);
      let x = out.emit(Inst::ConstBool(true));
      let _ = out.emit(Inst::Put(0, x));
      let r = out.emit_point(Some(1));
      ctx.points.put(r);
      return What::NumPoints(n + 1);
    }
    Expr::PostOp(&(f, ref x)) => {
      if let Expr::Variable(s) = *x {
        if let Some(&Referent::Local(v)) = get_referent(s, &ctx.scopes) {
          let x = out.emit(Inst::GetLocal(v));
          let y = out.emit(Inst::Op1(f, x));
          let _ = out.emit(Inst::SetLocal(v, y));
          ctx.values.put(x); // post
          return What::NumValues(1);
        }
      }
      // error, post-op must target a local variable
      let _ = out.emit(Inst::GotoStaticError);
      let _ = out.emit(Inst::Label(1));
      let x = out.emit(Inst::Get(0));
      ctx.values.put(x);
      return What::NumValues(1);
    }
    Expr::PreOp(&(f, ref x)) => {
      if let Expr::Variable(s) = *x {
        if let Some(&Referent::Local(v)) = get_referent(s, &ctx.scopes) {
          let x = out.emit(Inst::GetLocal(v));
          let y = out.emit(Inst::Op1(f, x));
          let _ = out.emit(Inst::SetLocal(v, y));
          ctx.values.put(y); // pre
          return What::NumValues(1);
        }
      }
      // error, pre-op must target a local variable
      let _ = out.emit(Inst::GotoStaticError);
      let _ = out.emit(Inst::Label(1));
      let x = out.emit(Inst::Get(0));
      ctx.values.put(x);
      return What::NumValues(1);
    }
    Expr::Ternary(&(ref x, ref y, ref z)) => {
      let x = compile_expr(x, ctx, out).into_value(ctx, out);
      let _ = out.emit(Inst::Cond(x));
      let p = out.emit_point(Some(0));
      let q = out.emit_point(Some(0));
      let _ = out.emit_label(0, [p]);
      let m = compile_expr(z, ctx, out).into_point_list(ctx, out);
      let _ = out.emit_label(0, [q]);
      let n = compile_expr(y, ctx, out).into_point_list(ctx, out);
      return What::NumPoints(m + n);
    }
    Expr::Undefined => {
      // error, evaluating undefined expression
      let _ = out.emit(Inst::GotoStaticError);
      let _ = out.emit(Inst::Label(1));
      let x = out.emit(Inst::Get(0));
      ctx.values.put(x);
      return What::NumValues(1);
    }
    Expr::Variable(s) => {
      match get_referent(s, &ctx.scopes) {
        None => {
          let x = out.emit(Inst::Const(s));
          ctx.values.put(x);
        }
        Some(&Referent::Value(x)) => {
          ctx.values.put(x);
        }
        Some(&Referent::Local(v)) => {
          let x = out.emit(Inst::GetLocal(v));
          ctx.values.put(x);
        }
      }
      return What::NumValues(1);
    }
  }
}

fn compile_expr_tail<'a>(x: &Expr<'a>, ctx: &mut Ctx, out: &mut Out) {
  match *x {
    Expr::And(&(ref x, ref y)) => {
      let x = compile_expr(x, ctx, out).into_value(ctx, out);
      let _ = out.emit(Inst::Cond(x));
      let p = out.emit_point(Some(0));
      let q = out.emit_point(Some(0));
      let _ = out.emit_label(0, [p]);
      let x = out.emit(Inst::ConstBool(false));
      let _ = out.emit(Inst::Put(0, x));
      let _ = out.emit(Inst::Ret);
      let _ = out.emit_label(0, [q]);
      compile_expr_tail(y, ctx, out);
    }
    Expr::Call(&(ref f, ref xs)) => {
      let n = xs.len() as u32;
      for x in xs.iter() {
        let x = compile_expr(x, ctx, out).into_value(ctx, out);
        ctx.values.put(x);
      }
      let f = compile_expr(f, ctx, out).into_value(ctx, out); // NB: evaluate *after* args
      for (i, x) in enumerate(ctx.values.pop_list(n)) {
        let _ = out.emit(Inst::Put(i, x));
      }
      let _ = out.emit(Inst::TailCall(f));
    }
    Expr::If(&(ref x, ref ys)) => {
      let x = compile_expr(x, ctx, out).into_value(ctx, out);
      let _ = out.emit(Inst::Cond(x));
      let p = out.emit_point(Some(0));
      let q = out.emit_point(Some(0));
      let _ = out.emit_label(0, [p]);
      let _ = out.emit(Inst::Ret);
      let _ = out.emit_label(0, [q]);
      compile_block_tail(ys, ctx, out);
    }
    Expr::IfElse(&(ref x, ref ys, ref zs)) => {
      let x = compile_expr(x, ctx, out).into_value(ctx, out);
      let _ = out.emit(Inst::Cond(x));
      let p = out.emit_point(Some(0));
      let q = out.emit_point(Some(0));
      let _ = out.emit_label(0, [p]);
      compile_block_tail(zs, ctx, out);
      let _ = out.emit_label(0, [q]);
      compile_block_tail(ys, ctx, out);
    }
    Expr::Loop(xs) => {
      let p = out.emit_point(Some(0));
      let a = out.emit_label(0, [p]);
      put_loop_tail(a, &mut ctx.loops);
      let n = compile_block(xs, ctx, out).into_point_list(ctx, out);
      patch_point_list(a, ctx.points.pop_list(n), out);
      pop_loop_tail(&mut ctx.loops);
    }
    Expr::Or(&(ref x, ref y)) => {
      let x = compile_expr(x, ctx, out).into_value(ctx, out);
      let _ = out.emit(Inst::Cond(x));
      let p = out.emit_point(Some(0));
      let q = out.emit_point(Some(0));
      let _ = out.emit_label(0, [p]);
      compile_expr_tail(y, ctx, out);
      let _ = out.emit_label(0, [q]);
      let x = out.emit(Inst::ConstBool(true));
      let _ = out.emit(Inst::Put(0, x));
      let _ = out.emit(Inst::Ret);
    }
    Expr::Ternary(&(ref x, ref y, ref z)) => {
      let x = compile_expr(x, ctx, out).into_value(ctx, out);
      let _ = out.emit(Inst::Cond(x));
      let p = out.emit_point(Some(0));
      let q = out.emit_point(Some(0));
      let _ = out.emit_label(0, [p]);
      compile_expr_tail(z, ctx, out);
      let _ = out.emit_label(0, [q]);
      compile_expr_tail(y, ctx, out);
    }
    Expr::Bool(..)
    | Expr::Field(..)
    | Expr::Index(..)
    | Expr::Int(..)
    | Expr::Op1(..)
    | Expr::Op2(..)
    | Expr::PostOp(..)
    | Expr::PreOp(..)
    | Expr::Undefined
    | Expr::Variable(..) => {
      let What::NumValues(1) = compile_expr(x, ctx, out) else { unreachable!() };
      let _ = out.emit(Inst::Put(0, ctx.values.pop()));
      let _ = out.emit(Inst::Ret);
    }
  }
}

fn compile_stmt<'a>(x: &Stmt<'a>, ctx: &mut Ctx, out: &mut Out) -> What {
  match *x {
    Stmt::ExprList(xs) => {
      return compile_expr_list(xs, ctx, out);
    }
    Stmt::Break(xs) => {
      match ctx.loops.info.top() {
        LoopInfo::TopLevel => {
          // error, break is not inside loop
          let _ = out.emit(Inst::GotoStaticError);
        }
        LoopInfo::NonTail { .. } => {
          let n = compile_expr_list(xs, ctx, out).into_point_list(ctx, out);
          let LoopInfo::NonTail { n_breaks, .. } = ctx.loops.info.top_mut() else { unreachable!() };
          for p in ctx.points.pop_list(n) {
            ctx.loops.breaks.put(p);
            *n_breaks += 1;
          }
        }
        LoopInfo::Tail { .. } => {
          compile_expr_list_tail(xs, ctx, out);
        }
      }
      return What::NEVER;
    }
    Stmt::Continue => {
      match ctx.loops.info.top() {
        LoopInfo::TopLevel => {
          // error, break is not inside loop
          let _ = out.emit(Inst::GotoStaticError);
        }
        LoopInfo::NonTail { label, .. } | LoopInfo::Tail { label } => {
          // NB: all loop headers have arity zero
          let _ = out.emit(Inst::Goto(label.index));
        }
      }
      return What::NEVER;
    }
    Stmt::Let(xs, ys) => {
      let n = xs.len() as u32;
      // NB: we do the bindings from left to right, so later bindings shadow
      // earlier ones.
      compile_expr_list(ys, ctx, out).into_value_list(n, ctx, out);
      for (x, y) in zip(xs, ctx.values.pop_list(n)) {
        if let Some(x) = x.name {
          put_referent(x, Referent::Value(y), &mut ctx.scopes);
        }
      }
      return What::NIL;
    }
    Stmt::Return(xs) => {
      compile_expr_list_tail(xs, ctx, out);
      return What::NEVER;
    }
    Stmt::Set(s, ref x) => {
      let x = compile_expr(x, ctx, out).into_value(ctx, out);
      if let Some(&Referent::Local(v)) = get_referent(s, &ctx.scopes) {
        let _ = out.emit(Inst::SetLocal(v, x));
      } else {
        // error, symbol does not refer to local variable
        let _ = out.emit(Inst::GotoStaticError);
        let _ = out.emit(Inst::Label(0));
      }
      return What::NIL;
    }
    Stmt::SetField(ref x, s, ref y) => {
      let x = compile_expr(x, ctx, out).into_value(ctx, out);
      let y = compile_expr(y, ctx, out).into_value(ctx, out);
      let _ = out.emit(Inst::SetField(x, s, y));
      return What::NIL;
    }
    Stmt::SetIndex(ref x, ref y, ref z) => {
      let x = compile_expr(x, ctx, out).into_value(ctx, out);
      let y = compile_expr(y, ctx, out).into_value(ctx, out);
      let z = compile_expr(z, ctx, out).into_value(ctx, out);
      let _ = out.emit(Inst::SetIndex(x, y, z));
      return What::NIL;
    }
    Stmt::Var(s, ref x) => {
      let x = compile_expr(x, ctx, out).into_value(ctx, out);
      let x = out.emit(Inst::Local(x));
      put_referent(s, Referent::Local(x), &mut ctx.scopes);
      return What::NIL;
    }
    Stmt::While(ref x, ys) => {
      let p = out.emit_point(Some(0));
      let a = out.emit_label(0, [p]);
      put_loop(a, &mut ctx.loops);
      let x = compile_expr(x, ctx, out).into_value(ctx, out);
      let _ = out.emit(Inst::Cond(x));
      let q = out.emit_point(Some(0));
      ctx.points.put(q);
      let r = out.emit_point(Some(0));
      let _ = out.emit_label(0, [r]);
      let m = compile_block(ys, ctx, out).into_point_list(ctx, out);
      patch_point_list(a, ctx.points.pop_list(m), out);
      let n = pop_loop(&mut ctx.loops, &mut ctx.points);
      return What::NumPoints(1 + n);
    }
  }
}

fn compile_stmt_tail<'a>(x: &Stmt<'a>, ctx: &mut Ctx, out: &mut Out) {
  match *x {
    Stmt::ExprList(xs) => {
      compile_expr_list_tail(xs, ctx, out);
    }
    | Stmt::Break(..)
    | Stmt::Continue
    | Stmt::Return(..) => {
      let What::NumPoints(0) = compile_stmt(x, ctx, out) else { unreachable!() };
    }
    Stmt::Let(..)
    | Stmt::Set(..)
    | Stmt::SetField(..)
    | Stmt::SetIndex(..)
    | Stmt::Var(..)
    | Stmt::While(..) => {
      let What::NumValues(0) = compile_stmt(x, ctx, out) else { unreachable!() };
      let _ = out.emit(Inst::Ret);
    }
  }
}

fn compile_block<'a>(xs: &'a [Stmt<'a>], ctx: &mut Ctx, out: &mut Out) -> What {
  match xs.split_last() {
    None => {
      return What::NIL;
    }
    Some((y, xs)) => {
      put_scope(&mut ctx.scopes);
      for x in xs {
        compile_stmt(x, ctx, out).into_nil(ctx, out);
      }
      let w = compile_stmt(y, ctx, out);
      pop_scope(&mut ctx.scopes);
      return w;
    }
  }
}

fn compile_block_tail<'a>(xs: &'a [Stmt<'a>], ctx: &mut Ctx, out: &mut Out) {
  match xs.split_last() {
    None => {
      let _ = out.emit(Inst::Ret);
    }
    Some((y, xs)) => {
      put_scope(&mut ctx.scopes);
      for x in xs {
        compile_stmt(x, ctx, out).into_nil(ctx, out);
      }
      compile_stmt_tail(y, ctx, out);
      pop_scope(&mut ctx.scopes);
    }
  }
}

fn compile_expr_list<'a>(xs: &'a [Expr<'a>], ctx: &mut Ctx, out: &mut Out) -> What {
  match xs {
    [x] => {
      return compile_expr(x, ctx, out);
    }
    xs => {
      let n = xs.len() as u32;
      for x in xs.iter() {
        let x = compile_expr(x, ctx, out).into_value(ctx, out);
        ctx.values.put(x);
      }
      return What::NumValues(n);
    }
  }
}

fn compile_expr_list_tail<'a>(xs: &'a [Expr<'a>], ctx: &mut Ctx, out: &mut Out) {
  match xs {
    [x] => {
      compile_expr_tail(x, ctx, out);
    }
    xs => {
      let n = xs.len() as u32;
      for x in xs.iter() {
        let x = compile_expr(x, ctx, out).into_value(ctx, out);
        ctx.values.put(x);
      }
      for (i, x) in enumerate(ctx.values.pop_list(n)) {
        let _ = out.emit(Inst::Put(i, x));
      }
      let _ = out.emit(Inst::Ret);
    }
  }
}
