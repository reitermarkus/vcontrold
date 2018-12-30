use std::io::{self, Read, Write};
use std::time::{Instant, Duration};

use crate::Optolink;

use super::Protocol;

#[derive(Debug)]
pub struct Kw2;

impl Kw2 {
  #[inline]
  fn sync(o: &mut Optolink) -> Result<(), std::io::Error> {
    let mut buf = [0xff];

    let start = Instant::now();

    loop {
      o.write_all(&[0x04])?;
      o.flush()?;

      if o.read_exact(&mut buf).is_ok() {
        if buf == [0x05] {
          o.purge()?;
          return Ok(())
        }
      }

      let stop = Instant::now();

      if (stop - start) > Optolink::TIMEOUT {
        break
      }
    }

    Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "sync timed out"))
  }
}

impl Protocol for Kw2 {
  fn get(o: &mut Optolink, addr: &[u8], buf: &mut [u8]) -> Result<(), io::Error> {
    let mut vec = Vec::new();
    vec.extend(&[0x01, 0xf7]);
    vec.extend(addr);
    vec.extend(&[buf.len() as u8]);

    let start = Instant::now();

    Self::sync(o)?;

    loop {
      o.write_all(&vec)?;
      o.flush()?;

      o.read_exact(buf)?;

      let stop = Instant::now();

      // Retry if the response only contains `0x05`,
      // since these could be synchronization bytes.
      if buf.iter().all(|byte| *byte == 0x05) {
        // Return `Ok` if they were received in a short amount of time,
        // since then they most likely are not synchronization bytes.
        if (stop - start) < Duration::from_millis(500 * buf.len() as u64) {
          return Ok(())
        }

        o.purge()?;

        eprintln!("{:?} -> retrying", buf);
      } else {
        return Ok(())
      }

      if (stop - start) > Optolink::TIMEOUT {
        break
      }
    }

    Err(io::Error::new(io::ErrorKind::TimedOut, "get timed out"))
  }

  fn set(o: &mut Optolink, addr: &[u8], value: &[u8]) -> Result<(), io::Error> {
    let mut vec = Vec::new();
    vec.extend(&[0x01, 0xf4]);
    vec.extend(addr);
    vec.extend(&[value.len() as u8]);

    Self::sync(o)?;
    o.purge()?;

    o.write_all(&vec)?;
    o.flush()?;

    let mut buf = [0xff];
    o.read_exact(&mut buf)?;

    if buf == [0x00] {
      return Ok(())
    }

    Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "set failed"))
  }
}
