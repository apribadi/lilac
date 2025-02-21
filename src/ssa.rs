use crate::prelude::*;

pub enum Inst<'a> {
  // block entry

  Func(u32, TypeList<'a>),
  Case(),
  Join(TypeList<'a>),
  Kont(TypeList<'a>),

  // block middle

  ConstBool(bool),
  ConstI32(u32),
  ConstI64(u64),
  Op1(Op1, Value),
  Op2(Op2, Value, Value),
  Select(Value, Value, Value),
  LetVariable(Value),
  GetVariable(Variable),
  SetVariable(Variable, Value),

  // block terminator

  If(Value, Label, Label),
  Return(u32, ValueList<'a>),
  Goto(Label, ValueList<'a>),
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Tag(pub u8);

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Type(pub u8);

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Op1(pub u8);

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Op2(pub u8);

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Value(pub u32);

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Label(pub u32);

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Variable(pub u32);

pub struct TypeList<'a>(&'a [u8]);

pub struct ValueList<'a>(&'a [u8]);

impl Tag {
  pub const FUNC: Self = Self(0x01);
  pub const CASE: Self = Self(0x02);
  pub const JOIN: Self = Self(0x03);
  pub const KONT: Self = Self(0x04);

  pub const CONST_BOOL: Self = Self(0x0f);
  pub const CONST_I32: Self = Self(0x08);
  pub const CONST_I64: Self = Self(0x09);
  pub const OP1: Self = Self(0x05);
  pub const OP2: Self = Self(0x06);
  pub const SELECT: Self = Self(0x07);

  pub const LET_VARIABLE: Self = Self(0x10);
  pub const GET_VARIABLE: Self = Self(0x11);
  pub const SET_VARIABLE: Self = Self(0x12);

  pub const IF: Self = Self(0x0a);
  pub const GOTO: Self = Self(0x0b);
  pub const RETURN: Self = Self(0x0c);
  pub const CALL: Self = Self(0x0d);
  pub const TAILCALL: Self = Self(0x0e);
}

impl Type {
  pub const BOOL: Self = Self(0x02);
  pub const I5: Self = Self(0x03);
  pub const I6: Self = Self(0x04);
  pub const I32: Self = Self(0x07);
  pub const I64: Self = Self(0x08);

  pub fn name(self) -> &'static str {
    self.info().0
  }

  fn info(self)
    -> &'static (
      &'static str,
    )
  {
    match self {
      Self::I64 => &(
        "i64",
      ),
      _ => &(
        "unknown",
      )
    }
  }
}

impl Op1 {
  pub const CAST_I32_I64_SX: Self = Self(0x01);
  pub const CAST_I32_I64_ZX: Self = Self(0x02);
  pub const CAST_I64_I32: Self = Self(0x03);
  pub const CTZ_I64: Self = Self(0x07);
  pub const NEG_I64: Self = Self(0x08);

  pub fn name(self) -> &'static str {
    self.info().0
  }

  fn info(self)
    -> &'static (
      &'static str,
    )
  {
    match self {
      Self::CTZ_I64 => &(
        "ctz.i64",
      ),
      Self::NEG_I64 => &(
        "neg.i64",
      ),
      _ => &(
        "unknown",
      )
    }
  }
}

impl Op2 {
  pub const ADD_I64: Self = Self(0x06);
  pub const SUB_I64: Self = Self(0x07);
  pub const IS_EQ_I64: Self = Self(0x08);

  pub fn name(self) -> &'static str {
    self.info().0
  }

  fn info(self)
    -> &'static (
      &'static str,
    )
  {
    match self {
      Self::ADD_I64 => &(
        "add.i64",
      ),
      Self::SUB_I64 => &(
        "sub.i64",
      ),
      Self::IS_EQ_I64 => &(
        "is_eq.i64",
      ),
      _ => &(
        "unknown",
      )
    }
  }
}

impl core::fmt::Display for Type {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "{}", self.name())
  }
}

impl core::fmt::Display for Op1 {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "{}", self.name())
  }
}

impl core::fmt::Display for Op2 {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "{}", self.name())
  }
}

impl core::fmt::Display for Value {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "%{}", self.0)
  }
}

impl core::fmt::Display for Label {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "=>{}", self.0)
  }
}

impl core::fmt::Display for Variable {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "@{}", self.0)
  }
}

