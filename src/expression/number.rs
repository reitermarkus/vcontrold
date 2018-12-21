use std::fmt;

#[derive(PartialEq, Clone)]
pub enum Number {
  Float(f32),
  Int(i32),
}

impl fmt::Debug for Number   {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Number::Float(float) => write!(f, "{}", float),
      Number::Int(int) => write!(f, "{}", int),
    }
  }
}

impl std::ops::Add for Number {
  type Output = Number;

  fn add(self, other: Number) -> Number {
    match (self, other) {
      (Number::Float(lhs), Number::Float(rhs)) => Number::Float(lhs + rhs),
      (Number::Int(lhs), Number::Int(rhs)) => Number::Int(lhs + rhs),
      (Number::Int(lhs), Number::Float(rhs)) => Number::Float(lhs as f32 + rhs),
      (Number::Float(lhs), Number::Int(rhs)) => Number::Float(lhs + rhs as f32),
    }
  }
}

impl std::ops::Sub for Number {
  type Output = Number;

  fn sub(self, other: Number) -> Number {
    match (self, other) {
      (Number::Float(lhs), Number::Float(rhs)) => Number::Float(lhs - rhs),
      (Number::Int(lhs), Number::Int(rhs)) => Number::Int(lhs - rhs),
      (Number::Int(lhs), Number::Float(rhs)) => Number::Float(lhs as f32 - rhs),
      (Number::Float(lhs), Number::Int(rhs)) => Number::Float(lhs - rhs as f32),
    }
  }
}

impl std::ops::Div for Number {
  type Output = Number;

  fn div(self, other: Number) -> Number {
    match (self, other) {
      (Number::Float(lhs), Number::Float(rhs)) => Number::Float(lhs / rhs),
      (Number::Int(lhs), Number::Int(rhs)) => Number::Int(lhs / rhs),
      (Number::Int(lhs), Number::Float(rhs)) => Number::Float(lhs as f32 / rhs),
      (Number::Float(lhs), Number::Int(rhs)) => Number::Float(lhs / rhs as f32),
    }
  }
}

impl std::ops::Mul for Number {
  type Output = Number;

  fn mul(self, other: Number) -> Number {
    match (self, other) {
      (Number::Float(lhs), Number::Float(rhs)) => Number::Float(lhs * rhs),
      (Number::Int(lhs), Number::Int(rhs)) => Number::Int(lhs * rhs),
      (Number::Int(lhs), Number::Float(rhs)) => Number::Float(lhs as f32 * rhs),
      (Number::Float(lhs), Number::Int(rhs)) => Number::Float(lhs * rhs as f32),
    }
  }
}

impl std::ops::Rem for Number {
  type Output = Number;

  fn rem(self, other: Number) -> Number {
    match (self, other) {
      (Number::Int(lhs), Number::Int(rhs)) => Number::Int(lhs % rhs),
      _ => unimplemented!("% for Number::Float"),
    }
  }
}

impl std::ops::Neg for Number {
  type Output = Number;

  fn neg(self) -> Number {
    match self {
      Number::Float(n) => Number::Float(-n),
      Number::Int(n) => Number::Int(-n),
    }
  }
}
impl std::ops::Not for Number {
  type Output = Number;

  fn not(self) -> Number {
    match self {
      Number::Int(n) => Number::Int(!n),
      _ => unimplemented!("~ for Number::Float"),
    }
  }
}

impl std::ops::BitAnd for Number {
  type Output = Number;

  fn bitand(self, other: Number) -> Number {
    match (self, other) {
      (Number::Int(lhs), Number::Int(rhs)) => Number::Int(lhs & rhs),
      _ => unimplemented!("& for Number::Float"),
    }
  }
}

impl std::ops::BitXor for Number {
  type Output = Number;

  fn bitxor(self, other: Number) -> Number {
    match (self, other) {
      (Number::Int(lhs), Number::Int(rhs)) => Number::Int(lhs ^ rhs),
      _ => unimplemented!("^ for Number::Float"),
    }
  }
}

impl std::ops::BitOr for Number {
  type Output = Number;

  fn bitor(self, other: Number) -> Number {
    match (self, other) {
      (Number::Int(lhs), Number::Int(rhs)) => Number::Int(lhs | rhs),
      _ => unimplemented!("| for Number::Float"),
    }
  }
}

impl std::ops::Shl for Number {
  type Output = Number;

  fn shl(self, other: Number) -> Number {
    match (self, other) {
      (Number::Int(lhs), Number::Int(rhs)) => Number::Int(lhs << rhs),
      _ => unimplemented!("<< for Number::Float"),
    }
  }
}

impl std::ops::Shr for Number {
  type Output = Number;

  fn shr(self, other: Number) -> Number {
    match (self, other) {
      (Number::Int(lhs), Number::Int(rhs)) => Number::Int(lhs >> rhs),
      _ => unimplemented!(">> for Number::Float"),
    }
  }
}
