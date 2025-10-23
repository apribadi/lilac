use crate::ast::Expr;
use crate::ast::Stmt;
use crate::symbol::Symbol;
use crate::symbol_table::SymbolTable;
use crate::uir::Inst;

// TODO: fundef

// TODO: unresolved patch points should be tagged with an arity
//
// enum _ {
//   Known(usize),
//   Unknown
// }
//
// where Unknown comes from calls with exactly one continuation but with
// unknown arity.
//
// Then, when we reolve patch points we can check arity.
//
// On an arity mismatch, we can replace JUMP block terminators with
// GOTO-STATIC-ERROR, but it's not clear what to do with patch points that are
// part of a, e.g. COND.
//
// e.g.
//
// let x = if ... { ... }
//
// should generate an arity error from (at least) one of the COND arms.
//
// Perhaps we could have a GOTO-STATIC-ERROR instruction that behaves
// similarly to GOTO? Or we could lazily generate a block
// { LABEL; GOTO-STATIC-ERROR } at the end of the function?

// NOTE:
//
// Functions either have
// - one ordinary continuation, or
// - N tagged continuations, where N = 0 or 1 is possible
//
// A tail call is ambiguous, in that it could be a call to either type of
// function.
//
// An ordinary function call in non-tail position is always the first case, a
// call to a function with one ordinary continuation.
//
// We'll have special syntax for non-tail calls to functions with tagged
// continuations, even if it has 0 or 1 continuations.
//
// (You could imagine propagating these special contexts to
// sub-expressions in a similar way that we do tail contexts - you would
// be unable to evaluate, e.g., operator expressions in such a context)
//
// OR
//
// we could have a syntax where the function receives multiple continuations
// explicitly and names them

enum Binding {
  Let(u32),
  Var(u32),
}

enum LoopBreakTarget {
  Tail,
  NonTail(usize),
}

struct Env {
  symbol_table: SymbolTable<Binding>,
  loops: Vec<LoopBreakTarget>,
  break_points: Vec<u32>,
  continue_labels: Vec<u32>,
  values: Vec<u32>,
  points: Vec<u32>,
}

impl Env {
  fn new() -> Self {
    Self {
      symbol_table: SymbolTable::new(),
      loops: Vec::new(),
      break_points: Vec::new(),
      continue_labels: Vec::new(),
      values: Vec::new(),
      points: Vec::new(),
    }
  }
}

fn put_let(s: Symbol, x: u32, e: &mut Env) {
  e.symbol_table.insert(s, Binding::Let(x));
}

fn put_var(s: Symbol, x: u32, e: &mut Env) {
  e.symbol_table.insert(s, Binding::Var(x));
}

fn get_binding(s: Symbol, e: &Env) -> Option<&Binding> {
  return e.symbol_table.get(s);
}

fn put_loop(a: u32, e: &mut Env) {
  e.loops.push(LoopBreakTarget::NonTail(0));
  e.continue_labels.push(a);
}

fn pop_loop(e: &mut Env) -> usize {
  let _ = e.continue_labels.pop().unwrap();
  match e.loops.pop() {
    Some(LoopBreakTarget::NonTail(n)) => {
      for _ in 0 .. n {
        let i = e.break_points.pop().unwrap();
        e.points.push(i);
      }
      return n;
    }
    _ => unreachable!()
  }
}

fn put_loop_tail(a: u32, e: &mut Env) {
  e.loops.push(LoopBreakTarget::Tail);
  e.continue_labels.push(a);
}

fn pop_loop_tail(e: &mut Env) {
  let _ = e.continue_labels.pop().unwrap();
  match e.loops.pop() {
    Some(LoopBreakTarget::Tail) => (),
    _ => unreachable!()
  }
}

fn put_scope(e: &mut Env) {
  e.symbol_table.put_scope();
}

