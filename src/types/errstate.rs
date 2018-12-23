use std::fmt;
use std::str::FromStr;

use crate::{FromBytes, AsBytes};
use crate::types::SysTime;

#[derive(Debug)]
pub struct ErrState([u8; 9]);

impl ErrState {
  pub fn id(&self) -> u8 {
    self.0[0]
  }

  pub fn time(&self) -> SysTime {
    SysTime::from_bytes(&self.0[1..9])
  }
}

impl FromStr for ErrState {
  type Err = String;

  fn from_str(s: &str) -> Result<ErrState, Self::Err> {
    unimplemented!("ErrState::from_str")
  }
}

impl FromBytes for ErrState {
  fn from_bytes(bytes: &[u8]) -> ErrState {
    assert_eq!(bytes.len(), std::mem::size_of::<ErrState>());
    let mut buf = [0; std::mem::size_of::<ErrState>()];
    buf.copy_from_slice(&bytes);
    ErrState(buf)
  }
}

impl AsBytes for ErrState {
  fn as_bytes(&self) -> &[u8] {
    &self.0
  }
}

impl fmt::Display for ErrState {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:02X} ({})", self.0[0], self.time())
  }
}
