use phf;
use serde::de::{self, Deserialize, Deserializer};

use crate::{Error, Optolink, protocol::Protocol, Unit, Value, ToBytes, types::Bytes};

#[derive(Debug, Clone, Copy)]
pub(crate) enum AccessMode {
  Read,
  Write,
  ReadWrite,
}

impl AccessMode {
  pub fn is_read(self) -> bool {
    match self {
      AccessMode::Read | AccessMode::ReadWrite => true,
      _ => false,
    }
  }

  pub fn is_write(self) -> bool {
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
#[derive(Debug)]
pub struct Command {
  pub(crate) addr: u16,
  pub(crate) mode: AccessMode,
  pub(crate) unit: Unit,
  pub(crate) block_len: usize,
  pub(crate) byte_len: usize,
  pub(crate) byte_pos: usize,
  pub(crate) bit_pos: Option<usize>,
  pub(crate) bit_len: Option<usize>,
  pub(crate) factor: f64,
  pub(crate) mapping: Option<phf::map::Map<Bytes, &'static str>>,
}

impl Command {
  #[inline]
  fn addr(&self) -> Vec<u8> {
    self.addr.to_be().to_bytes()
  }

  pub fn get<P: Protocol>(&self, o: &mut Optolink) -> Result<Value, Error> {
    log::trace!("Command::get(…)");

    if !self.mode.is_read() {
      return Err(Error::UnsupportedMode(format!("Address 0x{:04X} does not support reading.", self.addr)))
    }

    let block_len = self.block_len;
    let byte_len = self.byte_len;
    let byte_pos = self.byte_pos;

    let mut buf = vec![0; block_len];
    P::get(o, &self.addr(), &mut buf)?;

    if let Some(bit_pos) = self.bit_pos {
      let byte = buf[bit_pos / 8];
      let bit_len = self.bit_len.unwrap_or(1);

      buf.clear();
      buf.push((byte << (bit_pos % 8)) >> (8 - bit_len));
    }

    self.unit.bytes_to_output(&buf[byte_pos..(byte_pos + byte_len)], self.factor, &self.mapping)
  }

  pub fn set<P: Protocol>(&self, o: &mut Optolink, input: &Value) -> Result<(), Error> {
    log::trace!("Command::set(…)");

    if !self.mode.is_write() {
      return Err(Error::UnsupportedMode(format!("Address 0x{:04X} does not support writing.", self.addr)))
    }

    P::set(o, &self.addr(), &self.unit.input_to_bytes(input, self.factor, &self.mapping)?).map_err(Into::into)
  }
}
