use std::fmt;
use std::str::FromStr;

use chrono::{NaiveDate, NaiveDateTime, Datelike, Timelike};

#[inline]
fn byte_to_dec(byte: u8) -> u8 {
  byte / 16 * 10 + byte % 16
}

#[inline]
fn dec_to_byte(dec: u8) -> u8 {
  dec / 10 * 16 + dec % 10
}

byte_type!(SysTime, 8);

impl SysTime {
  pub fn new(year: u16, month: u8, day: u8, hour: u8, minute: u8, second: u8) -> SysTime {
    NaiveDate::from_ymd(year as i32, month as u32, day as u32).and_hms(hour as u32, minute as u32, second as u32).into()
  }

  pub fn year(&self) -> u16 {
    byte_to_dec(self.0[0]) as u16 * 100 + byte_to_dec(self.0[1]) as u16
  }

  pub fn month(&self) -> u8 {
    byte_to_dec(self.0[2])
  }

  pub fn day(&self) -> u8 {
    byte_to_dec(self.0[3])
  }

  pub fn weekday(&self) -> u8 {
    self.0[4]
  }

  pub fn hour(&self) -> u8 {
    byte_to_dec(self.0[5])
  }

  pub fn minute(&self) -> u8 {
    byte_to_dec(self.0[6])
  }

  pub fn second(&self) -> u8 {
    byte_to_dec(self.0[7])
  }
}

impl From<SysTime> for NaiveDateTime {
  fn from(systime: SysTime) -> NaiveDateTime {
    NaiveDate::from_ymd(
      systime.year() as i32,
      systime.month() as u32,
      systime.day() as u32,
    ).and_hms(
      systime.hour() as u32,
      systime.minute() as u32,
      systime.second() as u32,
    )
  }
}

impl From<NaiveDateTime> for SysTime {
  fn from(datetime: NaiveDateTime) -> SysTime {
    SysTime([
      dec_to_byte((datetime.year() / 100) as u8),
      dec_to_byte((datetime.year() % 100) as u8),
      dec_to_byte(datetime.month() as u8),
      dec_to_byte(datetime.day() as u8),
      datetime.weekday().number_from_monday() as u8,
      dec_to_byte(datetime.hour() as u8),
      dec_to_byte(datetime.minute() as u8),
      dec_to_byte(datetime.second() as u8),
    ])
  }
}

impl FromStr for SysTime {
  type Err = String;

  fn from_str(s: &str) -> Result<SysTime, Self::Err> {
    NaiveDateTime::from_str(s).map(Into::into).map_err(|err| err.to_string())
  }
}

impl fmt::Display for SysTime {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
      self.year(),
      self.month(),
      self.day(),
      self.hour(),
      self.minute(),
      self.second(),
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn new() {
    let time = SysTime::new(2018, 12, 23, 17, 49, 31);

    assert_eq!(time.year(), 2018);
    assert_eq!(time.month(), 12);
    assert_eq!(time.day(), 23);
    assert_eq!(time.weekday(), 7);
    assert_eq!(time.hour(), 17);
    assert_eq!(time.minute(), 49);
    assert_eq!(time.second(), 31);
  }

  #[test]
  fn from_str() {
    let time = SysTime::from_str("2018-12-23T17:49:31").unwrap();

    assert_eq!(time.year(), 2018);
    assert_eq!(time.month(), 12);
    assert_eq!(time.day(), 23);
    assert_eq!(time.weekday(), 7);
    assert_eq!(time.hour(), 17);
    assert_eq!(time.minute(), 49);
    assert_eq!(time.second(), 31);
  }

  #[test]
  fn from_bytes() {
    let time = SysTime::from_bytes(&[0x20, 0x18, 0x12, 0x23, 0x07, 0x17, 0x49, 0x31]);

    assert_eq!(time.year(), 2018);
    assert_eq!(time.month(), 12);
    assert_eq!(time.day(), 23);
    assert_eq!(time.weekday(), 7);
    assert_eq!(time.hour(), 17);
    assert_eq!(time.minute(), 49);
    assert_eq!(time.second(), 31);
  }

  #[test]
  fn to_bytes() {
    let time = SysTime::new(2018, 12, 23, 17, 49, 31);
    assert_eq!(time.to_bytes(), [0x20, 0x18, 0x12, 0x23, 0x07, 0x17, 0x49, 0x31]);
  }
}
