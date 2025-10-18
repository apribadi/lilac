use crate::ast::Expr;
use crate::symbol::Symbol;
use crate::uir::Inst;

/*
struct Code(Vec<Inst>);

impl Code {
  fn emit(&mut self, inst: Inst) -> u32 {
    let n = self.0.len();
    self.0.push(inst);
    return n as u32;
  }

  fn edit(&mut self, index: u32, inst: Inst) {
    self.0[index as usize] = inst;
  }
}

struct Env {
  stack: Vec<u32>,
}

impl Env {
  fn put(&mut self, x: u32) {
    self.stack.push(x)
  }

  fn pop(&mut self) -> u32 {
    return self.stack.pop().unwrap();
  }
}

fn compile_expr<'a>(env: &mut Env, code: &mut Code, x: Expr<'a>) {
  }
}
*/

struct Env {
  code: Vec<Inst>,
  args: Vec<u32>,
}

impl Env {
  fn emit(&mut self, inst: Inst) -> u32 {
    let n = self.code.len();
    self.code.push(inst);
    return n as u32;
  }

  fn edit(&mut self, index: u32, inst: Inst) {
    self.code[index as usize] = inst;
  }
}

pub fn compile<'a>(x: Expr<'a>) -> Vec<Inst> {
  let mut env =
    Env {
      code: Vec::new(),
      args: Vec::new(),
    };

  compile_expr_tail(&mut env, x);
  env.code
}

fn compile_expr<'a>(env: &mut Env, x: Expr<'a>) -> u32 {
  match x {
    Expr::And(&(x, y)) => {
      let x = compile_expr(env, x);
      let i = env.emit(Inst::Undefined); // Cond(x, a, b)
      let a = env.emit(Inst::Label);
      let f = env.emit(Inst::ConstBool(false));
      let _ = env.emit(Inst::Put(f));
      let j = env.emit(Inst::Undefined); // Jump(c)
      let b = env.emit(Inst::Label);
      let y = compile_expr(env, y);
      let _ = env.emit(Inst::Put(y));
      let k = env.emit(Inst::Undefined); // Jump(c)
      let c = env.emit(Inst::Label);
      env.edit(i, Inst::Cond(x, a, b));
      env.edit(j, Inst::Jump(c));
      env.edit(k, Inst::Jump(c));
      env.emit(Inst::Pop)
    }
    Expr::Call(&(f, x)) => {
      let n = x.len();
      let f = compile_expr(env, f);
      for &x in x {
        let i = compile_expr(env, x);
        env.args.push(i);
      }
      for x in env.args.drain(env.args.len() - n ..) {
        env.code.push(Inst::Put(x));
      }
      let i = env.emit(Inst::Undefined); // Call(a)
      let a = env.emit(Inst::Label);
      env.edit(i, Inst::Call(f, a));
      env.emit(Inst::Pop)
    }
    Expr::Field(&(symbol, x)) => {
      let x = compile_expr(env, x);
      env.emit(Inst::Field(Symbol::from_bytes(symbol), x))
    }
    Expr::Index(&(x, i)) => {
      let x = compile_expr(env, x);
      let i = compile_expr(env, i);
      env.emit(Inst::Index(x, i))
    }
    Expr::Int(n) => {
      env.emit(Inst::ConstInt(n))
    }
    Expr::Op1(&(op, x)) => {
      let x = compile_expr(env, x);
      env.emit(Inst::Op1(op, x))
    }
    Expr::Op2(&(op, x, y)) => {
      let x = compile_expr(env, x);
      let y = compile_expr(env, y);
      env.emit(Inst::Op2(op, x, y))
    }
    Expr::Or(&(x, y)) => {
      let x = compile_expr(env, x);
      let i = env.emit(Inst::Undefined); // Cond(x, a, b)
      let a = env.emit(Inst::Label);
      let y = compile_expr(env, y);
      let _ = env.emit(Inst::Put(y));
      let j = env.emit(Inst::Undefined); // Jump(c)
      let b = env.emit(Inst::Label);
      let t = env.emit(Inst::ConstBool(true));
      let _ = env.emit(Inst::Put(t));
      let k = env.emit(Inst::Undefined); // Jump(c)
      let c = env.emit(Inst::Label);
      env.edit(i, Inst::Cond(x, a, b));
      env.edit(j, Inst::Jump(c));
      env.edit(k, Inst::Jump(c));
      env.emit(Inst::Pop)
    }
    Expr::Ternary(&(p, x, y)) => {
      let p = compile_expr(env, p);
      let i = env.emit(Inst::Undefined); // Cond(p, a, b)
      let a = env.emit(Inst::Label);
      let x = compile_expr(env, x);
      let _ = env.emit(Inst::Put(x));
      let j = env.emit(Inst::Undefined); // Jump(c)
      let b = env.emit(Inst::Label);
      let y = compile_expr(env, y);
      let _ = env.emit(Inst::Put(y));
      let k = env.emit(Inst::Undefined); // Jump(c)
      let c = env.emit(Inst::Label);
      env.edit(i, Inst::Cond(p, a, b));
      env.edit(j, Inst::Jump(c));
      env.edit(k, Inst::Jump(c));
      env.emit(Inst::Pop)
    }
    Expr::Undefined => {
      env.emit(Inst::Undefined)
    }
    Expr::Variable(symbol) => {
      // TODO: local scope
      env.emit(Inst::Global(Symbol::from_bytes(symbol)))
    }
  }
}

