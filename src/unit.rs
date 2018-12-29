use std::collections::HashMap;
use std::fmt;
use std::io;

use serde::de::{self, Deserialize, Deserializer};

use crate::{Value, FromBytes, ToBytes, types::{SysTime, CycleTime}};

#[derive(Debug, Clone)]
pub enum Unit {
  I8,
  I16,
  I32,
  U8,
  U16,
  U32,
  SysTime,
  CycleTime,
}

impl<'de> Deserialize<'de> for Unit {
  fn deserialize<D>(deserializer: D) -> Result<Unit, D::Error>
  where
      D: Deserializer<'de>,
  {
    match String::deserialize(deserializer)?.as_str() {
      "i8" => Ok(Unit::I8),
      "i16" => Ok(Unit::I16),
      "i32" => Ok(Unit::I32),
      "u8" => Ok(Unit::U8),
      "u16" => Ok(Unit::U16),
      "u32" => Ok(Unit::U32),
      "systime" => Ok(Unit::SysTime),
      "cycletime" => Ok(Unit::CycleTime),
      variant => Err(de::Error::unknown_variant(&variant, &["i8", "i16", "i32", "u8", "u16", "u32", "systime", "cycletime"])),
    }
  }
}

impl Unit {
  pub fn size(&self) -> usize {
    match self {
      Unit::I8 => std::mem::size_of::<i8>(),
      Unit::I16 => std::mem::size_of::<i16>(),
      Unit::I32 => std::mem::size_of::<i32>(),
      Unit::U8 => std::mem::size_of::<u8>(),
      Unit::U16 => std::mem::size_of::<u16>(),
      Unit::U32 => std::mem::size_of::<u32>(),
      Unit::SysTime => std::mem::size_of::<SysTime>(),
      Unit::CycleTime => std::mem::size_of::<CycleTime>(),
    }
  }

  pub fn bytes_to_output(&self, bytes: &[u8], factor: Option<f32>, mapping: &Option<HashMap<Vec<u8>, String>>) -> Value {
    if let Some(mapping) = mapping {
      return Value::String(mapping[bytes].to_owned())
    }

    let n = match self {
      Unit::SysTime => return Value::SysTime(SysTime::from_bytes(bytes)),
      Unit::CycleTime => return Value::CycleTime(CycleTime::from_bytes(bytes)),
      Unit::I8 => i8::from_bytes(bytes).to_le() as i64,
      Unit::I16 => i16::from_bytes(bytes).to_le() as i64,
      Unit::I32 => i32::from_bytes(bytes).to_le() as i64,
      Unit::U8 => u8::from_bytes(bytes).to_le() as i64,
      Unit::U16 => u16::from_bytes(bytes).to_le() as i64,
      Unit::U32 => u32::from_bytes(bytes).to_le() as i64,
    };

    if let Some(factor) = factor {
      return Value::Float(n as f64 / factor as f64)
    }

    Value::Int(n)
  }

  pub fn input_to_bytes(&self, input: &str, factor: Option<f32>) -> Result<Vec<u8>, io::Error> {
    let factor = factor.unwrap_or(1.0);

    fn invalid_input(err: impl fmt::Display) -> io::Error {
      io::Error::new(std::io::ErrorKind::InvalidInput, err.to_string())
    }

    match self {
      Unit::I8 => input.parse::<f32>().map(|v| ((v * factor) as i8).to_bytes()).map_err(invalid_input),
      Unit::I16 => input.parse::<f32>().map(|v| ((v * factor) as i16).to_bytes()).map_err(invalid_input),
      Unit::I32 => input.parse::<f32>().map(|v| ((v * factor) as i32).to_bytes()).map_err(invalid_input),
      Unit::U8 => input.parse::<f32>().map(|v| ((v * factor) as u8).to_bytes()).map_err(invalid_input),
      Unit::U16 => input.parse::<f32>().map(|v| ((v * factor) as u16).to_bytes()).map_err(invalid_input),
      Unit::U32 => input.parse::<f32>().map(|v| ((v * factor) as u32).to_bytes()).map_err(invalid_input),
      Unit::SysTime => input.parse::<SysTime>().map(|v| v.to_bytes()).map_err(invalid_input),
      Unit::CycleTime => input.parse::<CycleTime>().map(|v| v.to_bytes()).map_err(invalid_input),
    }
  }
}
