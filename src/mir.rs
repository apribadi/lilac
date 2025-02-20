// (function $fib (($n i64)) ((i64))
//   (loop $continue-loop
//     (($n $n)
//      ($x #1)
//      ($y #0))
//     (if (i64.is_eq $n #0)
//       $y
//       (do
//         (let ($a) (i64.add $x $y))
//         (let ($b) (i64.sub $n #1))
//         (goto $continue-loop ($y $a $b))))))

use crate::prelude::*;

// An expr can *potentially* return a single value to a single continuation.

#[derive(Clone, Copy)]
pub enum Exp<'a> {
  Call(&'a Call<'a>),
  Do(&'a [Statement<'a>]),
  If(&'a If<'a>),
  Symbol(Symbol<'a>),
  ConstBool(bool),
  ConstI32(u32),
  ConstI64(u64),
}

const _: () = assert!(size_of::<Exp<'static>>() <= 24);


// let x = ...
// let x, y, z = ...
// goto ... ..., ..., ...
// return ..., ..., ...

#[derive(Clone, Copy)]
pub enum Statement<'a> {
  Let(Symbol<'a>, Exp<'a>),
  LetVariable(Symbol<'a>, Exp<'a>),
  SetVariable(Symbol<'a>, Exp<'a>),
  Goto(Symbol<'a>, &'a [Exp<'a>]),
  Return(),
}

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct Symbol<'a>(pub &'a [u8]);

#[derive(Clone, Copy)]
pub enum Type {
  I64,
}

/*
#[derive(Clone, Copy)]
pub enum ScalarType {
  Bool,
  I5,
  I6,
  I32,
  I64,
}

pub enum Type<'a> {
  Scalar(ScalarType),
  Tuple(&'a [Type<'a>]),
}

pub enum EffectType {
}
*/

pub struct Function<'a> {
  pub name: Symbol<'a>,
  pub params: &'a [(Symbol<'a>, Type)],
  // pub rets: &'a [&'a [Type]],
  pub body: Exp<'a>,
}

#[derive(Clone, Copy)]
pub struct Call<'a> {
  pub function: Symbol<'a>,
  pub args: &'a [Exp<'a>],
}

#[derive(Clone, Copy)]
pub struct If<'a> {
  pub condition: Exp<'a>,
  pub if_true: Exp<'a>,
  pub if_false: Exp<'a>,
}

#[derive(Clone, Copy)]
pub struct Loop<'a> {
  pub name: Symbol<'a>,
  pub bindings: &'a [(Symbol<'a>, Exp<'a>)],
  pub body: Exp<'a>,
}

pub static FIB: Function<'static> = Function {
  name: Symbol(b"fib"),
  params: &[(Symbol(b"n"), Type::I64)],
  //body: Exp::ConstI64(13),
  body:
    Exp::Call(&Call {
      function: Symbol(b"add.i64"),
      args: &[
        Exp::If(&If {
          condition: Exp::ConstBool(false),
          if_true: Exp::Symbol(Symbol(b"n")),
          if_false: Exp::ConstI64(2)
        }),
        Exp::If(&If {
          condition: Exp::ConstBool(true),
          if_true: Exp::ConstI64(3),
          if_false: Exp::ConstI64(4)
        })
      ]
    })
};
