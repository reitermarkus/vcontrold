use std::fmt;
use std::str::FromStr;

use serde::ser::{Serialize, Serializer};
use serde::de::{self, Deserialize, Deserializer};

byte_type!(CycleTime, 8);

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

impl Serialize for CycleTime {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    self.times().serialize(serializer)
  }
}

impl FromStr for CycleTime {
  type Err = String;

  fn from_str(s: &str) -> Result<CycleTime, Self::Err> {
    Err(format!("could not parse {}, from_str is not implemented for CycleTime", s))
  }
}

impl<'de> Deserialize<'de> for CycleTime {
  fn deserialize<D>(deserializer: D) -> Result<CycleTime, D::Error>
  where
      D: Deserializer<'de>,
  {
    let string = String::deserialize(deserializer)?;
    CycleTime::from_str(&string).map_err(|err| de::Error::custom(err))
  }
}

impl fmt::Display for CycleTime {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}",
      self.times().into_iter().map(|o| o.map(|((from_h, from_m), (to_h, to_m))| format!("{:02}:{:02}-{:02}:{:02}", from_h, from_m, to_h, to_m)).unwrap_or("".into())).collect::<Vec<String>>().join(","),
    )
  }
}