impl<'a> TypeList<'a> {
  #[inline(always)]
  pub fn iter(&self) -> impl Iterator<Item = Type> {
    self.0.iter_chunks().map(|&[x]| Type(x))
  }
}

impl<'a> ValueList<'a> {
  #[inline(always)]
  pub fn iter(&self) -> impl Iterator<Item = Value> {
    self.0.iter_chunks().map(|x| Value(u32::from_le_bytes(*x)))
  }
}

pub struct Builder {
  buf: Buf,
  value_id: u32,
  label_id: u32,
  variable_id: u32,
}

#[derive(Clone, Copy)]
pub struct PatchPoint(pub usize);

fn incr(x: &mut u32) -> u32 {
  let n = *x;
  *x = n + 1;
  n
}

impl Builder {
  pub fn new() -> Self {
    Self {
      buf: Buf::new(),
      value_id: 0,
      label_id: 0,
      variable_id: 0,
    }
  }

  pub fn next_value(&mut self) -> Value {
    Value(incr(&mut self.value_id))
  }

  pub fn next_label(&mut self) -> Label {
    Label(incr(&mut self.label_id))
  }

  pub fn next_variable(&mut self) -> Variable {
    Variable(incr(&mut self.variable_id))
  }

  pub fn view(&self) -> &[u8] {
    self.buf.view()
  }

  pub fn patch_label(&mut self, i: PatchPoint, a: Label) {
    let mut w = self.buf.get_slice_mut(i.0, 4);
    w.put_u32(a.0)
  }

  pub fn emit_param(&mut self, t: Type) -> Value {
    let mut w = self.buf.append(1);
    w.put_u8(t.0);
    self.next_value()
  }

  pub fn emit_value(&mut self, x: Value) {
    let mut w = self.buf.append(4);
    w.put_u32(x.0);
  }

  pub fn emit_func(&mut self, nkonts: u32, nargs: u32) {
    let mut w = self.buf.append(9);
    w.put_u8(Tag::FUNC.0);
    w.put_u32(nkonts);
    w.put_u32(nargs);
    self.value_id = 0;
    self.label_id = 1;
  }

  pub fn emit_case(&mut self) -> Label {
    let mut w = self.buf.append(1);
    w.put_u8(Tag::CASE.0);
    self.next_label()
  }

  pub fn emit_join(&mut self, nargs: u32) -> Label {
    let mut w = self.buf.append(5);
    w.put_u8(Tag::JOIN.0);
    w.put_u32(nargs);
    self.next_label()
  }

  pub fn emit_const_bool(&mut self, p: bool) -> Value {
    let mut w = self.buf.append(2);
    w.put_u8(Tag::CONST_BOOL.0);
    w.put_u8(p as u8);
    self.next_value()
  }

  pub fn emit_const_i32(&mut self, c: u32) -> Value {
    let mut w = self.buf.append(5);
    w.put_u8(Tag::CONST_I32.0);
    w.put_u32(c);
    self.next_value()
  }

  pub fn emit_const_i64(&mut self, c: u64) -> Value {
    let mut w = self.buf.append(9);
    w.put_u8(Tag::CONST_I64.0);
    w.put_u64(c);
    self.next_value()
  }

  pub fn emit_op1(&mut self, t: Op1, x: Value) -> Value {
    let mut w = self.buf.append(6);
    w.put_u8(Tag::OP1.0);
    w.put_u8(t.0);
    w.put_u32(x.0);
    self.next_value()
  }

  pub fn emit_op2(&mut self, t: Op2, x: Value, y: Value) -> Value {
    let mut w = self.buf.append(10);
    w.put_u8(Tag::OP2.0);
    w.put_u8(t.0);
    w.put_u32(x.0);
    w.put_u32(y.0);
    self.next_value()
  }

  pub fn emit_select(&mut self, p: Value, x: Value, y: Value) -> Value {
    let mut w = self.buf.append(13);
    w.put_u8(Tag::SELECT.0);
    w.put_u32(p.0);
    w.put_u32(x.0);
    w.put_u32(y.0);
    self.next_value()
  }

  pub fn emit_let_variable(&mut self, x: Value) -> Variable {
    let mut w = self.buf.append(5);
    w.put_u8(Tag::LET_VARIABLE.0);
    w.put_u32(x.0);
    self.next_variable()
  }

  pub fn emit_get_variable(&mut self, x: Variable) -> Value {
    let mut w = self.buf.append(5);
    w.put_u8(Tag::GET_VARIABLE.0);
    w.put_u32(x.0);
    self.next_value()
  }

