use crate::prelude::*;
use crate::ssa;
use crate::mir;
use crate::mir::Symbol;

pub struct Env<'a, 'b> {
  arena: Arena<'b>,
  out: ssa::Builder,
  symbol_table: Vec<(Symbol<'a>, Referent<'b>)>,
  symbol_count: Vec<usize>,
}

#[derive(Clone, Copy)]
pub enum Referent<'a> {
  Value(&'a [(ssa::Value, ssa::Type)]),
  Local(&'a [(ssa::Local, ssa::Type)]),
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
    let n = self.symbol_table.len() - self.symbol_count.pop().unwrap();
    self.symbol_table.truncate(n);
  }

  pub fn bind_value(&mut self, symbol: Symbol<'a>, value: &'b [(ssa::Value, ssa::Type)]) {
    self.symbol_table.push((symbol, Referent::Value(value)));
    *self.symbol_count.last_mut().unwrap() += 1;
  }

  pub fn bind_local(&mut self, symbol: Symbol<'a>, local: &'b [(ssa::Local, ssa::Type)]) {
    self.symbol_table.push((symbol, Referent::Local(local)));
    *self.symbol_count.last_mut().unwrap() += 1;
  }

  pub fn lookup(&self, symbol: Symbol<'a>) -> Option<Referent<'b>> {
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
        let x = env.arena.alloc().init([(x, ssa::Type::I64)]);
        env.bind_value(s, x)
      }
    }
  }

  match compile_expr(&mut env, func.body) {
    None => {}
    Some(&[&[(value, _)]]) => {
      env.out.emit_return(0, 1);
      env.out.emit_value(value);
    }
    _ => {
      panic!()
    }
  }

  env.pop_scope();

  ssa::display(env.out.view());
}


// Currently, an expression evaluates into
//
// - never, i.e. it never returns
//
// - a sequence of values, each of which is a sequence of primitive values

