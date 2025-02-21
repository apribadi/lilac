//! ???
//!
//!

use lilac::ssa::Op2;
use lilac::ssa::Value;
use lilac::ssa::Label;
use lilac::ssa::Type;

fn main() {
  let mut buf = lilac::ssa::Builder::new();

  buf.emit_func(1, 1);
  let _ = buf.emit_param(Type::I64);
  let _ = buf.emit_const_i64(1);
  let _ = buf.emit_const_i64(0);
  let _ = buf.emit_goto(Label(1), 3);
  buf.emit_value(Value(0));
  buf.emit_value(Value(1));
  buf.emit_value(Value(2));
  let _ = buf.emit_join(3);
  let _ = buf.emit_param(Type::I64);
  let _ = buf.emit_param(Type::I64);
  let _ = buf.emit_param(Type::I64);
  let _ = buf.emit_op2(Op2::IS_EQ_I64, Value(3), Value(2));
  let _ = buf.emit_if(Value(6), Label(3), Label(2));
  let _ = buf.emit_case();
  let _ = buf.emit_op2(Op2::ADD_I64, Value(4), Value(5));
  let _ = buf.emit_op2(Op2::SUB_I64, Value(3), Value(1));
  let _ = buf.emit_goto(Label(1), 3);
  buf.emit_value(Value(8));
  buf.emit_value(Value(5));
  buf.emit_value(Value(7));
  let _ = buf.emit_case();
  buf.emit_return(0, 1);
  buf.emit_value(Value(5));

  lilac::ssa::display(buf.view());

  print!("\n\n");

  lilac::compile::compile_func(&lilac::mir::FIB);

  print!("\n\n");

  lilac::compile::compile_func(&lilac::mir::FOO);
}