fn compile_expr_tail<'a>(env: &mut Env, x: Expr<'a>) {
  match x {
    Expr::And(&(x, y)) => {
      let x = compile_expr(env, x);
      let i = env.emit(Inst::Undefined); // Cond(x, a, b)
      let a = env.emit(Inst::Label);
      let f = env.emit(Inst::ConstBool(false));
      let _ = env.emit(Inst::Put(f));
      let _ = env.emit(Inst::Ret);
      let b = env.emit(Inst::Label);
      compile_expr_tail(env, y);
      env.edit(i, Inst::Cond(x, a, b));
    }
    Expr::Call(&(f, xs)) => {
      let f = compile_expr(env, f);
      let mut ys = Vec::with_capacity(xs.len());
      for &x in xs.iter() { ys.push(compile_expr(env, x)); }
      for &y in ys.iter() { let _ = env.emit(Inst::Put(y)); }
      let _ = env.emit(Inst::CallTail(f));
    }
    Expr::Or(&(x, y)) => {
      let x = compile_expr(env, x);
      let i = env.emit(Inst::Undefined); // Cond(x, a, b)
      let a = env.emit(Inst::Label);
      compile_expr_tail(env, y);
      let b = env.emit(Inst::Label);
      let t = env.emit(Inst::ConstBool(true));
      let _ = env.emit(Inst::Put(t));
      let _ = env.emit(Inst::Ret);
      env.edit(i, Inst::Cond(x, a, b));
    }
    Expr::Ternary(&(p, x, y)) => {
      let p = compile_expr(env, p);
      let i = env.emit(Inst::Undefined); // Cond(p, a, b)
      let a = env.emit(Inst::Label);
      compile_expr_tail(env, x);
      let b = env.emit(Inst::Label);
      compile_expr_tail(env, y);
      env.edit(i, Inst::Cond(p, a, b));
    }
    x @ (
      | Expr::Field(_)
      | Expr::Index(_)
      | Expr::Int(_)
      | Expr::Op1(_)
      | Expr::Op2(_)
      | Expr::Undefined
      | Expr::Variable(_)
    ) => {
      let x = compile_expr(env, x);
      let _ = env.emit(Inst::Put(x));
      let _ = env.emit(Inst::Ret);
    }
  }
}