fn pop_scope(e: &mut Env) {
  e.symbol_table.pop_scope();
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

fn put_point(i: u32, e: &mut Env) {
  e.points.push(i);
}

fn put_break_point(i: u32, e: &mut Env) {
  e.break_points.push(i);
  match e.loops.last_mut() {
    Some(LoopBreakTarget::NonTail(n)) => {
      *n += 1;
    }
    _ => unreachable!() // ???
  }
}

fn pop_point(e: &mut Env) -> u32 {
  return e.points.pop().unwrap();
}

fn pop_points(n: usize, e: &mut Env) -> impl Iterator<Item = u32> {
  return e.points.drain(e.points.len() - n ..);
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

  fn emit_point(&mut self) -> u32 {
    return self.emit(Inst::Goto(u32::MAX));
  }

  fn emit_label_and_patch_points(&mut self, points: impl IntoIterator<Item = u32>) -> u32 {
    let a = self.emit(Inst::Label);
    patch_points(a, points, self);
    return a;
  }
}

fn patch_points(a: u32, points: impl IntoIterator<Item = u32>, o: &mut Out) {
  for i in points {
    o.0[i as usize] = Inst::Goto(a);
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
  const NEVER: Self = What::NumPoints(0);

  const NIL: Self = What::NumValues(0);

  fn into_nil(self, e: &mut Env, o: &mut Out) {
    match self {
      What::NumPoints(n) => {
        let _ = o.emit_label_and_patch_points(pop_points(n, e));
      }
      What::NumValues(0) => {
      }
      What::NumValues(n) => {
        // error, arity mismatch
        let _ = pop_values(n, e);
        let _ = o.emit(Inst::GotoStaticError);
        let _ = o.emit(Inst::Label);
      }
    }
  }

  fn into_value(self, e: &mut Env, o: &mut Out) -> u32 {
    match self {
      What::NumPoints(n) => {
        let _ = o.emit_label_and_patch_points(pop_points(n, e));
        let x = o.emit(Inst::Pop);
        return x;
      }
      What::NumValues(1) => {
        return pop_value(e);
      }
      What::NumValues(n) => {
        // error, arity mismatch
        let _ = pop_values(n, e);
        let _ = o.emit(Inst::GotoStaticError);
        let _ = o.emit(Inst::Label);
        let x = o.emit(Inst::Pop);
        return x;
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
        let i = o.emit_point();
        put_point(i, e);
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
      let i = o.emit_point();
      let j = o.emit_point();
      let _ = o.emit_label_and_patch_points([i]);
      let x = o.emit(Inst::ConstBool(false));
      let _ = o.emit(Inst::Put(x));
      let k = o.emit_point();
      put_point(k, e);
      let _ = o.emit_label_and_patch_points([j]);
      let n = compile_expr(y, e, o).into_points(e, o);
      return What::NumPoints(1 + n);
    }
    Expr::Bool(x) => {
      let x = o.emit(Inst::ConstBool(x));
      put_value(x, e);
      return What::NumValues(1);
    }
    Expr::Call(&(f, xs)) => {
      let n = xs.len();
      let f = compile_expr(f, e, o).into_value(e, o);
      for &x in xs.iter() {
        let x = compile_expr(x, e, o).into_value(e, o);
        put_value(x, e);
      }
      for x in pop_values(n, e) {
        let _ = o.emit(Inst::Put(x));
      }
      let _ = o.emit(Inst::Call(f));
      let i = o.emit_point();
      put_point(i, e);
      return What::NumPoints(1);
    }
    Expr::Field(&(x, s)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let x = o.emit(Inst::Field(x, s));
      put_value(x, e);
      return What::NumValues(1);
    }
    Expr::If(&(x, ys)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let _ = o.emit(Inst::Cond(x));
      let i = o.emit_point();
      let j = o.emit_point();
      put_point(i, e);
      let _ = o.emit_label_and_patch_points([j]);
      let n = compile_block(ys, e, o).into_points(e, o);
      return What::NumPoints(1 + n);
    }
    Expr::IfElse(&(x, ys, zs)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let _ = o.emit(Inst::Cond(x));
      let i = o.emit_point();
      let j = o.emit_point();
      let _ = o.emit_label_and_patch_points([i]);
      let m = compile_block(zs, e, o).into_points(e, o);
      let _ = o.emit_label_and_patch_points([j]);
      let n = compile_block(ys, e, o).into_points(e, o);
      return What::NumPoints(m + n);
    }
    Expr::Index(&(x, y)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let y = compile_expr(y, e, o).into_value(e, o);
      let x = o.emit(Inst::Index(x, y));
      put_value(x, e);
      return What::NumValues(1);
    }
    Expr::Int(n) => {
      let x = o.emit(Inst::ConstInt(n));
      put_value(x, e);
      return What::NumValues(1);
    }
    Expr::Loop(xs) => {
      let i = o.emit_point();
      let a = o.emit_label_and_patch_points([i]);
      put_loop(a, e);
      let m = compile_block(xs, e, o).into_points(e, o);
      patch_points(a, pop_points(m, e), o);
      let n = pop_loop(e);
      return What::NumPoints(n);
    }
    Expr::Op1(&(f, x)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let x = o.emit(Inst::Op1(f, x));
      put_value(x, e);
      return What::NumValues(1);
    }
    Expr::Op2(&(f, x, y)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let y = compile_expr(y, e, o).into_value(e, o);
      let x = o.emit(Inst::Op2(f, x, y));
      put_value(x, e);
      return What::NumValues(1);
    }
    Expr::Or(&(x, y)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let _ = o.emit(Inst::Cond(x));
      let i = o.emit_point();
      let j = o.emit_point();
      let _ = o.emit_label_and_patch_points([i]);
      let n = compile_expr(y, e, o).into_points(e, o);
      let _ = o.emit_label_and_patch_points([j]);
      let x = o.emit(Inst::ConstBool(true));
      let _ = o.emit(Inst::Put(x));
      let k = o.emit_point();
      put_point(k, e);
      return What::NumPoints(n + 1);
    }
    Expr::Ternary(&(x, y, z)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let _ = o.emit(Inst::Cond(x));
      let i = o.emit_point();
      let j = o.emit_point();
      let _ = o.emit_label_and_patch_points([i]);
      let m = compile_expr(z, e, o).into_points(e, o);
      let _ = o.emit_label_and_patch_points([j]);
      let n = compile_expr(y, e, o).into_points(e, o);
      return What::NumPoints(m + n);
    }
    Expr::Undefined => {
      // error, evaluating undefined expression
      let _ = o.emit(Inst::GotoStaticError);
      let _ = o.emit(Inst::Label);
      let x = o.emit(Inst::Pop);
      put_value(x, e);
      return What::NumValues(1);
    }
    Expr::Variable(s) => {
      match get_binding(s, e) {
        None => {
          let x = o.emit(Inst::Global(s));
          put_value(x, e);
        }
        Some(&Binding::Let(x)) => {
          put_value(x, e);
        }
        Some(&Binding::Var(x)) => {
          let x = o.emit(Inst::Local(x));
          put_value(x, e);
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
      let i = o.emit_point();
      let j = o.emit_point();
      let _ = o.emit_label_and_patch_points([i]);
      let x = o.emit(Inst::ConstBool(false));
      let _ = o.emit(Inst::Put(x));
      let _ = o.emit(Inst::Ret);
      let _ = o.emit_label_and_patch_points([j]);
      compile_expr_tail(y, e, o);
    }
    Expr::Call(&(f, xs)) => {
      let n = xs.len();
      let f = compile_expr(f, e, o).into_value(e, o);
      for &x in xs.iter() {
        let x = compile_expr(x, e, o).into_value(e, o);
        put_value(x, e);
      }
      for x in pop_values(n, e) {
        let _ = o.emit(Inst::Put(x));
      }
      let _ = o.emit(Inst::TailCall(f));
    }
    Expr::If(&(x, ys)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let _ = o.emit(Inst::Cond(x));
      let i = o.emit_point();
      let j = o.emit_point();
      let _ = o.emit_label_and_patch_points([i]);
      let _ = o.emit(Inst::Ret);
      let _ = o.emit_label_and_patch_points([j]);
      compile_block_tail(ys, e, o);
    }
    Expr::IfElse(&(x, ys, zs)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let _ = o.emit(Inst::Cond(x));
      let i = o.emit_point();
      let j = o.emit_point();
      let _ = o.emit_label_and_patch_points([i]);
      compile_block_tail(zs, e, o);
      let _ = o.emit_label_and_patch_points([j]);
      compile_block_tail(ys, e, o);
    }
    Expr::Loop(xs) => {
      let i = o.emit_point();
      let a = o.emit_label_and_patch_points([i]);
      put_loop_tail(a, e);
      let m = compile_block(xs, e, o).into_points(e, o);
      patch_points(a, pop_points(m, e), o);
      pop_loop_tail(e);
    }
    Expr::Or(&(x, y)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let _ = o.emit(Inst::Cond(x));
      let i = o.emit_point();
      let j = o.emit_point();
      let _ = o.emit_label_and_patch_points([i]);
      compile_expr_tail(y, e, o);
      let _ = o.emit_label_and_patch_points([j]);
      let x = o.emit(Inst::ConstBool(true));
      let _ = o.emit(Inst::Put(x));
      let _ = o.emit(Inst::Ret);
    }
    Expr::Ternary(&(x, y, z)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let _ = o.emit(Inst::Cond(x));
      let i = o.emit_point();
      let j = o.emit_point();
      let _ = o.emit_label_and_patch_points([i]);
      compile_expr_tail(z, e, o);
      let _ = o.emit_label_and_patch_points([j]);
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

// NB: We don't jump-thread from a statement directly before a no-argument
// break or continue. It's probably not worth doing that.

fn compile_stmt<'a>(x: Stmt<'a>, e: &mut Env, o: &mut Out) -> What {
  match x {
    Stmt::Expr(x) => {
      return compile_expr(x, e, o);
    }
    Stmt::Break(xs) => {
      match e.loops.last_mut() {
        None => {
          // error, break is not inside loop
          let _ = o.emit(Inst::GotoStaticError);
        }
        Some(LoopBreakTarget::Tail) => {
          compile_expr_list_tail(xs, e, o);
        }
        Some(LoopBreakTarget::NonTail(_)) => {
          match xs {
            &[x] => {
              let n = compile_expr(x, e, o).into_points(e, o);
              for _ in 0 .. n {
                let i = pop_point(e);
                put_break_point(i, e);
              }
            }
            xs => {
              let n = xs.len();
              for &x in xs.iter() {
                let x = compile_expr(x, e, o).into_value(e, o);
                put_value(x, e);
              }
              for x in pop_values(n, e) {
                let _ = o.emit(Inst::Put(x));
              }
              let i = o.emit_point();
              put_break_point(i, e);
            }
          }
        }
      }
      return What::NEVER;
    }
    Stmt::Continue => {
      // TODO: if continue_labels.last() is None, generate an error.
      // like for break
      let _ = o.emit(Inst::Goto(*e.continue_labels.last().unwrap()));
      return What::NEVER;
    }
    Stmt::Let(s, x) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      put_let(s, x, e);
      return What::NIL;
    }
    Stmt::Return(xs) => {
      compile_expr_list_tail(xs, e, o);
      return What::NEVER;
    }
    Stmt::Set(s, x) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      match get_binding(s, e) {
        Some(&Binding::Var(y)) => {
          let _ = o.emit(Inst::SetLocal(y, x));
          return What::NIL;
        }
        _ => {
          // error, symbol does not refer to local variable
          let _ = o.emit(Inst::GotoStaticError);
          let _ = o.emit(Inst::Label);
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
      put_var(s, x, e);
      return What::NIL;
    }
  }
}

fn compile_stmt_tail<'a>(x: Stmt<'a>, e: &mut Env, o: &mut Out) {
  match x {
    Stmt::Expr(x) => {
      compile_expr_tail(x, e, o);
    }
    x @ (
      | Stmt::Break(..)
      | Stmt::Continue
      | Stmt::Return(..)
    ) => {
      match compile_stmt(x, e, o) {
        What::NumPoints(0) => {
        }
        _ => unreachable!()
      }
    }
    x @ (
      | Stmt::Let(..)
      | Stmt::Set(..)
      | Stmt::SetField(..)
      | Stmt::SetIndex(..)
      | Stmt::Var(..)
    ) => {
      match compile_stmt(x, e, o) {
        What::NumValues(0) => {
          let _ = o.emit(Inst::Ret);
        }
        _ => unreachable!()
      }
    }
  }
}

fn compile_block<'a>(xs: &'a [Stmt<'a>], e: &mut Env, o: &mut Out) -> What {
  match xs.split_last() {
    None => {
      return What::NIL;
    }
    Some((&y, xs)) => {
      put_scope(e);
      for &x in xs.iter() {
        compile_stmt(x, e, o).into_nil(e, o);
      }
      let w = compile_stmt(y, e, o);
      pop_scope(e);
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
      put_scope(e);
      for &x in xs.iter() {
        compile_stmt(x, e, o).into_nil(e, o);
      }
      compile_stmt_tail(y, e, o);
      pop_scope(e);
    }
  }
}

fn compile_expr_list_tail<'a>(xs: &'a [Expr<'a>], e: &mut Env, o: &mut Out) {
  match xs {
    &[x] => {
      compile_expr_tail(x, e, o);
    }
    xs => {
      let n = xs.len();
      for &x in xs.iter() {
        let x = compile_expr(x, e, o).into_value(e, o);
        put_value(x, e);
      }
      for x in pop_values(n, e) {
        let _ = o.emit(Inst::Put(x));
      }
      let _ = o.emit(Inst::Ret);
    }
  }
}
