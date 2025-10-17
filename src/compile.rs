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
}

pub fn compile<'a>(x: Expr<'a>) -> Vec<Inst> {
  let mut env =
    Env {
      code: Vec::new(),
    };

  compile_expr_tail(&mut env, x);
  env.code
}

fn compile_expr_one<'a>(env: &mut Env, x: Expr<'a>) -> u32 {
  match x {
    Expr::Integer(n) => {
      env.emit(Inst::Integer(n))
    }
    Expr::Op2(&(op, x, y)) => {
      let x = compile_expr_one(env, x);
      let y = compile_expr_one(env, y);
      env.emit(Inst::Op2(op, x, y))
    }
    _ => {
      unimplemented!()
    }
  }
}

fn compile_expr_tail<'a>(env: &mut Env, x: Expr<'a>) {
  match x {
    Expr::Op2(&(op, x, y)) => {
      let x = compile_expr_one(env, x);
      let y = compile_expr_one(env, y);
      let z = env.emit(Inst::Op2(op, x, y));
      let _ = env.emit(Inst::Put(z));
      let _ = env.emit(Inst::Ret);
    }
    _ => {
      unimplemented!()
    }
  }
}
