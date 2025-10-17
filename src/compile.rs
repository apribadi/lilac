use crate::ast::Expr;
use crate::symbol::Symbol;
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

fn compile_expr<'a>(env: &mut Env, x: Expr<'a>) -> u32 {
  match x {
    Expr::And(&(x, y)) => {
      let x = compile_expr(env, x);
      let i = env.emit(Inst::Undefined);
      let a = env.emit(Inst::Label);
      let f = env.emit(Inst::ConstBool(false));
      let _ = env.emit(Inst::Put(f));
      let j = env.emit(Inst::Undefined);
      let b = env.emit(Inst::Label);
      let y = compile_expr(env, y); // use compile_expr_kont?
      let _ = env.emit(Inst::Put(y));
      let k = env.emit(Inst::Undefined);
      let c = env.emit(Inst::Label);
      env.edit(i, Inst::Cond(x, a, b));
      env.edit(j, Inst::Jump(c));
      env.edit(k, Inst::Jump(c));
      env.emit(Inst::Pop)
    }
    Expr::Field(&(s, x)) => {
      let x = compile_expr(env, x);
      env.emit(Inst::Field(Symbol::from_bytes(s), x))
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
    _ => {
      unimplemented!()
    }
  }
}

fn compile_expr_tail<'a>(env: &mut Env, x: Expr<'a>) {
  match x {
    Expr::And(&(x, y)) => {
      let x = compile_expr(env, x);
      let i = env.emit(Inst::Undefined);
      let a = env.emit(Inst::Label);
      let f = env.emit(Inst::ConstBool(false));
      let _ = env.emit(Inst::Put(f));
      let _ = env.emit(Inst::Ret);
      let b = env.emit(Inst::Label);
      compile_expr_tail(env, y);
      env.edit(i, Inst::Cond(x, a, b));
    }
    x @ (
      | Expr::Field(_)
      | Expr::Index(_)
      | Expr::Int(_)
      | Expr::Op1(_)
      | Expr::Op2(_)
    ) => {
      let x = compile_expr(env, x);
      let _ = env.emit(Inst::Put(x));
      let _ = env.emit(Inst::Ret);
    }
    _ => {
      unimplemented!()
    }
  }
}
