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
