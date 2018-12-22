use std::fmt;
use std::str::FromStr;

use crate::{FromBytes, AsBytes};

#[derive(Debug)]
pub struct CycleTime([u8; 8]);

impl CycleTime {
  fn byte_to_time(&self, i: usize) -> Option<(u8, u8)> {
    match self.0[i] {
      0xff => None,
      byte => Some((byte >> 3, (byte & 0b111) * 10)),
    }
  }

  pub fn times(&self) -> [Option<((u8, u8), (u8, u8))>; 4] {
    [
      self.byte_to_time(0).and_then(|from| self.byte_to_time(1).map(|to| (from, to))),
      self.byte_to_time(2).and_then(|from| self.byte_to_time(3).map(|to| (from, to))),
      self.byte_to_time(4).and_then(|from| self.byte_to_time(5).map(|to| (from, to))),
      self.byte_to_time(6).and_then(|from| self.byte_to_time(7).map(|to| (from, to))),
    ]
  }
}

impl FromStr for CycleTime {
  type Err = String;

  fn from_str(s: &str) -> Result<CycleTime, Self::Err> {
    unimplemented!("CycleTime::from_str")
  }
}

impl FromBytes for CycleTime {
  fn from_bytes(bytes: &[u8]) -> CycleTime {
    assert_eq!(bytes.len(), std::mem::size_of::<CycleTime>());
    let mut buf = [0; std::mem::size_of::<CycleTime>()];
    buf.copy_from_slice(&bytes);
    CycleTime(buf)
  }
}

impl AsBytes for CycleTime {
  fn as_bytes(&self) -> &[u8] {
    &self.0
  }
}

impl fmt::Display for CycleTime {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}",
      self.times().into_iter().map(|o| o.map(|((from_h, from_m), (to_h, to_m))| format!("{:02}:{:02}-{:02}:{:02}", from_h, from_m, to_h, to_m)).unwrap_or("".into())).collect::<Vec<String>>().join(","),
    )
  }
}
