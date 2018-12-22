use std::collections::HashMap;
use std::fmt;

use byteorder::{ByteOrder, LittleEndian};
use serde_derive::*;
use serde_yaml;
use serde::de::{self, Deserialize, Deserializer};

use crate::expression::Expression;
use crate::types::{*};

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

#[derive(Clone)]
pub enum ProtocolCommand {
  Command(String),
  Send(Vec<Scalar>),
  Wait(Vec<u8>),
  Recv(String),
}

impl<'de> Deserialize<'de> for ProtocolCommand {
  fn deserialize<D>(deserializer: D) -> Result<ProtocolCommand, D::Error>
  where
      D: Deserializer<'de>,
  {
    #[derive(Deserialize)]
    enum Operation {
      #[serde(rename = "send")]
      Send(Vec<Scalar>),
      #[serde(rename = "wait")]
      Wait(Vec<u8>),
      #[serde(rename = "recv")]
      Recv(String),
    }

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OperationOrReference {
      Operation(Operation),
      Reference(String),
    }

    match OperationOrReference::deserialize(deserializer)? {
      OperationOrReference::Operation(operation) => match operation {
        Operation::Send(scalars) => Ok(ProtocolCommand::Send(scalars)),
        Operation::Wait(bytes) => Ok(ProtocolCommand::Wait(bytes)),
        Operation::Recv(string) => Ok(ProtocolCommand::Recv(string)),
      },
      OperationOrReference::Reference(reference) => Ok(ProtocolCommand::Command(reference)),
    }
  }
}

impl fmt::Debug for ProtocolCommand {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      ProtocolCommand::Command(command) => write!(f, "{}", command)?,
      ProtocolCommand::Send(scalars) => {
        write!(f, "send")?;

        for s in scalars {
          write!(f, " {:?}", s)?;
        }
      },
      ProtocolCommand::Wait(bytes) => {
        write!(f, "wait")?;

        for b in bytes {
          write!(f, " {:02X}", b)?;
        }
      },
      ProtocolCommand::Recv(unit) => {
        write!(f, "recv {}", unit)?;
      }
    }

    Ok(())
  }
}

#[derive(Debug, Deserialize)]
pub struct Configuration {
  pub commands: HashMap<String, Command>,
  pub units: HashMap<String, Unit>,
  pub protocols: HashMap<String, HashMap<String, Vec<ProtocolCommand>>>,
}

pub trait FromBytes {
  fn from_bytes(bytes:&[u8]) -> Self;
}

pub trait AsBytes {
  fn as_bytes(&self) -> &[u8];
}

impl FromBytes for u8 {
  fn from_bytes(bytes: &[u8]) -> u8 {
    assert_eq!(bytes.len(), 1);
    bytes[0]
  }
}

impl FromBytes for i16 {
  fn from_bytes(bytes: &[u8]) -> i16 {
    LittleEndian::read_i16(&bytes)
  }
}

impl Default for Configuration {
  fn default() -> Configuration {
    Configuration::from_str(&DEFAULT_CONFIG).unwrap()
  }
}

impl Configuration {
  pub fn from_str(string: &str) -> Result<Configuration, serde_yaml::Error> {
    let mut config: Configuration = serde_yaml::from_str(&string)?;
    config.canonicalize_protocols();
    Ok(config)
  }

  fn canonicalize_protocols(&mut self) {
    self.protocols = self.protocols.clone().into_iter().map(|(protocol_name, commands)| {
      (
        protocol_name,
        commands.clone().into_iter().map(|(command_name, command)| {
          (
            command_name,
            command.into_iter().flat_map(|operation| {
              match operation {
                ProtocolCommand::Command(alias_name) => commands[&alias_name].clone(),
                operation => vec![operation],
              }
            }).collect(),
          )
        }).collect(),
      )
    }).collect();
  }

  pub fn prepare_command(&self, protocol: &str, command: &str, action: &str, bytes: &[u8]) -> Vec<PreparedProtocolCommand> {
    let procol_command_name = if action == "get" {
      &self.commands[command].get
    } else {
      &self.commands[command].set
    };

    let addr = &self.commands[command].addr;

    let unit = self.unit_for_command(&command);
    let protocol_command = &self.protocols[protocol][&procol_command_name.clone().unwrap()];
    self.prepare_command_raw(&protocol_command, &addr, &bytes, &unit)
  }

