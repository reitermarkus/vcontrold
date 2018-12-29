use std::io;

use crate::Optolink;

pub trait Protocol {
  /// Reads the value at the address `addr` into `buf`.
  fn get(o: &mut Optolink, addr: &[u8], buf: &mut [u8]) -> Result<(), io::Error>;

  /// Writes the given value `value` to the the address `addr`.
  fn set(o: &mut Optolink, addr: &[u8], value: &[u8]) -> Result<(), io::Error>;
}

mod kw2;
pub use self::kw2::Kw2;
