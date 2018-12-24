use std::io;
use std::collections::HashMap;
use std::fmt;

use serde_derive::*;
use serde_yaml;
use serde::de::{self, Deserialize, Deserializer};

use crate::types::{*};
use crate::{OptoLink, Protocol};

pub const DEFAULT_CONFIG: &str = include_str!("../config/default.yml");

#[derive(Debug, Clone, Deserialize)]
pub struct Command {
  addr: Vec<u8>,
  unit: String,
  get: Option<String>,
  set: Option<String>,
}

#[derive(Clone)]
pub enum Scalar {
  Byte(u8),
  Addr,
  Bytes,
  Len,
}

impl<'de> Deserialize<'de> for Scalar {
  fn deserialize<D>(deserializer: D) -> Result<Scalar, D::Error>
  where
      D: Deserializer<'de>,
  {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum ByteOrVariable {
      Byte(u8),
      Variable(String),
    }

    match ByteOrVariable::deserialize(deserializer)? {
      ByteOrVariable::Byte(byte) => Ok(Scalar::Byte(byte)),
      ByteOrVariable::Variable(variable) => match variable.as_str() {
        "$addr" => Ok(Scalar::Addr),
        "$bytes" => Ok(Scalar::Bytes),
        "$len" => Ok(Scalar::Len),
        variant => Err(de::Error::unknown_variant(&variant, &["$addr", "$bytes", "$len"])),
      },
    }
  }
}

impl fmt::Debug for Scalar {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Scalar::Byte(b) => write!(f, "{:02X}", b),
      Scalar::Addr => write!(f, "$addr"),
      Scalar::Bytes => write!(f, "$bytes"),
      Scalar::Len => write!(f, "$len"),
    }
  }
}

#[derive(Debug, Deserialize)]
pub struct Configuration {
  pub commands: HashMap<String, Command>,
  pub units: HashMap<String, Unit>,
}

pub trait FromBytes {
  fn from_bytes(bytes: &[u8]) -> Self;
}

pub trait ToBytes {
  fn to_bytes(&self) -> Vec<u8>;
}

impl FromBytes for Vec<u8> {
  fn from_bytes(bytes: &[u8]) -> Vec<u8> {
    bytes.to_vec()
  }
}

impl ToBytes for Vec<u8> {
  fn to_bytes(&self) -> Vec<u8> {
    self.clone()
  }
}

macro_rules! from_bytes_le {
  ($($t:ty),+) => {
    $(
      impl FromBytes for $t {
        fn from_bytes(bytes: &[u8]) -> Self {
          let mut buf = [0; std::mem::size_of::<Self>()];
          buf.copy_from_slice(&bytes);
          Self::from_le(unsafe { std::mem::transmute(buf) })
        }
      }
    )+
  };
}

macro_rules! to_bytes_le {
  ($t:ty, [u8; 1]) => {
    impl ToBytes for $t {
      fn to_bytes(&self) -> Vec<u8> {
        vec![*self as u8]
      }
    }
  };
  ($t:ty, $n:ty) => {
    impl ToBytes for $t {
      fn to_bytes(&self) -> Vec<u8> {
        unsafe { std::mem::transmute::<$t, $n>(self.to_le()) }.to_vec()
      }
    }
  };
}

from_bytes_le!(i8, i16, i32);
to_bytes_le!(i8,  [u8; 1]);
to_bytes_le!(i16, [u8; 2]);
to_bytes_le!(i32, [u8; 4]);

from_bytes_le!(u8, u16, u32);
to_bytes_le!(u8,  [u8; 1]);
to_bytes_le!(u16, [u8; 2]);
to_bytes_le!(u32, [u8; 4]);

impl Default for Configuration {
  fn default() -> Configuration {
    Configuration::from_str(&DEFAULT_CONFIG).unwrap()
  }
}

impl Configuration {
  pub fn from_str(string: &str) -> Result<Configuration, serde_yaml::Error> {
    serde_yaml::from_str(&string)
  }

  pub fn get_command<P: Protocol>(&self, o: &mut OptoLink, command: &str) -> Result<Box<fmt::Display>, io::Error> {
    let addr = &self.commands[command].addr;
    let unit = self.unit_for_command(&command);

    let mut buf = vec![0; unit.size()];
    P::get(o, &addr, &mut buf)?;
    Ok(unit.bytes_to_output(&buf))
  }

  pub fn set_command<P: Protocol>(&self, o: &mut OptoLink, command: &str, input: &str) -> Result<(), io::Error> {
    let addr = &self.commands[command].addr;
    let unit = self.unit_for_command(&command);

    P::set(o, &addr, &unit.input_to_bytes(input).unwrap().to_bytes())?;
    Ok(())
  }

  fn unit_for_command(&self, command: &str) -> &Unit {
    let unit = &self.commands[command].unit;
    &self.units[unit]
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
  I8 { name: String, #[serde(default = "f32_one")] factor: f32 },
  #[serde(rename = "i16")]
  I16 { name: String, #[serde(default = "f32_one")] factor: f32 },
  #[serde(rename = "i32")]
  I32 { name: String, #[serde(default = "f32_one")] factor: f32 },
  #[serde(rename = "u8")]
  U8 { name: String, #[serde(default = "f32_one")] factor: f32 },
  #[serde(rename = "u16")]
  U16 { name: String, #[serde(default = "f32_one")] factor: f32 },
  #[serde(rename = "u32")]
  U32 { name: String, #[serde(default = "f32_one")] factor: f32 },
  #[serde(rename = "enum")]
  Enum { name: String, mapping: HashMap<Vec<u8>, String> },
  #[serde(rename = "systime")]
  SysTime { name: String },
  #[serde(rename = "errstate")]
  ErrState { name: String, mapping: HashMap<Vec<u8>, String> },
  #[serde(rename = "cycletime")]
  CycleTime { name: String },
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
