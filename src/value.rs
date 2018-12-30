use serde_derive::*;

use crate::types::{SysTime, CycleTime};

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
  Int(i64),
  Float(f64),
  SysTime(SysTime),
  CycleTime(CycleTime),
  String(String),
}