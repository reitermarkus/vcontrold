use std::io::{self, Read, Write};

use crate::{OptoLink, Protocol};

pub struct Kw2;

impl Kw2 {
  fn sync(o: &mut OptoLink) -> Result<(), std::io::Error> {
    o.write(&[0x04])?;
    o.flush()?;

    let mut buf = [0xff];
    o.read_exact(&mut buf)?;

    if buf != [0x05] {
      return Err(std::io::Error::new(std::io::ErrorKind::Other, "sync failed"))
    }

    Ok(())
  }
}

impl Protocol for Kw2 {
  fn get(o: &mut OptoLink, addr: &[u8], buf: &mut [u8]) -> Result<(), io::Error> {
    Self::sync(o)?;

    o.write(&[0x01, 0xf7])?;
    o.write(addr)?;
    o.write(&[buf.len() as u8])?;
    o.flush()?;

    o.read_exact(buf)?;

    Ok(())
  }

  fn set(o: &mut OptoLink, addr: &[u8], value: &[u8]) -> Result<(), io::Error> {
    Self::sync(o)?;

    o.write(&[0x01, 0xf4])?;
    o.write(addr)?;
    o.write(&[value.len() as u8])?;
    o.write(value)?;
    o.flush()?;

    let mut buf = [0xff];
    o.read_exact(&mut buf)?;

    if buf != [0x00] {
      return Err(std::io::Error::new(std::io::ErrorKind::Other, "set failed"))
    }

    Ok(())
  }
}
