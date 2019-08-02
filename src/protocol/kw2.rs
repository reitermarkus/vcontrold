use std::io::{self, Read, Write};
use std::time::{Instant, Duration};

use crate::Optolink;

use super::Protocol;

const RESET: u8 = 0x04;
const SYNC: u8  = 0x05;

#[derive(Debug)]
pub struct Kw2;

impl Kw2 {
  fn sync(o: &mut Optolink) -> Result<(), std::io::Error> {
    log::trace!("Kw2::sync(…)");

    let mut buf = [0xff];

    let start = Instant::now();

    // Reset the Optolink connection to get a faster SYNC (`0x05`).
    Self::negotiate(o)?;

    loop {
      log::trace!("Kw2::sync(…) loop");

      if o.read_exact(&mut buf).is_ok() && buf == [SYNC] {
        o.purge()?;
        return Ok(())
      }

      let stop = Instant::now();

      if (stop - start) > Optolink::TIMEOUT {
        break
      }
    }

    Err(io::Error::new(io::ErrorKind::TimedOut, "sync timed out"))
  }
}

impl Protocol for Kw2 {
  fn negotiate(o: &mut Optolink) -> Result<(), io::Error> {
    log::trace!("Kw2::negotiate(…)");

    o.purge()?;
    o.write_all(&[RESET])?;
    o.flush()?;

    Ok(())
  }

  fn get(o: &mut Optolink, addr: &[u8], buf: &mut [u8]) -> Result<(), io::Error> {
    log::trace!("Kw2::get(…)");

    let mut vec = Vec::new();
    vec.extend(&[0x01, 0xf7]);
    vec.extend(addr);
    vec.extend(&[buf.len() as u8]);

    let start = Instant::now();

    Self::sync(o)?;

    loop {
      log::trace!("Kw2::get(…) loop");

      o.write_all(&vec)?;
      o.flush()?;

      let read_start = Instant::now();

      o.read_exact(buf)?;

      let stop = Instant::now();

      // Retry if the response contains `SYNC` (`0x05`),
      // since these could be synchronization bytes.
      if buf.iter().any(|byte| *byte == SYNC) {
        let read_time = stop - read_start;

        log::debug!("Kw2::get(…) buf = {}", buf.iter().map(|b| format!("{:02X}", b)).collect::<Vec<String>>().join(" "));
        log::debug!("Kw2::get(…) read_time = {:?}", read_time);

        // Return `Ok` if the response was received in a short amount of time,
        // since then they most likely are not synchronization bytes.
        if read_time < Duration::from_millis(500 * buf.len() as u64) {
          return Ok(())
        }

        o.purge()?;
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
    log::trace!("Kw2::set(…)");

    let mut vec = Vec::new();
    vec.extend(&[0x01, 0xf4]);
    vec.extend(addr);
    vec.extend(&[value.len() as u8]);
    vec.extend(value);

    let start = Instant::now();

    Self::sync(o)?;

    loop {
      o.write_all(&vec)?;
      o.flush()?;

      let mut buf = [0xff];
      o.read_exact(&mut buf)?;

      let stop = Instant::now();

      if buf == [0x00] {
        return Ok(())
      }

      if (stop - start) > Optolink::TIMEOUT {
        break
      }
    }

    Err(io::Error::new(io::ErrorKind::TimedOut, "set timed out"))
  }
}
