// (func $fib (($n i64)) ((i64))
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

// (func fib ((n))
//   (do
//     (let-mutable n n)
//     (let-mutable x 1)
//     (let-mutable y 0)
//     (while (is_ne.i64 n 0)
//       (do
//       ))
//     y))
//

// use crate::prelude::*;

// An expr can *potentially* return a single value to a single continuation.

#[derive(Clone, Copy)]
pub enum Expr<'a> {
  Call(&'a Call<'a>),
  Do(&'a [Stmt<'a>]),
  If(&'a If<'a>),
  Symbol(Symbol<'a>),
  ConstBool(bool),
  ConstI32(u32),
  ConstI64(u64),
}

const _: () = assert!(size_of::<Expr<'static>>() <= 24);


// let x = ...
// let x, y, z = ...
// goto ... ..., ..., ...
// return ..., ..., ...

#[derive(Clone, Copy)]
pub enum Stmt<'a> {
  Expr(Expr<'a>),
  Let(&'a [Symbol<'a>], Expr<'a>),
  LetLocal(Symbol<'a>, Expr<'a>),
  SetLocal(Symbol<'a>, Expr<'a>),
  Return(&'a [Expr<'a>]),
  While(Expr<'a>, Expr<'a>),
  // Goto(Symbol<'a>, &'a [Expr<'a>]),
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

pub struct Func<'a> {
  pub name: Symbol<'a>,
  pub params: &'a [(Symbol<'a>, Type)],
  // pub rets: &'a [&'a [Type]],
  pub body: Expr<'a>,
}

#[derive(Clone, Copy)]
pub struct Call<'a>(pub Symbol<'a>, pub &'a [Expr<'a>]);

#[derive(Clone, Copy)]
pub struct If<'a> {
  pub condition: Expr<'a>,
  pub if_true: Expr<'a>,
  pub if_false: Expr<'a>,
}

#[derive(Clone, Copy)]
pub struct Loop<'a> {
  pub name: Symbol<'a>,
  pub bindings: &'a [(Symbol<'a>, Expr<'a>)],
  pub body: Expr<'a>,
}

pub static FOO: Func<'static> = Func {
  name: Symbol(b"foo"),
  params: &[
    (Symbol(b"x"), Type::I64),
    (Symbol(b"y"), Type::I64),
    (Symbol(b"z"), Type::I64),
  ],
  body:
    Expr::Do(&[
      Stmt::Let(
        &[Symbol(b"a")],
        Expr::Call(&Call(
          Symbol(b"add.i64"),
          &[Expr::Symbol(Symbol(b"x")), Expr::Symbol(Symbol(b"y"))]
        ))
      ),
      Stmt::Let(
        &[Symbol(b"b")],
        Expr::Call(&Call(
          Symbol(b"add.i64"),
          &[Expr::Symbol(Symbol(b"a")), Expr::Symbol(Symbol(b"z"))]
        ))
      ),
      Stmt::Expr(Expr::Symbol(Symbol(b"b")))
    ])
};

pub static FIB: Func<'static> = Func {
  name: Symbol(b"fib"),
  params: &[(Symbol(b"n"), Type::I64)],
  body:
    Expr::Do(&[
      Stmt::LetLocal(Symbol(b"n"), Expr::Symbol(Symbol(b"n"))),
      Stmt::LetLocal(Symbol(b"x"), Expr::ConstI64(1)),
      Stmt::LetLocal(Symbol(b"y"), Expr::ConstI64(0)),
      Stmt::While(
        Expr::Call(&Call(
          Symbol(b"is_ne.i64"),
          &[Expr::Symbol(Symbol(b"n")), Expr::ConstI64(0)]
        )),
        Expr::Do(&[
          Stmt::SetLocal(
            Symbol(b"n"),
            Expr::Call(&Call(
              Symbol(b"sub.i64"),
              &[Expr::Symbol(Symbol(b"n")), Expr::ConstI64(1)]
            ))
          ),
          Stmt::Let(
            &[Symbol(b"z")],
            Expr::Call(&Call(
              Symbol(b"add.i64"),
              &[Expr::Symbol(Symbol(b"x")), Expr::Symbol(Symbol(b"y"))]
            ))
          ),
          Stmt::SetLocal(Symbol(b"x"), Expr::Symbol(Symbol(b"y"))),
          Stmt::SetLocal(Symbol(b"y"), Expr::Symbol(Symbol(b"z")))
        ])
      ),
      Stmt::Return(&[Expr::Symbol(Symbol(b"y"))])
    ])

    /*

    Expr::Call(&Call {
      func: Symbol(b"add.i64"),
      args: &[
        Expr::If(&If {
          condition: Expr::ConstBool(false),
          if_true: Expr::Symbol(Symbol(b"n")),
          if_false: Expr::ConstI64(2)
        }),
        Expr::If(&If {
          condition: Expr::ConstBool(true),
          if_true: Expr::ConstI64(3),
          if_false: Expr::ConstI64(4)
        })
      ]
    })
    */
};

