use std::io;

use crate::Optolink;

pub trait Protocol {
  fn get(o: &mut Optolink, addr: &[u8], buf: &mut [u8]) -> Result<(), io::Error>;
  fn set(o: &mut Optolink, addr: &[u8], value: &[u8]) -> Result<(), io::Error>;
}

mod kw2;
pub use self::kw2::Kw2;
