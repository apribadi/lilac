use crate::prelude::*;
use crate::ssa;
use crate::mir;
use crate::mir::Symbol;

pub struct Env<'a, 'b> {
  arena: Arena<'b>,
  out: ssa::Builder,
  symbol_table: Vec<(Symbol<'a>, Referent)>,
  symbol_count: Vec<usize>,
}

#[derive(Clone, Copy)]
pub enum Referent {
  Value(ssa::Value, ssa::Type),
  Variable(ssa::Variable, ssa::Type),
}

impl<'a, 'b> Env<'a, 'b> {
  pub fn new(arena: Arena<'b>) -> Self {
    Self {
      arena,
      out: ssa::Builder::new(),
      symbol_table: Vec::new(),
      symbol_count: vec![0],
    }
  }

  pub fn push_scope(&mut self) {
    self.symbol_count.push(0);
  }

  pub fn pop_scope(&mut self) {
    let c = self.symbol_count.pop().unwrap();

    for _ in 0 .. c {
      let _: Option<_> = self.symbol_table.pop();
    }
  }

  pub fn bind_value(&mut self, symbol: Symbol<'a>, value: ssa::Value, t: ssa::Type) {
    self.symbol_table.push((symbol, Referent::Value(value, t)));
    let r = self.symbol_count.last_mut().unwrap();
    *r = *r + 1;
  }

  pub fn lookup(&self, symbol: Symbol<'a>) -> Option<Referent> {
    for &(s, x) in self.symbol_table.iter().rev() {
      if symbol == s {
        return Some(x);
      }
    }

    return None;
  }
}

pub fn compile_func(func: &mir::Func<'_>) {
  let mut store = oxcart::Store::new();
  let mut env = Env::new(store.arena());

  env.push_scope();

  env.out.emit_func(1, func.params.len() as u32);

  for &(s, t) in func.params.iter() {
    match t {
      mir::Type::I64 => {
        let x = env.out.emit_param(ssa::Type::I64);
        env.bind_value(s, x, ssa::Type::I64);
      }
    }
  }

  match compile_expr(&mut env, func.body) {
    None => {}
    Some((value, _)) => {
      env.out.emit_return(0, 1);
      env.out.emit_value(value);
    }
  }

  env.pop_scope();

  ssa::display(env.out.view());
}


// For now, an expression either evaluates to a single typed ssa value, or
// doesn't return to its continuation at all.
//
// Later we will want to extend this to
// - aggregate types
// - zero or multiple return values
// - two or more continuations

pub fn compile_expr<'a, 'b>(env: &mut Env<'a, 'b>, expr: mir::Expr<'a>) -> Option<(ssa::Value, ssa::Type)> {
  match expr {
    mir::Expr::Symbol(s) => {
      match env.lookup(s).unwrap() {
        Referent::Value(x, t) => {
          Some((x, t))
        }
        Referent::Variable(x, t) => {
          Some((env.out.emit_get_variable(x), t))
        }
      }
    }

    mir::Expr::ConstBool(p) => {
      Some((env.out.emit_const_bool(p), ssa::Type::BOOL))
    }

    mir::Expr::ConstI32(n) => {
      Some((env.out.emit_const_i32(n), ssa::Type::I32))
    }

    mir::Expr::ConstI64(n) => {
      Some((env.out.emit_const_i64(n), ssa::Type::I64))
    }

    mir::Expr::Call(&mir::Call { func: Symbol(b"add.i64"), args: &[x, y] }) => {
      let (x, t) = compile_expr(env, x)?;
      assert!(t == ssa::Type::I64);
      let (y, t) = compile_expr(env, y)?;
      assert!(t == ssa::Type::I64);
      Some((env.out.emit_op2(ssa::Op2::ADD_I64, x, y), ssa::Type::I64))
    }

    mir::Expr::If(&mir::If { condition, if_true, if_false }) => {
      let (p, t) = compile_expr(env, condition)?;
      assert!(t == ssa::Type::BOOL);
      let (a, b) = env.out.emit_if(p, ssa::Label(0), ssa::Label(0));

      let case0 = 'arm: {
        let label = env.out.emit_case();
        env.out.patch_label(b, label);
        let Some((x, t)) = compile_expr(env, if_false) else { break 'arm None; };
        let point = env.out.emit_goto(ssa::Label(0), 1);
        env.out.emit_value(x);
        Some((t, point))
      };

      let case1 = 'arm: {
        let label = env.out.emit_case();
        env.out.patch_label(a, label);
        let Some((x, t)) = compile_expr(env, if_true) else { break 'arm None; };
        let point = env.out.emit_goto(ssa::Label(0), 1);
        env.out.emit_value(x);
        Some((t, point))
      };

      match [case0, case1] {
        [None, None] => {
          None
        }
        [Some((t, point)), None] | [None, Some((t, point))] => {
          let label = env.out.emit_join(1);
          env.out.patch_label(point, label);
          Some((env.out.emit_param(t), t))
        }
        [Some((t0, point0)), Some((t1, point1))] => {
          assert!(t0 == t1);
          let label = env.out.emit_join(1);
          env.out.patch_label(point0, label);
          env.out.patch_label(point1, label);
          Some((env.out.emit_param(t0), t0))
        }
      }
    }

    mir::Expr::Do(stmts) => {
      env.push_scope();

      let result = 'do_result: {
        let (&last, rest) = stmts.split_last().unwrap();

        for &stmt in rest.iter() {
          match compile_stmt(env, stmt) {
            None => {
              break 'do_result None;
            }
            Some(&[]) => {
            }
            _ => {
              panic!()
            }
          }
        }

        match compile_stmt(env, last) {
          None => {
            break 'do_result None;
          }
          Some(&[&[value]]) => {
            Some(value)
          }
          _ => {
            panic!()
          }
        }
      };

      env.pop_scope();

      result
    }

    _ => {
      panic!()
    }
  }
}

// - outer option: do you return
// - inner option: when you return, do you return one value or zero values?

pub fn compile_stmt<'a, 'b>(
    env: &mut Env<'a, 'b>,
    stmt: mir::Stmt<'a>
  ) -> Option<&'b [&'b [(ssa::Value, ssa::Type)]]>
{
  match stmt {
    mir::Stmt::Expr(expr) => {
      match compile_expr(env, expr) {
        None => {
          return None;
        }
        Some(value) => {
          let value = core::slice::from_ref(env.arena.alloc().init(value));
          let value = core::slice::from_ref(env.arena.alloc().init(value));
          return Some(value);
        }
      }
    }
    mir::Stmt::Let(name, init) => {
      let (x, t) = compile_expr(env, init)?;
      env.bind_value(name, x, t);
      Some(&[])
    }
    _ => {
      panic!()
    }
  }
}
