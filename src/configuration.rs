use std::collections::HashMap;
use std::fmt;

use byteorder::{ByteOrder, LittleEndian};
use serde_derive::*;
use serde_yaml;
use serde::de::{self, Deserialize, Deserializer};

use crate::expression::Expression;
use crate::types::SysTime;

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

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum Unit {
  #[serde(rename = "int")]
  Int { name: String, #[serde(default)] calc: Calc },
  #[serde(rename = "uint")]
  UInt { name: String, #[serde(default)] calc: Calc },
  #[serde(rename = "short")]
  Short { name: String, #[serde(default)] calc: Calc },
  #[serde(rename = "uchar")]
  UChar { name: String, #[serde(default)] calc: Calc },
  #[serde(rename = "enum")]
  Enum { name: String, mapping: HashMap<Vec<u8>, String> },
  #[serde(rename = "systime")]
  SysTime { name: String },
  #[serde(rename = "errstate")]
  ErrState { name: String },
  #[serde(rename = "cycletime")]
  CycleTime { name: String },
}

#[derive(Debug)]
pub struct ErrState([u8; 9]);

impl FromBytes for ErrState {
  fn from_bytes(bytes: &[u8]) -> ErrState {
    assert_eq!(bytes.len(), std::mem::size_of::<ErrState>());
    let mut buf = [0; std::mem::size_of::<ErrState>()];
    buf.copy_from_slice(&bytes);
    ErrState(buf)
  }
}

impl fmt::Display for ErrState {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:02X} ({})", self.0[0], SysTime::from_bytes(&self.0[1..9]))
  }
}

#[derive(Debug)]
pub struct CycleTime([u8; 8]);

impl CycleTime {
  fn byte_to_time(&self, i: usize) -> Option<(u8, u8)> {
    match self.0[i] {
      0xff => None,
      byte => Some((byte >> 3, (byte & 0b111) * 10)),
    }
  }

  pub fn times(&self) -> [Option<((u8, u8), (u8, u8))>; 4] {
    [
      self.byte_to_time(0).and_then(|from| self.byte_to_time(1).map(|to| (from, to))),
      self.byte_to_time(2).and_then(|from| self.byte_to_time(3).map(|to| (from, to))),
      self.byte_to_time(4).and_then(|from| self.byte_to_time(5).map(|to| (from, to))),
      self.byte_to_time(6).and_then(|from| self.byte_to_time(7).map(|to| (from, to))),
    ]
  }
}

impl FromBytes for CycleTime {
  fn from_bytes(bytes: &[u8]) -> CycleTime {
    assert_eq!(bytes.len(), std::mem::size_of::<CycleTime>());
    let mut buf = [0; std::mem::size_of::<CycleTime>()];
    buf.copy_from_slice(&bytes);
    CycleTime(buf)
  }
}

impl fmt::Display for CycleTime {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}",
      self.times().into_iter().map(|o| o.map(|((from_h, from_m), (to_h, to_m))| format!("{:02}:{:02}-{:02}:{:02}", from_h, from_m, to_h, to_m)).unwrap_or("".into())).collect::<Vec<String>>().join(","),
    )
  }
}


impl Unit {
  pub fn size(&self) -> usize {
    use self::Unit::*;

    match self {
      Int { .. } => std::mem::size_of::<i32>(),
      UInt { .. } => std::mem::size_of::<u32>(),
      Short { .. } => std::mem::size_of::<i16>(),
      UChar { .. } => std::mem::size_of::<u8>(),
      Enum { mapping, .. } => mapping.keys().next().unwrap().len(),
      SysTime { .. } => std::mem::size_of::<self::SysTime>(),
      ErrState { .. } => std::mem::size_of::<self::ErrState>(),
      CycleTime { .. } => std::mem::size_of::<self::CycleTime>(),
    }
  }

  pub fn bytes_to_output(&self, bytes: &[u8]) -> Output {
    use self::Unit::*;
    use self::Output::*;

    assert_eq!(bytes.len(), self.size());

    match self {
      Int { .. } => Float(LittleEndian::read_i32(bytes) as f32, bytes.to_vec()),
      UInt { .. } => Float(LittleEndian::read_u32(bytes) as f32, bytes.to_vec()),
      Short { .. } => Float(LittleEndian::read_i16(bytes) as f32, bytes.to_vec()),
      UChar { .. } => Float(bytes[0] as f32, bytes.to_vec()),
      Enum { mapping, .. } => String(mapping[bytes].clone(), bytes.to_vec()),
      SysTime { .. } => String(self::SysTime::from_bytes(bytes).to_string(), bytes.to_vec()),
      ErrState { .. } => String(self::ErrState::from_bytes(bytes).to_string(), bytes.to_vec()),
      CycleTime { .. } => String(self::CycleTime::from_bytes(bytes).to_string(), bytes.to_vec()),
    }
  }
}

pub enum Output {
  Float(f32, Vec<u8>),
  String(String, Vec<u8>),
}

impl Output {
  pub fn as_bytes(&self) -> &[u8] {
    match self {
      Output::Float(_, bytes) => bytes,
      Output::String(_, bytes) => bytes,
    }
  }
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct Calc {
  get: Option<Expression>,
  set: Option<Expression>,
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
