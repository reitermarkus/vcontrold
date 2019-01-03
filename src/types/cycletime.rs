use std::fmt;
use std::str::FromStr;

use serde::ser::{Serialize, Serializer};
use serde::de::{self, Deserialize, Deserializer};
use serde_derive::*;

byte_type!(CycleTime, 8);

impl CycleTime {
  fn byte_to_time(&self, i: usize) -> Option<(u8, u8)> {
    match self.0[i] {
      0xff => None,
      byte => Some((byte >> 3, (byte & 0b111) * 10)),
    }
  }

  fn times(&self) -> [TimeSpan; 4] {
    [
      TimeSpan { from: self.byte_to_time(0).into(), to: self.byte_to_time(1).into() },
      TimeSpan { from: self.byte_to_time(2).into(), to: self.byte_to_time(3).into() },
      TimeSpan { from: self.byte_to_time(4).into(), to: self.byte_to_time(5).into() },
      TimeSpan { from: self.byte_to_time(6).into(), to: self.byte_to_time(7).into() },
    ]
  }
}

#[derive(Serialize, Clone)]
struct TimeSpan {
  from: Time,
  to: Time,
}

impl fmt::Display for TimeSpan {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{} – {}", self.from, self.to)
  }
}

#[derive(Serialize, Clone)]
struct Time {
  hh: String,
  mm: String,
}

impl fmt::Display for Time {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}:{}", self.hh, self.mm)
  }
}

impl From<Option<(u8, u8)>> for Time {
  fn from(tuple: Option<(u8, u8)>) -> Self {
    if let Some((hh, mm)) = tuple {
      Time { hh: format!("{:02}", hh), mm: format!("{:02}", mm) }
    } else {
      Time { hh: "--".into(), mm: "--".into() }
    }
  }
}

impl Serialize for CycleTime {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    #[derive(Serialize)]
    struct TimeSpanFull {
      full: String,
      from: TimeFull,
      to: TimeFull,
    }

    #[derive(Serialize)]
    struct TimeFull {
      full: String,
      hh: String,
      mm: String,
    }

    self.times().into_iter()
      .map(|ts|
        TimeSpanFull {
          full: ts.to_string(),
          from: TimeFull {
            full: ts.from.to_string(),
            hh: ts.from.hh.to_owned(),
            mm: ts.from.mm.to_owned(),
          },
          to: TimeFull {
            full: ts.to.to_string(),
            hh: ts.to.hh.to_owned(),
            mm: ts.to.mm.to_owned(),
          },
      })
      .collect::<Vec<TimeSpanFull>>()
      .serialize(serializer)
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
      self.times().into_iter().map(|timespan| timespan.to_string()).collect::<Vec<String>>().join(","),
    )
  }
}
