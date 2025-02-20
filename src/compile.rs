use crate::prelude::*;
use crate::ssa;
use crate::mir;
use crate::mir::Exp;
use crate::mir::Symbol;
use crate::ssa::Label;
use crate::ssa::Value;

pub struct Env<'a> {
  //arena: Arena<'a>,
  out: ssa::Builder,
  symbol_table: Vec<(Symbol<'a>, Referent)>,
  symbol_count: Vec<usize>,
}

#[derive(Clone, Copy)]
pub enum Referent {
  Value(ssa::Value, ssa::Type),
  Variable(ssa::Variable, ssa::Type),
}

impl<'a> Env<'a> {
  pub fn new() -> Self {
    Self {
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
      self.symbol_table.pop();
    }
  }

  pub fn bind_value(&mut self, symbol: Symbol<'a>, value: ssa::Value, t: ssa::Type) {
    self.symbol_table.push((symbol, Referent::Value(value, t)));
    let r = self.symbol_count.last_mut().unwrap();
    *r = *r + 1;
  }

  pub fn lookup(&self, symbol: Symbol) -> Option<Referent> {
    for &(s, x) in self.symbol_table.iter().rev() {
      if symbol == s {
        return Some(x);
      }
    }

    return None;
  }
}

pub fn compile(fun: &mir::Function<'_>) {
  let mut env = Env::new();


  env.out.emit_function(1, fun.params.len() as u32);

  env.push_scope();

  for &(s, t) in fun.params.iter() {
    match t {
      mir::Type::I64 => {
        let x = env.out.emit_param(ssa::Type::I64);
        env.bind_value(s, x, ssa::Type::I64);
      }
    }
  }

  match compile_expression(&mut env, fun.body) {
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

pub fn compile_expression<'a>(env: &mut Env, exp: Exp<'a>) -> Option<(ssa::Value, ssa::Type)> {
  match exp {
    Exp::Symbol(s) => {
      match env.lookup(s).unwrap() {
        Referent::Value(x, t) => {
          Some((x, t))
        }
        Referent::Variable(x, t) => {
          Some((env.out.emit_get_variable(x), t))
        }
      }
    }

    Exp::ConstBool(p) => {
      Some((env.out.emit_const_bool(p), ssa::Type::BOOL))
    }

    Exp::ConstI32(n) => {
      Some((env.out.emit_const_i32(n), ssa::Type::I32))
    }

    Exp::ConstI64(n) => {
      Some((env.out.emit_const_i64(n), ssa::Type::I64))
    }

    Exp::Call(&mir::Call { function: Symbol(b"add.i64"), args: &[x, y] }) => {
      let (x, t) = compile_expression(env, x)?;
      assert!(t == ssa::Type::I64);
      let (y, t) = compile_expression(env, y)?;
      assert!(t == ssa::Type::I64);
      Some((env.out.emit_op2(ssa::Op2::ADD_I64, x, y), ssa::Type::I64))
    }

    Exp::If(&mir::If { condition, if_true, if_false }) => {
      let (p, t) = compile_expression(env, condition)?;
      assert!(t == ssa::Type::BOOL);
      let (a, b) = env.out.emit_if(p, Label(0), Label(0));

      let case0 = 'arm: {
        let label = env.out.emit_case();
        env.out.patch_label(b, label);
        let Some((x, t)) = compile_expression(env, if_false) else { break 'arm None; };
        let point = env.out.emit_goto(Label(0), 1);
        env.out.emit_value(x);
        Some((t, point))
      };

      let case1 = 'arm: {
        let label = env.out.emit_case();
        env.out.patch_label(a, label);
        let Some((x, t)) = compile_expression(env, if_true) else { break 'arm None; };
        let point = env.out.emit_goto(Label(0), 1);
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

    Exp::Do(statements) => {
      env.push_scope();
      for &stmt in statements.iter() {
      }
      env.pop_scope();
      panic!()
    }

    _ => {
      panic!()
    }
  }
}

pub fn compile_statement<'a>(
    env: &mut Env,
    stmt: mir::Statement<'a>
  ) -> Option<(ssa::Value, ssa::Type)>
{
  None
}