pub fn compile_expr<'a, 'b>(
    env: &mut Env<'a, 'b>,
    expr: mir::Expr<'a>
  ) -> Option<&'b [&'b [(ssa::Value, ssa::Type)]]>
{
  match expr {
    mir::Expr::Symbol(s) => {
      match env.lookup(s).unwrap() {
        Referent::Value(x) => {
          Some(env.arena.alloc().init([x]) as &[_])
        }
        Referent::Local(x) => {
          Some(env.arena.alloc().init([
            env.arena.alloc_slice(x.len()).init_slice(|i| {
              let (y, t) = x[i];
              (env.out.emit_get_local(y), t)
            }) as &[_]
          ]) as &[_])
        }
      }
    }

    mir::Expr::ConstBool(p) => {
      Some(env.arena.alloc().init([
        env.arena.alloc().init([
          (env.out.emit_const_bool(p), ssa::Type::BOOL)
        ]) as &[_]
      ]) as &[_])
    }

    mir::Expr::ConstI64(n) => {
      Some(env.arena.alloc().init([
        env.arena.alloc().init([
          (env.out.emit_const_i64(n), ssa::Type::I64)
        ]) as &[_]
      ]) as &[_])
    }

    mir::Expr::Call(&mir::Call(Symbol(b"add.i64"), &[x, y])) => {
      let &[&[(x, ssa::Type::I64)]] = compile_expr(env, x)? else { panic!() };
      let &[&[(y, ssa::Type::I64)]] = compile_expr(env, y)? else { panic!() };
      Some(env.arena.alloc().init([
        env.arena.alloc().init([
          (env.out.emit_op2(ssa::Op2::ADD_I64, x, y), ssa::Type::I64)
        ]) as &[_]
      ]) as &[_])
    }

    mir::Expr::Call(&mir::Call(Symbol(b"sub.i64"), &[x, y])) => {
      let &[&[(x, ssa::Type::I64)]] = compile_expr(env, x)? else { panic!() };
      let &[&[(y, ssa::Type::I64)]] = compile_expr(env, y)? else { panic!() };
      Some(env.arena.alloc().init([
        env.arena.alloc().init([
          (env.out.emit_op2(ssa::Op2::SUB_I64, x, y), ssa::Type::I64)
        ]) as &[_]
      ]) as &[_])
    }

    mir::Expr::Call(&mir::Call(Symbol(b"is_ne.i64"), &[x, y])) => {
      let &[&[(x, ssa::Type::I64)]] = compile_expr(env, x)? else { panic!() };
      let &[&[(y, ssa::Type::I64)]] = compile_expr(env, y)? else { panic!() };
      Some(env.arena.alloc().init([
        env.arena.alloc().init([
          (env.out.emit_op2(ssa::Op2::IS_NE_I64, x, y), ssa::Type::BOOL)
        ]) as &[_]
      ]) as &[_])
    }

    mir::Expr::If(&mir::If { condition, if_true, if_false }) => {
      let &[&[(p, ssa::Type::BOOL)]] = compile_expr(env, condition)? else { panic!() };

      let (patch_arm1, patch_arm0) = env.out.emit_if(p, ssa::Label(0), ssa::Label(0));

      let arms: [Option<(_, &[&[_]], _)>; 2] =
        [(patch_arm0, if_false), (patch_arm1, if_true)].map(|(patch_arm, arm)| {
          let label = env.out.emit_case();
          env.out.patch_label(patch_arm, label);
          match compile_expr(env, arm) {
            None => None,
            Some(xs) => {
              let arity = xs.iter().map(|y| y.len() as u32).sum();
              let patch_join = env.out.emit_goto(ssa::Label(0), arity);
              let ts =
                env.arena.alloc_slice(xs.len()).init_slice(|i| {
                  let xs = xs[i];
                  env.arena.alloc_slice(xs.len()).init_slice(|j| {
                    let (x, t) = xs[j];
                    env.out.emit_value(x);
                    t
                  }) as &[_]
                }) as &[_];
              Some((arity, ts, patch_join))
            }
          }
        });

      match arms {
        [None, None] => {
          None
        }
        [Some((arity, ts, patch_join)), None] | [None, Some((arity, ts, patch_join))] => {
          let label = env.out.emit_join(arity);
          env.out.patch_label(patch_join, label);
          Some(env.arena.alloc_slice(ts.len()).init_slice(|i| {
            let ts = ts[i];
            env.arena.alloc_slice(ts.len()).init_slice(|j| {
              let t = ts[j];
              (env.out.emit_param(t), t)
            }) as &[_]
          }) as &[_])
        }
        [Some((arity, ts, patch_join0)), Some((arity1, ts1, patch_join1))] => {
          assert!(arity == arity1);
          assert!(ts == ts1);
          let label = env.out.emit_join(1);
          env.out.patch_label(patch_join0, label);
          env.out.patch_label(patch_join1, label);
          Some(env.arena.alloc_slice(ts.len()).init_slice(|i| {
            let ts = ts[i];
            env.arena.alloc_slice(ts.len()).init_slice(|j| {
              let t = ts[j];
              (env.out.emit_param(t), t)
            }) as &[_]
          }) as &[_])
        }
      }
    }

    mir::Expr::Do(stmts) => {
      env.push_scope();
      let result: Option<&[&[_]]> = 'do_result: {
        match stmts.split_last() {
          None => Some(&[]),
          Some((&last, rest)) => {
            for &stmt in rest.iter() {
              match compile_stmt(env, stmt) {
                None => break 'do_result None,
                Some(&[_, ..]) => panic!(),
                Some(&[]) => { }
              }
            }
            compile_stmt(env, last)
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

pub fn compile_stmt<'a, 'b>(
    env: &mut Env<'a, 'b>,
    stmt: mir::Stmt<'a>
  ) -> Option<&'b [&'b [(ssa::Value, ssa::Type)]]>
{
  match stmt {
    mir::Stmt::Expr(expr) => {
      compile_expr(env, expr)
    }
    mir::Stmt::Let(names, init) => {
      let xs = compile_expr(env, init)?;
      for (&name, &x) in zip(names.iter(), xs.iter()) {
        env.bind_value(name, x);
      }
      Some(&[])
    }
    mir::Stmt::LetLocal(name, init) => {
      let &[xs] = compile_expr(env, init)? else { panic!() };
      let xs =
        env.arena.alloc_slice(xs.len()).init_slice(|i| {
          let (x, t) = xs[i];
          (env.out.emit_let_local(x), t)
        }) as &[_];
      env.bind_local(name, xs);
      Some(&[])
    }
    mir::Stmt::SetLocal(name, value) => {
      let &[xs] = compile_expr(env, value)? else { panic!() };
      let Some(Referent::Local(vs)) = env.lookup(name) else { panic!() };
      for (&(v, _), &(x, _)) in zip(vs.iter(), xs.iter()) {
        env.out.emit_set_local(v, x);
      }
      Some(&[])
    }
    mir::Stmt::Return(xs) => {
      let mut ys = Vec::new();
      for &e in xs.iter() {
        let xs = compile_expr(env, e)?;
        for &xs in xs.iter() {
          for &(x, _) in xs.iter() {
            ys.push(x)
          }
        }
      }
      env.out.emit_return(0, ys.len() as u32);
      for &y in ys.iter() {
        env.out.emit_value(y);
      }
      None
    }
    mir::Stmt::While(cond, body) => {
      let join_patch = env.out.emit_goto(ssa::Label(0), 0);
      let join_label = env.out.emit_join(0);
      env.out.patch_label(join_patch, join_label);
      let &[&[(cond, ssa::Type::BOOL)]] = compile_expr(env, cond)? else { panic!() };
      let (body_patch, rest_patch) = env.out.emit_if(cond, ssa::Label(0), ssa::Label(0));
      let body_label = env.out.emit_case();
      env.out.patch_label(body_patch, body_label);
      match compile_expr(env, body) {
        None => {}
        Some(&[]) => { let _ = env.out.emit_goto(join_label, 0); }
        Some(&[_, ..]) => { panic!() }
      };
      let rest_label = env.out.emit_case();
      env.out.patch_label(rest_patch, rest_label);
      Some(&[])
    }
    /*
    _ => {
      panic!()
    }
    */
  }
}