  pub fn prepare_command_raw(&self, command: &[ProtocolCommand], addr: &[u8], bytes: &[u8], unit: &Unit) -> Vec<PreparedProtocolCommand> {
    let len = unit.size();

    command.to_vec().into_iter().map(|protocol_operation| {
      match protocol_operation {
        ProtocolCommand::Send(scalars) => PreparedProtocolCommand::Send(scalars.into_iter().flat_map(|scalar| {
            match scalar {
              Scalar::Addr => addr.to_vec(),
              Scalar::Bytes => {
                assert_eq!(bytes.len(), len, "length of $bytes must match $len");
                bytes.to_vec()
              },
              Scalar::Byte(byte) => vec![byte],
              Scalar::Len => vec![unit.size() as u8],
            }
        }).collect()),
        ProtocolCommand::Wait(bytes) => PreparedProtocolCommand::Wait(bytes),
        ProtocolCommand::Recv(_) => PreparedProtocolCommand::Recv(unit.clone()),
        _ => panic!("canonicalize_protocols was not called"),
      }
    }).collect()
  }

  fn unit_for_command(&self, command: &str) -> &Unit {
    let unit = &self.commands[command].unit;
    &self.units[unit]
  }
}

#[derive(Debug, Clone)]
pub enum PreparedProtocolCommand {
  Send(Vec<u8>),
  Wait(Vec<u8>),
  Recv(Unit),
}

#[inline(always)]
fn f32_one() -> f32 {
  0.0
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum Unit {
  #[serde(rename = "f8")]
  F8 { name: String, #[serde(default = "f32_one")] factor: f32 },
  #[serde(rename = "f16")]
  F16 { name: String, #[serde(default = "f32_one")] factor: f32 },
  #[serde(rename = "f32")]
  F32 { name: String, #[serde(default = "f32_one")] factor: f32 },
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
  ErrState { name: String },
  #[serde(rename = "cycletime")]
  CycleTime { name: String },
}

impl Unit {
  pub fn size(&self) -> usize {
    use self::Unit::*;

    match self {
      F8 { .. } => std::mem::size_of::<Float8>(),
      F16 { .. } => std::mem::size_of::<Float16>(),
      F32 { .. } => std::mem::size_of::<Float32>(),
      I8 { .. } => std::mem::size_of::<Int8>(),
      I16 { .. } => std::mem::size_of::<Int16>(),
      I32 { .. } => std::mem::size_of::<Int32>(),
      U8 { .. } => std::mem::size_of::<UInt8>(),
      U16 { .. } => std::mem::size_of::<UInt16>(),
      U32 { .. } => std::mem::size_of::<UInt32>(),
      Enum { mapping, .. } => mapping.keys().next().unwrap().len(),
      SysTime { .. } => std::mem::size_of::<self::SysTime>(),
      ErrState { .. } => std::mem::size_of::<self::ErrState>(),
      CycleTime { .. } => std::mem::size_of::<self::CycleTime>(),
    }
  }

  pub fn bytes_to_output(&self, bytes: &[u8]) -> Box<fmt::Display> {
    assert_eq!(bytes.len(), self.size());

    match self {
      Unit::F8 { .. }            => Box::new(Float8::from_bytes(bytes)),
      Unit::F16 { .. }           => Box::new(Float16::from_bytes(bytes)),
      Unit::F32 { .. }           => Box::new(Float32::from_bytes(bytes)),
      Unit::I8 { .. }            => Box::new(Int8::from_bytes(bytes)),
      Unit::I16 { .. }           => Box::new(Int16::from_bytes(bytes)),
      Unit::I32 { .. }           => Box::new(Int32::from_bytes(bytes)),
      Unit::U8 { .. }            => Box::new(UInt8::from_bytes(bytes)),
      Unit::U16 { .. }           => Box::new(UInt16::from_bytes(bytes)),
      Unit::U32 { .. }           => Box::new(UInt32::from_bytes(bytes)),
      Unit::Enum { mapping, .. } => unimplemented!(),
      Unit::SysTime { .. }       => Box::new(SysTime::from_bytes(bytes)),
      Unit::ErrState { .. }      => Box::new(ErrState::from_bytes(bytes)),
      Unit::CycleTime { .. }     => Box::new(CycleTime::from_bytes(bytes)),
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
