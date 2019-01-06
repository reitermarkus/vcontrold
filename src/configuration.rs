use std::collections::HashMap;
use std::io::{self, Read, BufReader};
use std::fs::File;
use std::str::FromStr;
use std::path::{Path, PathBuf};

use serde_derive::*;
use serde_yaml;
use yaml_merge_keys;

use crate::{Command, Optolink};


#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Device {
  Tty { path: PathBuf },
  Tcp { host: String, port: u16 },
}

#[derive(Debug, Deserialize)]
pub struct Configuration {
  device: Device,
  commands: HashMap<String, Command>,
}

impl FromStr for Configuration {
  type Err = String;

  fn from_str(s: &str) -> Result<Configuration, Self::Err> {
    let value: serde_yaml::Value = serde_yaml::from_str(&s).map_err(|err| err.to_string())?;

    let value = yaml_merge_keys::merge_keys_serde(value).map_err(|err| err.kind().description().to_string())?;

    serde_yaml::from_value(value).map_err(|err| err.to_string())
  }
}

impl Configuration {
  pub fn open(path: impl AsRef<Path>) -> Result<Configuration, io::Error> {
    let file = File::open(path.as_ref())?;

    let mut content = String::new();
    BufReader::new(file).read_to_string(&mut content)?;

    content.parse().map_err(|err| io::Error::new(io::ErrorKind::Other, err))
  }

  pub fn device(&self) -> Result<Optolink, io::Error> {
    match &self.device {
      Device::Tty { path } => Optolink::open(path),
      Device::Tcp { host, port } => Optolink::connect((host.as_str(), *port)),
    }
  }

  pub fn command(&self, command: &str) -> &Command {
    &self.commands[command]
  }

  pub fn commands(&self) -> HashMap<String, Command> {
    self.commands.clone()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn config() {
    let config = include_str!("../config/default.yml").parse::<Configuration>();
    assert!(config.is_ok());
  }
}