  pub fn emit_set_variable(&mut self, x: Variable, y: Value) {
    let mut w = self.buf.append(9);
    w.put_u8(Tag::SET_VARIABLE.0);
    w.put_u32(x.0);
    w.put_u32(y.0);
  }

  pub fn emit_if(&mut self, p: Value, a: Label, b: Label) -> (PatchPoint, PatchPoint) {
    let n = self.buf.len();
    let mut w = self.buf.append(13);
    w.put_u8(Tag::IF.0);
    w.put_u32(p.0);
    w.put_u32(a.0);
    w.put_u32(b.0);
    (PatchPoint(n + 5), PatchPoint(n + 9))
  }

  pub fn emit_goto(&mut self, a: Label, nargs: u32) -> PatchPoint {
    let n = self.buf.len();
    let mut w = self.buf.append(9);
    w.put_u8(Tag::GOTO.0);
    w.put_u32(a.0);
    w.put_u32(nargs);
    PatchPoint(n + 1)
  }

  pub fn emit_return(&mut self, index: u32, nargs: u32) {
    let mut w = self.buf.append(9);
    w.put_u8(Tag::RETURN.0);
    w.put_u32(index);
    w.put_u32(nargs);
  }
}

fn chomp<'a, 'b>(buf: &'a mut &'b [u8], size: usize) -> Option<&'b [u8]> {
  if size <= buf.len() {
    Some(buf.pop_slice(size))
  } else {
    None
  }
}

pub fn read<'a, 'b>(buf: &'a mut &'b [u8]) -> Option<Inst<'b>> {
  let mut cursor = *buf;

  let instr =
    match Tag(chomp(&mut cursor, 1)?.pop_u8()) {
      Tag::FUNC => {
        let mut r = chomp(&mut cursor, 8)?;
        let nkonts = r.pop_u32();
        let nargs = r.pop_u32();
        let mut r = chomp(&mut cursor, nargs as usize)?;
        Inst::Func(nkonts, TypeList(r.pop_all()))
      }
      Tag::CASE => {
        Inst::Case()
      }
      Tag::JOIN => {
        let mut r = chomp(&mut cursor, 4)?;
        let nargs = r.pop_u32();
        let mut r = chomp(&mut cursor, nargs as usize)?;
        Inst::Join(TypeList(r.pop_all()))
      }
      Tag::CONST_BOOL => {
        let mut r = chomp(&mut cursor, 1)?;
        Inst::ConstBool(r.pop_u8() != 0)
      }
      Tag::CONST_I32 => {
        let mut r = chomp(&mut cursor, 4)?;
        Inst::ConstI32(r.pop_u32())
      }
      Tag::CONST_I64 => {
        let mut r = chomp(&mut cursor, 8)?;
        Inst::ConstI64(r.pop_u64())
      }
      Tag::OP1 => {
        let mut r = chomp(&mut cursor, 5)?;
        let t = Op1(r.pop_u8());
        let x = Value(r.pop_u32());
        Inst::Op1(t, x)
      }
      Tag::OP2 => {
        let mut r = chomp(&mut cursor, 9)?;
        let t = Op2(r.pop_u8());
        let x = Value(r.pop_u32());
        let y = Value(r.pop_u32());
        Inst::Op2(t, x, y)
      }
      Tag::SELECT => {
        let mut r = chomp(&mut cursor, 12)?;
        let p = Value(r.pop_u32());
        let x = Value(r.pop_u32());
        let y = Value(r.pop_u32());
        Inst::Select(p, x, y)
      }
      Tag::LET_VARIABLE => {
        let mut r = chomp(&mut cursor, 4)?;
        let x = Value(r.pop_u32());
        Inst::LetVariable(x)
      }
      Tag::GET_VARIABLE => {
        let mut r = chomp(&mut cursor, 4)?;
        let x = Variable(r.pop_u32());
        Inst::GetVariable(x)
      }
      Tag::SET_VARIABLE => {
        let mut r = chomp(&mut cursor, 8)?;
        let x = Variable(r.pop_u32());
        let y = Value(r.pop_u32());
        Inst::SetVariable(x, y)
      }
      Tag::IF => {
        let mut r = chomp(&mut cursor, 12)?;
        let p = Value(r.pop_u32());
        let a = Label(r.pop_u32());
        let b = Label(r.pop_u32());
        Inst::If(p, a, b)
      }
      Tag::GOTO => {
        let mut r = chomp(&mut cursor, 8)?;
        let a = r.pop_u32();
        let nargs = r.pop_u32();
        let mut r = chomp(&mut cursor, nargs as usize * 4)?;
        Inst::Goto(Label(a), ValueList(r.pop_all()))
      }
      Tag::RETURN => {
        let mut r = chomp(&mut cursor, 8)?;
        let index = r.pop_u32();
        let nargs = r.pop_u32();
        let mut r = chomp(&mut cursor, nargs as usize * 4)?;
        Inst::Return(index, ValueList(r.pop_all()))
      }
      _ => {
        return None;
      }
    };

  *buf = cursor;

  return Some(instr);
}

