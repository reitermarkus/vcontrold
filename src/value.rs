use serde::ser::{Serialize, Serializer};

use crate::types::{SysTime, CycleTime};

#[derive(Debug)]
pub enum Value {
  Int(i64),
  Float(f64),
  SysTime(SysTime),
  CycleTime(CycleTime),
  String(String),
}

impl Serialize for Value {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
      S: Serializer,
  {
    match self {
      Value::Int(n) => n.serialize(serializer),
      Value::Float(f) => f.serialize(serializer),
      Value::SysTime(systime) => systime.serialize(serializer),
      Value::CycleTime(cycletime) => cycletime.serialize(serializer),
      Value::String(s) => s.serialize(serializer),
    }
  }
}
