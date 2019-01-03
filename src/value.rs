use std::str::FromStr;

use serde_derive::*;

use crate::types::{SysTime, CycleTime};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
  Number(f64),
  SysTime(SysTime),
  CycleTime(CycleTime),
  String(String),
}

#[derive(Debug)]
pub enum Never {}

impl FromStr for Value {
  type Err = Never;

  fn from_str(s: &str) -> Result<Value, Self::Err> {
    if let Ok(number) = s.parse::<f64>() {
      return Ok(Value::Number(number))
    }

    if let Ok(systime) = s.parse::<SysTime>() {
      return Ok(Value::SysTime(systime))
    }

    if let Ok(cycletime) = s.parse::<CycleTime>() {
      return Ok(Value::CycleTime(cycletime))
    }

    Ok(Value::String(s.to_owned()))
  }
}
