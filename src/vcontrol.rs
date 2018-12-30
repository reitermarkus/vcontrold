use std::collections::HashMap;
use std::io;

use crate::{Configuration, Command, Optolink, protocol::Kw2, Value};

#[derive(Debug)]
pub struct VControl {
  device: Optolink,
  commands: HashMap<String, Command>,
}

impl VControl {
  pub fn new(device: Optolink, commands: HashMap<String, Command>) -> VControl {
    VControl { device, commands }
  }

  pub fn from_config(config: Configuration) -> Result<VControl, io::Error> {
    Ok(VControl { device: config.device()?, commands: config.commands() })
  }

  /// Gets the value for the given command.
  ///
  /// If the command specified is not available, an IO error of the kind `AddrNotAvailable` is returned.
  pub fn get(&mut self, command: &str) -> Result<Value, io::Error> {
    let command = if let Some(command) = self.commands.get(command) {
      command
    } else {
      return Err(io::Error::new(io::ErrorKind::AddrNotAvailable, format!("no such command: {}", command)))
    };

    command.get::<Kw2>(&mut self.device)
  }

  /// Sets the value for the given command.
  ///
  /// If the command specified is not available, an IO error of the kind `AddrNotAvailable` is returned.
  pub fn set(&mut self, command: &str, input: &str) -> Result<(), io::Error> {
    let command = if let Some(command) = self.commands.get(command) {
      command
    } else {
      return Err(io::Error::new(io::ErrorKind::AddrNotAvailable, format!("no such command: {}", command)))
    };

    command.set::<Kw2>(&mut self.device, input)
  }
}
