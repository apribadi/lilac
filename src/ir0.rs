#[derive(Debug)]
pub struct Symbol(pub u64);

#[derive(Debug)]
pub enum Inst {
  Label { n: u32 },
  Pop { i: u32 },
  Put { i: u32, x: u32 },
  Jump { n: u32, a: u32 },
  If { x: u32, a: u32, b: u32 },
  Call { n: u32, f: u32, k: u32 },
  TailCall { n: u32, f: u32 },
  Ret { n: u32 },
  Global { s: Symbol },
  OpAdd { x: u32, y: u32 },    // x + y
  OpBitAnd { x: u32, y: u32 }, // x & y
  OpBitOr { x: u32, y: u32 },  // x | y
  OpBitXor { x: u32, y: u32 }, // x ^ y
  OpCmpEq { x: u32, y: u32 },  // x == y
  OpCmpGe { x: u32, y: u32 },  // x >= y
  OpCmpGt { x: u32, y: u32 },  // x > y
  OpCmpLe { x: u32, y: u32 },  // x <= y
  OpCmpLt { x: u32, y: u32 },  // x < y
  OpCmpNe { x: u32, y: u32 },  // x != y
  OpDiv { x: u32, y: u32 },    // x / y
  OpMul { x: u32, y: u32 },    // x * y
  OpNeg { x: u32 },            // - x
  OpNot { x: u32, y: u32 },    // ! x
  OpRem { x: u32, y: u32 },    // x % y
  OpShl { x: u32, y: u32 },    // x << y
  OpShr { x: u32, y: u32 },    // x >> y
  OpSub { x: u32, y: u32 },    // x - y
}
