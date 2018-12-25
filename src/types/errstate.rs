use std::fmt;
use std::str::FromStr;

use crate::FromBytes;

use super::SysTime;

byte_type!(ErrState, 9);

impl ErrState {
  pub fn id(&self) -> &[u8] {
    &self.0[0..1]
  }

  pub fn time(&self) -> SysTime {
    SysTime::from_bytes(&self.0[1..9])
  }
}

impl FromStr for ErrState {
  type Err = String;

  fn from_str(_: &str) -> Result<ErrState, Self::Err> {
    Err("parsing ErrState is not supported")?
  }
}

impl fmt::Display for ErrState {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:02X} ({})", self.0[0], self.time())
  }
}
