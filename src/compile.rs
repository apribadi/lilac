use crate::ast::Expr;
use crate::symbol::Symbol;
use crate::uir::Inst;

struct Code(Vec<Inst>);

impl Code {
  fn new() -> Self {
    Self(Vec::new())
  }

  fn put(&mut self, inst: Inst) -> u32 {
    let n = self.0.len();
    self.0.push(inst);
    return n as u32;
  }

  fn set(&mut self, index: u32, inst: Inst) {
    self.0[index as usize] = inst;
  }
}

struct Env {
  args: Vec<u32>,
}

impl Env {
  fn new() -> Self {
    Self {
      args: Vec::new(),
    }
  }

  fn put_arg(&mut self, x: u32) {
    self.args.push(x)
  }

  fn pop_arg_multi(&mut self, arity: usize) -> impl Iterator<Item = u32> {
    return self.args.drain(self.args.len() - arity ..);
  }
}

pub fn compile<'a>(x: Expr<'a>) -> Vec<Inst> {
  let mut t = Env::new();
  let mut o = Code::new();

  compile_expr_tail(&mut t, &mut o, x);
  return o.0;
}

fn compile_expr<'a>(t: &mut Env, o: &mut Code, x: Expr<'a>) -> u32 {
  match x {
    Expr::And(&(x, y)) => {
      let x = compile_expr(t, o, x);
      let i = o.put(Inst::Undefined); // Cond(x, a, b)
      let a = o.put(Inst::Label);
      let f = o.put(Inst::ConstBool(false));
      let _ = o.put(Inst::Put(f));
      let j = o.put(Inst::Undefined); // Jump(c)
      let b = o.put(Inst::Label);
      let y = compile_expr(t, o, y);
      let _ = o.put(Inst::Put(y));
      let k = o.put(Inst::Undefined); // Jump(c)
      let c = o.put(Inst::Label);
      o.set(i, Inst::Cond(x, a, b));
      o.set(j, Inst::Jump(c));
      o.set(k, Inst::Jump(c));
      return o.put(Inst::Pop);
    }
    Expr::Call(&(f, x)) => {
      let n = x.len();
      let f = compile_expr(t, o, f);
      for x in x {
        let x = compile_expr(t, o, *x);
        t.put_arg(x);
      }
      for x in t.pop_arg_multi(n) {
        let _ = o.put(Inst::Put(x));
      }
      let i = o.put(Inst::Undefined); // Call(a)
      let a = o.put(Inst::Label);
      o.set(i, Inst::Call(f, a));
      return o.put(Inst::Pop);
    }
    Expr::Field(&(symbol, x)) => {
      let x = compile_expr(t, o, x);
      return o.put(Inst::Field(Symbol::from_bytes(symbol), x));
    }
    Expr::Index(&(x, i)) => {
      let x = compile_expr(t, o, x);
      let i = compile_expr(t, o, i);
      return o.put(Inst::Index(x, i));
    }
    Expr::Int(n) => {
      return o.put(Inst::ConstInt(n));
    }
    Expr::Op1(&(op, x)) => {
      let x = compile_expr(t, o, x);
      return o.put(Inst::Op1(op, x));
    }
    Expr::Op2(&(op, x, y)) => {
      let x = compile_expr(t, o, x);
      let y = compile_expr(t, o, y);
      return o.put(Inst::Op2(op, x, y));
    }
    Expr::Or(&(x, y)) => {
      let x = compile_expr(t, o, x);
      let i = o.put(Inst::Undefined); // Cond(x, a, b)
      let a = o.put(Inst::Label);
      let y = compile_expr(t, o, y);
      let _ = o.put(Inst::Put(y));
      let j = o.put(Inst::Undefined); // Jump(c)
      let b = o.put(Inst::Label);
      let t = o.put(Inst::ConstBool(true));
      let _ = o.put(Inst::Put(t));
      let k = o.put(Inst::Undefined); // Jump(c)
      let c = o.put(Inst::Label);
      o.set(i, Inst::Cond(x, a, b));
      o.set(j, Inst::Jump(c));
      o.set(k, Inst::Jump(c));
      return o.put(Inst::Pop);
    }
    Expr::Ternary(&(p, x, y)) => {
      let p = compile_expr(t, o, p);
      let i = o.put(Inst::Undefined); // Cond(p, a, b)
      let a = o.put(Inst::Label);
      let x = compile_expr(t, o, x);
      let _ = o.put(Inst::Put(x));
      let j = o.put(Inst::Undefined); // Jump(c)
      let b = o.put(Inst::Label);
      let y = compile_expr(t, o, y);
      let _ = o.put(Inst::Put(y));
      let k = o.put(Inst::Undefined); // Jump(c)
      let c = o.put(Inst::Label);
      o.set(i, Inst::Cond(p, a, b));
      o.set(j, Inst::Jump(c));
      o.set(k, Inst::Jump(c));
      return o.put(Inst::Pop);
    }
    Expr::Undefined => {
      return o.put(Inst::Undefined);
    }
    Expr::Variable(symbol) => {
      // TODO: local scope
      return o.put(Inst::Global(Symbol::from_bytes(symbol)));
    }
  }
}

fn compile_expr_tail<'a>(t: &mut Env, o: &mut Code, x: Expr<'a>) {
  match x {
    Expr::And(&(x, y)) => {
      let x = compile_expr(t, o, x);
      let i = o.put(Inst::Undefined); // Cond(x, a, b)
      let a = o.put(Inst::Label);
      let f = o.put(Inst::ConstBool(false));
      let _ = o.put(Inst::Put(f));
      let _ = o.put(Inst::Ret);
      let b = o.put(Inst::Label);
      compile_expr_tail(t, o, y);
      o.set(i, Inst::Cond(x, a, b));
    }
    Expr::Call(&(f, x)) => {
      let n = x.len();
      let f = compile_expr(t, o, f);
      for x in x {
        let x = compile_expr(t, o, *x);
        t.put_arg(x);
      }
      for x in t.pop_arg_multi(n) {
        let _ = o.put(Inst::Put(x));
      }
      let _ = o.put(Inst::TailCall(f));
    }
    Expr::Or(&(x, y)) => {
      let x = compile_expr(t, o, x);
      let i = o.put(Inst::Undefined); // Cond(x, a, b)
      let a = o.put(Inst::Label);
      compile_expr_tail(t, o, y);
      let b = o.put(Inst::Label);
      let t = o.put(Inst::ConstBool(true));
      let _ = o.put(Inst::Put(t));
      let _ = o.put(Inst::Ret);
      o.set(i, Inst::Cond(x, a, b));
    }
    Expr::Ternary(&(p, x, y)) => {
      let p = compile_expr(t, o, p);
      let i = o.put(Inst::Undefined); // Cond(p, a, b)
      let a = o.put(Inst::Label);
      compile_expr_tail(t, o, x);
      let b = o.put(Inst::Label);
      compile_expr_tail(t, o, y);
      o.set(i, Inst::Cond(p, a, b));
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
      let x = compile_expr(t, o, x);
      let _ = o.put(Inst::Put(x));
      let _ = o.put(Inst::Ret);
    }
  }
}
