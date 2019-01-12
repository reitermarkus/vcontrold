use phf;

use crate::{Error, Command, Optolink, types::Bytes, protocol::*, Value};

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

pub trait Device {
  type Protocol: Protocol;

  fn map() -> &'static phf::Map<&'static str, Command>;

  fn commands() -> Vec<&'static str> {
    Self::map().keys().map(|s| *s).collect::<Vec<_>>()
  }

  fn command(name: &str) -> Option<&Command> {
    Self::map().get(name)
  }

  fn get(o: &mut Optolink, cmd: &Command) -> Result<Value, Error> {
    cmd.get::<Self::Protocol>(o)
  }

  fn set(o: &mut Optolink, cmd: &Command, input: &Value) -> Result<(), Error> {
    cmd.set::<Self::Protocol>(o, input)
  }
}
