use crate::ast::Expr;
use crate::ast::Item;
use crate::ast::Stmt;
use crate::hir::Inst;
use crate::symbol::Symbol;
use crate::symbol_table::SymbolTable;

#[derive(Clone, Copy)]
enum What {
  NumPoints(usize),
  NumValues(usize),
}

#[derive(Clone, Copy)]
enum Referent {
  Let(u32),
  Var(u32),
}

#[derive(Clone, Copy)]
enum LoopBreakTarget {
  Tail,
  NonTail(usize),
}

#[derive(Clone, Copy)]
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
  symbol_table: SymbolTable<Referent>,
  loops: Vec<LoopBreakTarget>,
  break_points: Vec<Point>,
  continue_labels: Vec<Label>,
  values: Vec<u32>,
  points: Vec<Point>,
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
  e.symbol_table.insert(s, Referent::Let(x));
}

fn put_var(s: Symbol, x: u32, e: &mut Env) {
  e.symbol_table.insert(s, Referent::Var(x));
}

fn get_referent(s: Symbol, e: &Env) -> Option<&Referent> {
  return e.symbol_table.get(s);
}

fn put_loop(a: Label, e: &mut Env) {
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

fn put_loop_tail(a: Label, e: &mut Env) {
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

fn pop_value_list(n: usize, e: &mut Env) -> impl Iterator<Item = u32> {
  return e.values.drain(e.values.len() - n ..);
}

fn rev_values(n: usize, e: &mut Env) {
  let k = e.values.len() - n;
  e.values[k ..].reverse();
}

fn put_point(i: Point, e: &mut Env) {
  e.points.push(i);
}

fn put_break_point(i: Point, e: &mut Env) {
  e.break_points.push(i);
  match e.loops.last_mut() {
    Some(LoopBreakTarget::NonTail(n)) => {
      *n += 1;
    }
    _ => unreachable!() // ???
  }
}

fn pop_point(e: &mut Env) -> Point {
  return e.points.pop().unwrap();
}

fn pop_point_list(n: usize, e: &mut Env) -> impl Iterator<Item = Point> {
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

pub fn compile<'a>(item_list: impl Iterator<Item = Item<'a>>) -> Vec<Inst> {
  let mut e = Env::new();
  let mut o = Out::new();

  for Item::Fundef(f) in item_list {
    let _ = o.emit(Inst::Entry(f.args.len() as u32));

    for x in f.args {
      let y = o.emit(Inst::Pop);
      if let Some(x) = x.name {
        put_let(x, y, &mut e);
      }
    }

    compile_block_tail(f.body, &mut e, &mut o);
  }

  return o.0;
}

impl What {
  const NEVER: Self = What::NumPoints(0);

  const NIL: Self = What::NumValues(0);

  fn into_nil(self, e: &mut Env, o: &mut Out) {
    match self {
      What::NumPoints(n) => {
        let _ = o.emit_label_and_patch_point_list(0, pop_point_list(n, e));
      }
      What::NumValues(n) => {
        if n != 0 {
          // error, arity mismatch
          let _ = pop_value_list(n, e);
          let _ = o.emit(Inst::GotoStaticError);
          let _ = o.emit(Inst::Label(0));
        }
      }
    }
  }

  fn into_value(self, e: &mut Env, o: &mut Out) -> u32 {
    match self {
      What::NumPoints(n) => {
        let _ = o.emit_label_and_patch_point_list(1, pop_point_list(n, e));
        let x = o.emit(Inst::Pop);
        return x;
      }
      What::NumValues(n) => {
        if n == 1 {
          return pop_value(e);
        } else {
          // error, arity mismatch
          let _ = pop_value_list(n, e);
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
        let _ = o.emit_label_and_patch_point_list(arity, pop_point_list(n, e));
        for _ in 0 .. arity {
          let x = o.emit(Inst::Pop);
          put_value(x, e);
        }
      }
      What::NumValues(n) => {
        if arity != n {
          // error, arity mismatch
          let _ = pop_value_list(n, e);
          let _ = o.emit(Inst::GotoStaticError);
          let _ = o.emit(Inst::Label(arity as u32));
          for _ in 0 .. arity {
            let x = o.emit(Inst::Pop);
            put_value(x, e);
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
        for x in pop_value_list(n, e) {
          let _ = o.emit(Inst::Put(x));
        }
        let i = o.emit_point(Some(n));
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
      let i = o.emit_point(Some(0));
      let j = o.emit_point(Some(0));
      let _ = o.emit_label_and_patch_point_list(0, [i]);
      let x = o.emit(Inst::ConstBool(false));
      let _ = o.emit(Inst::Put(x));
      let k = o.emit_point(Some(1));
      put_point(k, e);
      let _ = o.emit_label_and_patch_point_list(0, [j]);
      let n = compile_expr(y, e, o).into_point_list(e, o);
      return What::NumPoints(1 + n);
    }
    Expr::Bool(x) => {
      let x = o.emit(Inst::ConstBool(x));
      put_value(x, e);
      return What::NumValues(1);
    }
    Expr::Call(&(f, xs)) => {
      let f = compile_expr(f, e, o).into_value(e, o);
      for &x in xs.iter() {
        let x = compile_expr(x, e, o).into_value(e, o);
        put_value(x, e);
      }
      for x in pop_value_list(xs.len(), e) {
        let _ = o.emit(Inst::Put(x));
      }
      let _ = o.emit(Inst::Call(f));
      let i = o.emit_point(None);
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
      let i = o.emit_point(Some(0));
      let j = o.emit_point(Some(0));
      put_point(i, e);
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
      put_value(x, e);
      return What::NumValues(1);
    }
    Expr::Int(n) => {
      let x = o.emit(Inst::ConstInt(n));
      put_value(x, e);
      return What::NumValues(1);
    }
    Expr::Loop(xs) => {
      let i = o.emit_point(Some(0));
      let a = o.emit_label_and_patch_point_list(0, [i]);
      put_loop(a, e);
      let m = compile_block(xs, e, o).into_point_list(e, o);
      patch_point_list(a, pop_point_list(m, e), o);
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
      let i = o.emit_point(Some(0));
      let j = o.emit_point(Some(0));
      let _ = o.emit_label_and_patch_point_list(0, [i]);
      let n = compile_expr(y, e, o).into_point_list(e, o);
      let _ = o.emit_label_and_patch_point_list(0, [j]);
      let x = o.emit(Inst::ConstBool(true));
      let _ = o.emit(Inst::Put(x));
      let k = o.emit_point(Some(1));
      put_point(k, e);
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
      put_value(x, e);
      return What::NumValues(1);
    }
    Expr::Variable(s) => {
      match get_referent(s, e) {
        None => {
          let x = o.emit(Inst::Const(s));
          put_value(x, e);
        }
        Some(&Referent::Let(x)) => {
          put_value(x, e);
        }
        Some(&Referent::Var(x)) => {
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
      for &x in xs.iter() {
        let x = compile_expr(x, e, o).into_value(e, o);
        put_value(x, e);
      }
      for x in pop_value_list(xs.len(), e) {
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
      let m = compile_block(xs, e, o).into_point_list(e, o);
      patch_point_list(a, pop_point_list(m, e), o);
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
    Stmt::ExprList(xs) => {
      return compile_expr_list(xs, e, o);
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
          let n = compile_expr_list(xs, e, o).into_point_list(e, o);
          for _ in 0 .. n {
            let i = pop_point(e);
            put_break_point(i, e);
          }
        }
      }
      return What::NEVER;
    }
    Stmt::Continue => {
      match e.continue_labels.last() {
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
      // TODO: we do the bindings from left to right, so later bindings shadow
      // earlier ones. we should just produce an error in that case
      compile_expr_list(ys, e, o).into_value_list(xs.len(), e, o);
      rev_values(xs.len(), e);
      for &x in xs.iter() {
        let y = pop_value(e);
        if let Some(x) = x.name {
          put_let(x, y, e);
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
      match get_referent(s, e) {
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
      put_var(s, x, e);
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

fn compile_expr_list<'a>(xs: &'a [Expr<'a>], e: &mut Env, o: &mut Out) -> What {
  match xs {
    &[x] => {
      return compile_expr(x, e, o);
    }
    xs => {
      for &x in xs.iter() {
        let x = compile_expr(x, e, o).into_value(e, o);
        put_value(x, e);
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
      for &x in xs.iter() {
        let x = compile_expr(x, e, o).into_value(e, o);
        put_value(x, e);
      }
      for x in pop_value_list(xs.len(), e) {
        let _ = o.emit(Inst::Put(x));
      }
      let _ = o.emit(Inst::Ret);
    }
  }
}
