use crate::ast::Expr;
use crate::uir::Inst;

struct Env {
  code: Vec<Inst>,
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
    };

  compile_expr_tail(&mut env, x);
  env.code
}

fn compile_expr_value<'a>(env: &mut Env, x: Expr<'a>) -> u32 {
  match x {
    Expr::And(&(x, y)) => {
      let x = compile_expr_value(env, x);
      let c = env.emit(Inst::Undefined);
      let a = env.emit(Inst::Label);
      let f = env.emit(Inst::ConstBool(false));
      let _ = env.emit(Inst::Put(f));
      let u = env.emit(Inst::Undefined);
      let b = env.emit(Inst::Label);
      let y = compile_expr_value(env, y);
      let _ = env.emit(Inst::Put(y));
      let v = env.emit(Inst::Undefined);
      let j = env.emit(Inst::Label);
      env.edit(c, Inst::Cond(x, a, b));
      env.edit(u, Inst::Jump(j));
      env.edit(v, Inst::Jump(j));
      env.emit(Inst::Pop)
    }
    Expr::Index(&(x, i)) => {
      let x = compile_expr_value(env, x);
      let i = compile_expr_value(env, i);
      env.emit(Inst::Index(x, i))
    }
    Expr::Integer(n) => {
      env.emit(Inst::Integer(n))
    }
    Expr::Op1(&(op, x)) => {
      let x = compile_expr_value(env, x);
      env.emit(Inst::Op1(op, x))
    }
    Expr::Op2(&(op, x, y)) => {
      let x = compile_expr_value(env, x);
      let y = compile_expr_value(env, y);
      env.emit(Inst::Op2(op, x, y))
    }
    _ => {
      unimplemented!()
    }
  }
}

fn compile_expr_tail<'a>(env: &mut Env, x: Expr<'a>) {
  match x {
    Expr::And(&(x, y)) => {
      let x = compile_expr_value(env, x);
      let c = env.emit(Inst::Undefined);
      let a = env.emit(Inst::Label);
      let f = env.emit(Inst::ConstBool(false));
      let _ = env.emit(Inst::Put(f));
      let _ = env.emit(Inst::Ret);
      let b = env.emit(Inst::Label);
      let y = compile_expr_value(env, y);
      let _ = env.emit(Inst::Put(y));
      let _ = env.emit(Inst::Ret);
      env.edit(c, Inst::Cond(x, a, b));
    }
    Expr::Op1(&(op, x)) => {
      let x = compile_expr_value(env, x);
      let y = env.emit(Inst::Op1(op, x));
      let _ = env.emit(Inst::Put(y));
      let _ = env.emit(Inst::Ret);
    }
    Expr::Op2(&(op, x, y)) => {
      let x = compile_expr_value(env, x);
      let y = compile_expr_value(env, y);
      let z = env.emit(Inst::Op2(op, x, y));
      let _ = env.emit(Inst::Put(z));
      let _ = env.emit(Inst::Ret);
    }
    _ => {
      unimplemented!()
    }
  }
}