pub fn display(buf: &[u8]) {
  let mut r = buf;
  let mut func_id = 0;
  let mut label_id = 0;
  let mut value_id = 0;
  let mut variable_id = 0;
  let mut nkonts = 0;

  fn next(x: &mut u32) -> u32 {
    let y = *x;
    *x = y + 1;
    y
  }

  while let Some(inst) = read(&mut r) {
    match inst {
      Inst::Func(n, args) => {
        label_id = 0;
        value_id = 0;
        variable_id = 0;
        nkonts = n;
        print!("{}: func ${} (", next(&mut func_id), next(&mut label_id));
        for (i, ty) in args.iter().enumerate() {
          if i != 0 {
            print!(", ");
          }
          print!("%{} {}", next(&mut value_id), ty);
        }
        print!(") -> ");
        if nkonts == 0 {
          print!("!");
        } else {
          print!("(");
          for i in 0 .. nkonts {
            if i != 0 {
              print!("|");
            }
            print!("...");
          }
          print!(")");
        }
        print!("\n");
      }
      Inst::Case() => {
        print!("{}: case\n", next(&mut label_id));
      }
      Inst::Join(args) => {
        print!("{}: join (", next(&mut label_id));
        for (i, ty) in args.iter().enumerate() {
          if i != 0 {
            print!(", ");
          }
          print!("%{} {}", next(&mut value_id), ty);
        }
        print!(")\n");
      }
      Inst::Kont(args) => {
        print!("{}: kont (", next(&mut label_id));
        for (i, ty) in args.iter().enumerate() {
          if i != 0 {
            print!(", ");
          }
          print!("%{} {}", next(&mut value_id), ty);
        }
        print!(")\n");
      }
      Inst::ConstBool(p) => {
        print!("\t%{} = const.bool #{}\n", next(&mut value_id), p);
      }
      Inst::ConstI32(c) => {
        print!("\t%{} = const.i32 #{}\n", next(&mut value_id), c);
      }
      Inst::ConstI64(c) => {
        print!("\t%{} = const.i64 #{}\n", next(&mut value_id), c);
      }
      Inst::Op1(t, x) => {
        print!("\t%{} = {} {}\n", next(&mut value_id), t, x);
      }
      Inst::Op2(t, x, y) => {
        print!("\t%{} = {} {} {}\n", next(&mut value_id), t, x, y);
      }
      Inst::Select(p, x, y) => {
        print!("\t%{} = select {} {} {}\n", next(&mut value_id), p, x, y);
      }
      Inst::LetVariable(x) => {
        print!("\tlet mutable @{} = {}\n", next(&mut variable_id), x);
      }
      Inst::GetVariable(x) => {
        print!("\t%{} = {}\n", next(&mut value_id), x);
      }
      Inst::SetVariable(x, y) => {
        print!("\t{} <- {}\n", x, y);
      }
      Inst::If(p, a, b) => {
        print!("\tif {} then {} else {}\n", p, a, b);
      }
      Inst::Goto(a, args) => {
        print!("\tgoto {} (", a);
        for (i, x) in args.iter().enumerate() {
          if i != 0 {
            print!(", ");
          }
          print!("{}", x);
        }
        print!(")\n");
      }
      Inst::Return(index, args) => {
        print!("\treturn (", );
        for _ in 0 .. index {
          print!("|")
        }
        for (i, x) in args.iter().enumerate() {
          if i != 0 {
            print!(", ");
          }
          print!("{}", x);
        }
        for _ in 0 .. nkonts.saturating_sub(index).saturating_sub(1) {
          print!("|")
        }
        print!(")\n");
      }
    }
  }
}
