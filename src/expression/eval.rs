use super::{Number, ParseNode::{self, *}, Op::*, Var::*};

pub fn eval(node: &ParseNode, value: i32, bytes: &[u8]) -> Result<Number, String> {
  match node {
    Number(n) => Ok(n.clone()),
    Var(Value) => Ok(Number::Int(value)),
    Var(Byte(i)) => if let Some(byte) = bytes.get(*i) {
      Ok(Number::Int(*byte as i32))
    } else {
      return Err(format!("missing byte at index {}", i))
    },
    Binary(op, lhs, rhs) => {
      let lhs = eval(lhs, value, bytes)?;
      let rhs = eval(rhs, value, bytes)?;

      Ok(match op {
        Add => lhs + rhs,
        Sub => lhs - rhs,
        Mul => lhs * rhs,
        Div => lhs / rhs,
        Mod => lhs % rhs,
        And => lhs & rhs,
        Xor => lhs ^ rhs,
        Or => lhs | rhs,
        Shl => lhs << rhs,
        Shr => lhs >> rhs,
        _ => unreachable!(),
      })
    },
    Unary(op, expr) => {
      let expr = eval(expr, value, bytes)?;

      Ok(match op {
        Sub => -expr,
        Not => !expr,
        _ => unreachable!(),
      })
    }
  }
}
