use std::collections::HashMap;

use serde_derive::*;
use serde::de::{self, Deserialize, Deserializer};

use crate::{Error, Optolink, protocol::Protocol, Unit, Value, ToBytes};

#[derive(Debug, Clone, Copy)]
pub enum AccessMode {
  Read,
  Write,
  ReadWrite,
}

impl AccessMode {
  pub fn is_read(&self) -> bool {
    match self {
      AccessMode::Read | AccessMode::ReadWrite => true,
      _ => false,
    }
  }

  pub fn is_write(&self) -> bool {
    match self {
      AccessMode::Write | AccessMode::ReadWrite => true,
      _ => false,
    }
  }
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

/// A command which can be executed on an Optolink connection.
#[derive(Debug, Clone, Deserialize)]
pub struct Command {
  addr: u16,
  mode: AccessMode,
  unit: Unit,
  byte_len: Option<usize>,
  byte_pos: Option<usize>,
  bit_pos: Option<usize>,
  bit_len: Option<usize>,
  factor: Option<f64>,
  mapping: Option<HashMap<Vec<u8>, String>>,
}

impl Command {
  #[inline]
  fn addr(&self) -> Vec<u8> {
    self.addr.to_be().to_bytes()
  }

  pub fn get<P: Protocol>(&self, o: &mut Optolink) -> Result<Value, Error> {
    if !self.mode.is_read() {
      return Err(Error::UnsupportedMode(format!("Address 0x{:04X} does not support reading.", self.addr)))
    }

    let byte_len = self.byte_len.unwrap_or(self.unit.size());
    let byte_pos = self.byte_pos.unwrap_or(0);

    let mut buf = vec![0; byte_len];
    P::get(o, &self.addr(), &mut buf)?;

    if let Some(bit_pos) = self.bit_pos {
      let byte = buf[bit_pos / 8];
      let bit_len = self.bit_len.unwrap_or(1);

      buf.clear();
      buf.push((byte << (bit_pos % 8)) >> (8 - bit_len));
    }

    Ok(self.unit.bytes_to_output(&buf[byte_pos..(byte_pos + self.unit.size())], self.factor, &self.mapping))
  }

  pub fn set<P: Protocol>(&self, o: &mut Optolink, input: &Value) -> Result<(), Error> {
    if !self.mode.is_write() {
      return Err(Error::UnsupportedMode(format!("Address 0x{:04X} does not support writing.", self.addr)))
    }

    P::set(o, &self.addr(), &self.unit.input_to_bytes(input, self.factor, &self.mapping)?).map_err(Into::into)
  }
}
