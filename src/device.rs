use phf;

use crate::{Error, Command, Optolink, Protocol, Value};

#[allow(clippy::unreadable_literal)]
mod codegen {
  use super::*;
  use crate::types::Bytes;
  use crate::protocol::*;

  include!(concat!(env!("OUT_DIR"), "/codegen.rs"));
}

pub use self::codegen::*;

pub trait Device {
  type Protocol: Protocol;

  fn map() -> &'static phf::Map<&'static str, Command>;

  fn commands() -> Vec<&'static str> {
    Self::map().keys().cloned().collect::<Vec<_>>()
  }

  fn command(name: &str) -> Option<&Command> {
    Self::map().get(name)
  }

  fn get(o: &mut Optolink, cmd: &Command) -> Result<Value, Error> {
    log::trace!("Device::get(…)");

    cmd.get::<Self::Protocol>(o)
  }

  fn set(o: &mut Optolink, cmd: &Command, input: &Value) -> Result<(), Error> {
    log::trace!("Device::set(…)");

    cmd.set::<Self::Protocol>(o, input)
  }
}
