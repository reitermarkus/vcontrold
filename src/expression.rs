use std::str::FromStr;

mod number;
use self::number::*;

mod lexer;
use self::lexer::*;

mod parser;
use self::parser::*;

mod eval;
use self::eval::*;

use serde_derive::*;
use serde_yaml;
use serde::de::{self, Deserialize, Deserializer};

#[derive(Debug, Clone)]
pub struct Expression(ParseNode);

impl FromStr for Expression {
  type Err = String;

  fn from_str(s: &str) -> Result<Expression, Self::Err> {
    Ok(Expression(ParseNode::from_str(s)?))
  }
}

impl Expression {
  pub fn eval(&self, value: i32, bytes: &[u8]) -> Result<Number, String> {
    eval(&self.0, value, bytes)
  }
}

impl<'de> Deserialize<'de> for Expression {
  fn deserialize<D>(deserializer: D) -> Result<Expression, D::Error>
  where
      D: Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    Expression::from_str(&s).map_err(de::Error::custom)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_lex() {
    // println!("{:?}", lex("0xFF + 3").unwrap());
    // println!("{:?}", lex("1024 << 0x04").unwrap());
    // println!("{:?}", lex("9.32 + ($b1 * $v)").unwrap());
    //
    // let l = parse_expr(&lex("$b1 * 100 + $b0").unwrap()).unwrap();
    // println!("{:?}", l);
    // println!("{:?}", eval(&l, 12, &[1, 2, 3, 4, 5, 6, 7, 8]).unwrap());
    //
    // let lexed_output = lex("~(1 & 2 | 3 ^ 4 & 5 + 10)").unwrap();
    // println!("lex: {:?}", lexed_output);
    //
    // let expr = parse_expr(&lexed_output).unwrap();
    // println!("parse: {:#?}", expr);
    //
    // println!("result: {:?}", eval(&expr, 0, &[]));
  }
}
