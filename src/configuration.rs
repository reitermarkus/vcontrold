use std::io;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use serde_derive::*;
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
  len: Option<usize>,
  pos: Option<usize>,
  get: Option<String>,
  set: Option<String>,
  mode: AccessMode,
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
  pub fn get_command<P: Protocol>(&self, o: &mut Optolink, command: &str) -> Result<Box<fmt::Display>, io::Error> {
    let unit = self.commands[command].unit.clone();

    let addr = &self.commands[command].addr.to_be().to_bytes();
    let len = self.commands[command].len.unwrap_or(unit.size());
    let pos = self.commands[command].pos.unwrap_or(0);

    let mut buf = vec![0; len];
    P::get(o, &addr, &mut buf)?;

    Ok(unit.bytes_to_output(&buf[pos..(pos + unit.size())]))
  }

  pub fn set_command<P: Protocol>(&self, o: &mut Optolink, command: &str, input: &str) -> Result<(), io::Error> {
    let addr = &self.commands[command].addr.to_be().to_bytes();
    let unit = self.commands[command].unit.clone();

    P::set(o, &addr, &unit.input_to_bytes(input).unwrap().to_bytes())?;
    Ok(())
  }
}

#[inline(always)]
fn f32_one() -> f32 {
  1.0
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum Unit {
  #[serde(rename = "i8")]
  I8 { #[serde(default = "f32_one")] factor: f32 },
  #[serde(rename = "i16")]
  I16 { #[serde(default = "f32_one")] factor: f32 },
  #[serde(rename = "i32")]
  I32 { #[serde(default = "f32_one")] factor: f32 },
  #[serde(rename = "u8")]
  U8 { #[serde(default = "f32_one")] factor: f32 },
  #[serde(rename = "u16")]
  U16 { #[serde(default = "f32_one")] factor: f32 },
  #[serde(rename = "u32")]
  U32 { #[serde(default = "f32_one")] factor: f32 },
  #[serde(rename = "enum")]
  Enum { mapping: HashMap<Vec<u8>, String> },
  #[serde(rename = "systime")]
  SysTime,
  #[serde(rename = "errstate")]
  ErrState { mapping: HashMap<Vec<u8>, String> },
  #[serde(rename = "cycletime")]
  CycleTime,
}

impl Unit {
  pub fn size(&self) -> usize {
    use self::Unit::*;

    match self {
      I8 { .. } => std::mem::size_of::<i8>(),
      I16 { .. } => std::mem::size_of::<i16>(),
      I32 { .. } => std::mem::size_of::<i32>(),
      U8 { .. } => std::mem::size_of::<u8>(),
      U16 { .. } => std::mem::size_of::<u16>(),
      U32 { .. } => std::mem::size_of::<u32>(),
      Enum { mapping, .. } => mapping.keys().next().unwrap().len(),
      SysTime { .. } => std::mem::size_of::<self::SysTime>(),
      ErrState { .. } => std::mem::size_of::<self::ErrState>(),
      CycleTime { .. } => std::mem::size_of::<self::CycleTime>(),
    }
  }

  pub fn bytes_to_output(&self, bytes: &[u8]) -> Box<fmt::Display> {
    match self {
      Unit::I8 { factor, .. }    => Box::new(i8::from_bytes(bytes) as f32 / factor),
      Unit::I16 { factor, .. }   => Box::new(i16::from_bytes(bytes) as f32 / factor),
      Unit::I32 { factor, .. }   => Box::new(i32::from_bytes(bytes) as f32 / factor),
      Unit::U8 { factor, .. }    => Box::new(u8::from_bytes(bytes) as f32 / factor),
      Unit::U16 { factor, .. }   => Box::new(u16::from_bytes(bytes) as f32 / factor),
      Unit::U32 { factor, .. }   => Box::new(u32::from_bytes(bytes) as f32 / factor),
      Unit::Enum { mapping, .. } => Box::new(mapping[&bytes.to_vec()].to_owned()),
      Unit::SysTime { .. }       => Box::new(SysTime::from_bytes(bytes)),
      Unit::ErrState { mapping, .. } => {
        let errstate = ErrState::from_bytes(bytes);
        Box::new(format!("{} ({})", mapping[&errstate.id().to_vec()], errstate.time()))
      },
      Unit::CycleTime { .. }     => Box::new(CycleTime::from_bytes(bytes)),
    }
  }

  pub fn input_to_bytes(&self, input: &str) -> Result<Box<ToBytes>, String> {
    Ok(match self {
      Unit::I8 { factor, .. }    => Box::new(input.parse::<f32>().map(|v| (v * factor) as i8).map_err(|err| err.to_string())?),
      Unit::I16 { factor, .. }   => Box::new(input.parse::<f32>().map(|v| (v * factor) as i16).map_err(|err| err.to_string())?),
      Unit::I32 { factor, .. }   => Box::new(input.parse::<f32>().map(|v| (v * factor) as i32).map_err(|err| err.to_string())?),
      Unit::U8 { factor, .. }    => Box::new(input.parse::<f32>().map(|v| (v * factor) as u8).map_err(|err| err.to_string())?),
      Unit::U16 { factor, .. }   => Box::new(input.parse::<f32>().map(|v| (v * factor) as u16).map_err(|err| err.to_string())?),
      Unit::U32 { factor, .. }   => Box::new(input.parse::<f32>().map(|v| (v * factor) as u32).map_err(|err| err.to_string())?),
      Unit::Enum { mapping, .. } => Box::new(mapping.iter().find(|(_, value)| value == &input).map(|(key, _)| key.clone()).unwrap()),
      Unit::SysTime { .. }       => Box::new(input.parse::<SysTime>()?),
      Unit::ErrState { .. }      => Box::new(input.parse::<ErrState>()?),
      Unit::CycleTime { .. }     => Box::new(input.parse::<CycleTime>()?),
    })
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
