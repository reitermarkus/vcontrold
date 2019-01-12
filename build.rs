use std::env;
use std::fs::File;
use std::io::{Read, BufReader, BufWriter, Write};
use std::path::Path;
use std::collections::HashMap;
use std::fmt;

use serde_derive::*;
use serde_yaml;
use yaml_merge_keys;
use phf_codegen;
use serde::de::{self, Deserialize, Deserializer};

#[path = "src/types/mod.rs"]
mod types;
use self::types::*;

fn main() {
  let device = "V200KW2_6";

  let config_path = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("config").join(format!("{}.yml", device));

  let file = File::open(config_path).unwrap();

  let mut content = String::new();
  BufReader::new(file).read_to_string(&mut content).unwrap();

  let config: Configuration = serde_yaml::from_value(yaml_merge_keys::merge_keys_serde(serde_yaml::from_str::<serde_yaml::Value>(&content).unwrap()).unwrap()).unwrap();

  let path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen.rs");
  let mut file = BufWriter::new(File::create(&path).unwrap());

  let mut map = phf_codegen::Map::new();

  for (name, command) in config.commands {
    map.entry(name, &format!("{:?}", command));
  }

  write!(&mut file, "static {}_COMMANDS: phf::Map<&'static str, Command> =", device).unwrap();

  map.build(&mut file).unwrap();

  write!(&mut file, ";\n").unwrap();

  write!(&mut file, "
    #[derive(Debug)]
    pub enum {} {{}}

    impl Device for {} {{
      type Protocol = Kw2;

      #[inline(always)]
      fn map() -> &'static phf::Map<&'static str, Command> {{
        &{}_COMMANDS
      }}
    }}
  ", device, device, device).unwrap();

  write!(&mut file, "pub type V200KW2 = V200KW2_6;").unwrap();
}

#[derive(Debug, Deserialize)]
pub struct Configuration {
  pub commands: HashMap<String, Command>,
}

/// A command which can be executed on an Optolink connection.
#[derive(Deserialize)]
pub struct Command {
  addr: u16,
  mode: AccessMode,
  unit: Unit,
  byte_pos: Option<usize>,
  byte_len: Option<usize>,
  bit_pos: Option<usize>,
  bit_len: Option<usize>,
  factor: Option<f64>,
  mapping: Option<HashMap<Vec<u8>, String>>,
}

impl fmt::Debug for Command {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let byte_pos = self.byte_pos.unwrap_or(0);
    let byte_len = self.byte_len.unwrap_or_else(|| self.unit.size());

    let mapping = if let Some(mapping) = &self.mapping {
      let mut map = phf_codegen::Map::new();

      for (k, v) in mapping {
        map.entry(Bytes::from_bytes(k), &format!("{:?}", v));
      }

      let mut buf = Vec::new();
      map.build(&mut buf).unwrap();

      format!("Some({})", String::from_utf8(buf).unwrap())
    } else {
      "None".into()
    };

    f.debug_struct("Command")
       .field("addr", &format_args!("0x{:04X}", self.addr))
       .field("mode", &format_args!("crate::AccessMode::{:?}", self.mode))
       .field("unit", &format_args!("crate::Unit::{:?}", self.unit))
       .field("byte_pos", &byte_pos)
       .field("byte_len", &byte_len)
       .field("bit_len", &self.bit_len)
       .field("bit_pos", &self.bit_pos)
       .field("factor", &self.factor.unwrap_or(1.0))
       .field("mapping", &format_args!("{}", mapping))
       .finish()
  }
}

#[derive(Debug)]
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

#[derive(Debug)]
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
