use crate::ast::Expr;
use crate::ast::Stmt;
use crate::symbol::Symbol;
use crate::symbol_table::SymbolTable;
use crate::uir::Inst;

#[derive(Clone, Copy)]
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

fn put_binding(s: Symbol, x: Binding, e: &mut Env) {
  e.symbol_table.insert(s, x);
}

fn get_binding(s: Symbol, e: &Env) -> Option<Binding> {
  return e.symbol_table.get(s).map(|x| *x);
}

// TODO: put_loop_tail, pop_loop_tail

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

#[allow(dead_code)]
fn pop_point(e: &mut Env) -> u32 {
  return e.points.pop().unwrap();
}

fn pop_points(n: usize, e: &mut Env) -> impl Iterator<Item = u32> {
  return e.points.drain(e.points.len() - n ..);
}

/*
fn put_break_scope(e: &mut Env) {
  e.breaks.push(0);
}

fn pop_break_scope(e: &mut Env) -> impl Iterator<Item = u32> {
  let n = e.breaks.pop().unwrap();
  return e.points.drain(e.points.len() - n ..);
}

fn put_break(a: u32, e: &mut Env) {
  e.breaks.last_mut().unwrap() += 1;
  e.points.push(a);
}
*/

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
        let a = o.emit(Inst::Label);
        for k in pop_points(n, e) {
          o.edit_patch_point(k, a);
        }
      }
      What::NumValues(0) => {
      }
      What::NumValues(n) => {
        let _ = pop_values(n, e);
        let _ = o.emit(Inst::AbortStaticError); // arity mismatch
        let _ = o.emit(Inst::Label);
      }
    }
  }

  fn into_value(self, e: &mut Env, o: &mut Out) -> u32 {
    match self {
      What::NumPoints(n) => {
        let a = o.emit(Inst::Label);
        let x = o.emit(Inst::Pop);
        for k in pop_points(n, e) {
          o.edit_patch_point(k, a);
        }
        return x;
      }
      What::NumValues(1) => {
        return pop_value(e);
      }
      What::NumValues(n) => {
        let _ = pop_values(n, e);
        let _ = o.emit(Inst::AbortStaticError); // arity mismatch
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
        let i = o.emit_patch_point();
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
      let i = o.emit_patch_point();
      let j = o.emit_patch_point();
      let a = o.emit(Inst::Label);
      let x = o.emit(Inst::ConstBool(false));
      let _ = o.emit(Inst::Put(x));
      let k = o.emit_patch_point();
      put_point(k, e);
      let b = o.emit(Inst::Label);
      let n = compile_expr(y, e, o).into_points(e, o);
      o.edit_patch_point(i, a);
      o.edit_patch_point(j, b);
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
      let i = o.emit_patch_point();
      put_point(i, e);
      return What::NumPoints(1);
    }
    Expr::Field(&(x, s)) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      let x = o.emit(Inst::Field(x, s));
      put_value(x, e);
      return What::NumValues(1);
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
      let i = o.emit_patch_point();
      let a = o.emit(Inst::Label);
      o.edit_patch_point(i, a);
      put_loop(a, e);
      compile_block(xs, e, o).into_nil(e, o);
      let _ = o.emit(Inst::Jump(a));
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
      let i = o.emit_patch_point();
      let j = o.emit_patch_point();
      let a = o.emit(Inst::Label);
      let n = compile_expr(y, e, o).into_points(e, o);
      let b = o.emit(Inst::Label);
      let x = o.emit(Inst::ConstBool(true));
      let _ = o.emit(Inst::Put(x));
      let k = o.emit_patch_point();
      put_point(k, e);
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
      let _ = o.emit(Inst::AbortStaticError);
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
        Some(Binding::Let(x)) => {
          put_value(x, e);
        }
        Some(Binding::Var(x)) => {
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
      | Expr::Bool(_)
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

fn compile_stmt<'a>(x: Stmt<'a>, e: &mut Env, o: &mut Out) -> What {
  match x {
    Stmt::Expr(x) => {
      return compile_expr(x, e, o);
    }
    Stmt::Break(xs) => {
      match e.loops.last_mut() {
        None => {
          // error, break not inside loop
          unimplemented!()
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
              let i = o.emit_patch_point();
              put_break_point(i, e);
            }
          }
        }
      }
      return What::NEVER;
    }
    Stmt::Continue => {
      let _ = o.emit(Inst::Jump(*e.continue_labels.last().unwrap()));
      return What::NEVER;
    }
    Stmt::Let(s, x) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      put_binding(s, Binding::Let(x), e);
      return What::NIL;
    }
    Stmt::Return(xs) => {
      compile_expr_list_tail(xs, e, o);
      return What::NEVER;
    }
    Stmt::Set(s, x) => {
      let x = compile_expr(x, e, o).into_value(e, o);
      match get_binding(s, e) {
        Some(Binding::Var(y)) => {
          let _ = o.emit(Inst::SetLocal(y, x));
          return What::NIL;
        }
        _ => {
          let _ = o.emit(Inst::AbortStaticError); // no variable
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
      put_binding(s, Binding::Var(x), e);
      return What::NIL;
    }
  }
}

fn compile_block<'a>(xs: &'a [Stmt<'a>], e: &mut Env, o: &mut Out) -> What {
  put_scope(e);
  let w =
    match xs.split_last() {
      None => {
        What::NIL
      }
      Some((&y, xs)) => {
        for &x in xs.iter() {
          compile_stmt(x, e, o).into_nil(e, o);
        }
        compile_stmt(y, e, o)
      }
    };
  pop_scope(e);
  return w;
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
