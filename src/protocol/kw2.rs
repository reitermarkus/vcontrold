use std::io::{self, Read, Write};

use crate::Optolink;

use super::Protocol;

pub struct Kw2;

impl Kw2 {
  fn sync(o: &mut Optolink) -> Result<(), std::io::Error> {
    o.purge()?;

    o.write(&[0x04])?;
    o.flush()?;

    let mut buf = [0xff];
    o.read_exact(&mut buf)?;

    if buf != [0x05] {
      return Err(std::io::Error::new(std::io::ErrorKind::Other, "sync failed"))
    }

    o.purge()?;

    Ok(())
  }
}

impl Protocol for Kw2 {
  fn get(o: &mut Optolink, addr: &[u8], buf: &mut [u8]) -> Result<(), io::Error> {
    for _ in 0..2 {
      Self::sync(o)?;

      o.write(&[0x01, 0xf7])?;
      o.write(addr)?;
      o.purge()?;
      o.write(&[buf.len() as u8])?;
      o.flush()?;

      o.read_exact(buf)?;

      println!("{:?}", buf);

      // Retry once if the response only contains `0x05`,
      // since these could be synchronization bytes.
      if buf.iter().all(|byte| *byte == 0x05) {
        eprintln!("{:?} -> retrying", buf);
        continue
      } else {
        break
      }
    }

    Ok(())
  }

  fn set(o: &mut Optolink, addr: &[u8], value: &[u8]) -> Result<(), io::Error> {
    Self::sync(o)?;

    o.write(&[0x01, 0xf4])?;
    o.write(addr)?;
    o.write(&[value.len() as u8])?;
    o.purge()?;
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
