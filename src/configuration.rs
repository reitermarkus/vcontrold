use std::collections::HashMap;
use std::fmt;

use byteorder::{ByteOrder, LittleEndian};
use serde_derive::*;
use serde_yaml;
use serde::de::{self, Deserialize, Deserializer};

use crate::expression::Expression;

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
  CycleTime { name: String, #[serde(default)] calc: Calc },
}

#[derive(Debug)]
pub struct SysTime([u8; 8]);

#[inline]
fn byte_to_dec(byte: u8) -> u8 {
  byte / 16 * 10 + byte % 16
}

#[inline]
fn dec_to_byte(dec: u8) -> u8 {
  dec / 10 * 16 + dec % 10
}

impl SysTime {
  pub fn from_dec(year: u16, month: u8, day: u8, hour: u8, minute: u8, second: u8) -> SysTime {
    assert!(year <= 9999);
    assert!(month <= 12);
    assert!(day <= 31);
    assert!(hour <= 23);
    assert!(minute <= 59);
    assert!(second <= 59);

    SysTime([
      dec_to_byte((year / 100) as u8),
      dec_to_byte((year % 100) as u8),
      dec_to_byte(month),
      dec_to_byte(day),
      0,
      dec_to_byte(hour),
      dec_to_byte(minute),
      dec_to_byte(second),
    ])
  }

  pub fn year(&self) -> u16 {
    byte_to_dec(self.0[0]) as u16 * 100 + byte_to_dec(self.0[1]) as u16
  }

  pub fn month(&self) -> u8 {
    byte_to_dec(self.0[2])
  }

  pub fn day(&self) -> u8 {
    byte_to_dec(self.0[3])
  }

  pub fn weekday(&self) -> u8 {
    self.0[4] % 7
  }

  pub fn hour(&self) -> u8 {
    byte_to_dec(self.0[5])
  }

  pub fn minute(&self) -> u8 {
    byte_to_dec(self.0[6])
  }

  pub fn second(&self) -> u8 {
    byte_to_dec(self.0[7])
  }
}

impl FromBytes for SysTime {
  fn from_bytes(bytes: &[u8]) -> SysTime {
    assert_eq!(bytes.len(), 8);
    let mut buf = [0; 8];
    buf.copy_from_slice(&bytes);
    SysTime(buf)
  }
}

impl fmt::Display for SysTime {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
      self.year(),
      self.month(),
      self.day(),
      self.hour(),
      self.minute(),
      self.second(),
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
      ErrState { .. } => unimplemented!(),
      CycleTime { .. } => unimplemented!(),
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
      ErrState { .. } => unimplemented!(),
      CycleTime { .. } => unimplemented!(),
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
