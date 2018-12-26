use std::io;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use serde_derive::*;
use serde::ser::{Serialize, Serializer};
use serde::de::{self, Deserialize, Deserializer};
use serde_yaml;
use yaml_merge_keys;

use crate::types::{*};
use crate::{Optolink, protocol::Protocol, FromBytes, ToBytes};

const DEFAULT_CONFIG: &str = include_str!("../config/default.yml");

#[derive(Debug, Clone, Copy)]
pub enum AccessMode {
  Read,
  Write,
  ReadWrite,
}

impl<'de> Deserialize<'de> for AccessMode {
  fn deserialize<D>(deserializer: D) -> Result<AccessMode, D::Error>
  where
      D: Deserializer<'de>,
  {
    match String::deserialize(deserializer)?.as_str() {
      "read" => Ok(AccessMode::Read),
      "write" => Ok(AccessMode::Write),
      "read_write" => Ok(AccessMode::ReadWrite),
      variant => Err(de::Error::unknown_variant(&variant, &["read", "write", "read_write"])),
    }
  }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Command {
  addr: u16,
  unit: Unit,
  byte_len: Option<usize>,
  #[serde(default)]
  byte_pos: usize,
  mode: AccessMode,
  bit_pos: Option<usize>,
  bit_len: Option<usize>,
  factor: Option<f32>,
  mapping: Option<HashMap<Vec<u8>, String>>,
}

impl Command {
  #[inline]
  fn addr(&self) -> Vec<u8> {
    self.addr.to_be().to_bytes()
  }

  pub fn get<P: Protocol>(&self, o: &mut Optolink) -> Result<Box<fmt::Display>, io::Error> {
    let byte_len = self.byte_len.unwrap_or(self.unit.size());

    let mut buf = vec![0; byte_len];
    P::get(o, &self.addr(), &mut buf)?;

    if let Some(bit_pos) = self.bit_pos {
      let byte = buf[bit_pos / 8];
      let bit_len = self.bit_len.unwrap_or(1);

      buf.clear();
      buf.push((byte << (bit_pos % 8)) >> (8 - bit_len));
    }

    Ok(self.unit.bytes_to_output(&buf[self.byte_pos..(self.byte_pos + self.unit.size())], self.factor, &self.mapping))
  }

  pub fn set<P: Protocol>(&self, o: &mut Optolink, input: &str) -> Result<(), io::Error> {
    P::set(o, &self.addr(), &self.unit.input_to_bytes(input, self.factor)?)
  }
}

#[derive(Debug, Deserialize)]
pub struct Configuration {
  pub commands: HashMap<String, Command>,
}

impl Default for Configuration {
  fn default() -> Configuration {
    DEFAULT_CONFIG.parse().unwrap()
  }
}

impl FromStr for Configuration {
  type Err = String;

  fn from_str(s: &str) -> Result<Configuration, Self::Err> {
    let value: serde_yaml::Value = serde_yaml::from_str(&s).map_err(|err| err.to_string())?;

    let value = yaml_merge_keys::merge_keys_serde(value).map_err(|err| err.kind().description().to_string())?;

    serde_yaml::from_value(value).map_err(|err| err.to_string())
  }
}

impl Configuration {
  pub fn command(&self, command: &str) -> &Command {
    &self.commands[command]
  }
}

#[inline(always)]
fn f32_one() -> f32 {
  1.0
}

#[derive(Debug, Clone)]
enum Unit {
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
    use self::Unit::*;

    match self {
      I8 => std::mem::size_of::<i8>(),
      I16 => std::mem::size_of::<i16>(),
      I32 => std::mem::size_of::<i32>(),
      U8 => std::mem::size_of::<u8>(),
      U16 => std::mem::size_of::<u16>(),
      U32 => std::mem::size_of::<u32>(),
      SysTime => std::mem::size_of::<self::SysTime>(),
      ErrState => std::mem::size_of::<self::ErrState>(),
      CycleTime => std::mem::size_of::<self::CycleTime>(),
    }
  }

  pub fn bytes_to_output(&self, bytes: &[u8], factor: Option<f32>, mapping: &Option<HashMap<Vec<u8>, String>>) -> Box<fmt::Display> {
    if let Some(mapping) = mapping {
      return Box::new(mapping[bytes].to_owned())
    }

    let n = match self {
      Unit::SysTime => return Box::new(SysTime::from_bytes(bytes)),
      Unit::CycleTime => return Box::new(CycleTime::from_bytes(bytes)),
      Unit::I8 => i8::from_bytes(bytes).to_le() as i64,
      Unit::I16 => i16::from_bytes(bytes).to_le() as i64,
      Unit::I32 => i32::from_bytes(bytes).to_le() as i64,
      Unit::U8 => u8::from_bytes(bytes).to_le() as i64,
      Unit::U16 => u16::from_bytes(bytes).to_le() as i64,
      Unit::U32 => u32::from_bytes(bytes).to_le() as i64,
    };

    if let Some(factor) = factor {
      return Box::new(n as f32 / factor)
    }

    Box::new(n)
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

#[cfg(test)]
mod tests {
  use super::*;

  use serde_yaml;

  #[test]
  fn config() {
    let config = Configuration::default();
    println!("{:#?}", config);
  }
}
