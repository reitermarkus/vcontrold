use std::io::{self, Read, Write};
use std::time::Instant;

use crate::Optolink;

use super::Protocol;

const LEADIN: u8    = 0x41;

const RESET: u8     = 0x04;
const SYNC: u8      = 0x05;
const ACK: u8       = 0x06;
const NACK: u8      = 0x15;

const REQUEST: u8   = 0x00;
const RESPONSE: u8  = 0x01;

const READDATA: u8  = 0x01;
const WRITEDATA: u8 = 0x02;

#[derive(Debug)]
pub struct P300;

impl P300 {
  #[inline]
  fn write_telegram(o: &mut Optolink, message: &[u8]) -> Result<(), std::io::Error> {
    let message_length = message.len() as u8;
    let checksum: u8 = message.iter().sum();
    let checksum = checksum + message_length;

    let start = Instant::now();

    loop {
      o.write_all(&[LEADIN])?;
      o.write_all(&[message_length])?;
      o.write_all(&message)?;
      o.write_all(&[checksum])?;
      o.flush()?;

      let mut status = [0xff];
      o.read_exact(&mut status)?;
      match status {
        [ACK] => return Ok(()),
        [NACK] => {}
        [_] => return Err(io::Error::new(io::ErrorKind::InvalidData, "send telegram failed")),
      }

      let stop = Instant::now();

      if (stop - start) > Optolink::TIMEOUT {
        break;
      }
    }

    Err(io::Error::new(io::ErrorKind::TimedOut, "send telegram timed out"))
  }

  #[inline]
  fn read_telegram(o: &mut Optolink) -> Result<Vec<u8>, std::io::Error> {
    let mut buf = [0xff];

    let start = Instant::now();

    loop {
      o.read_exact(&mut buf)?;
      if buf != [LEADIN] {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "telegram leadin expected"))
      }

      o.read_exact(&mut buf)?;
      let message_length: usize = buf[0] as usize;

      let mut message = vec![0; message_length];
      o.read_exact(&mut message)?;
      let message = message;

      let checksum: u8 = message.iter().sum();
      let checksum = checksum + message_length as u8;

      o.read_exact(&mut buf)?;
      if checksum == buf[0] {
        o.write_all(&[ACK])?;
        o.flush()?;
        return Ok(message)
      }

      o.write_all(&[NACK])?;
      o.flush()?;

      let stop = Instant::now();

      if (stop - start) > Optolink::TIMEOUT {
        break;
      }
    }

    Err(io::Error::new(io::ErrorKind::TimedOut, "send telegram timed out"))
  }
}

impl Protocol for P300 {
  fn negotiate(o: &mut Optolink) -> Result<(), io::Error> {
    o.write_all(&[RESET])?;
    o.flush()?;

    let mut status = [0xff];

    let start = Instant::now();

    loop {
      let stop = Instant::now();

      if (stop - start) > Optolink::TIMEOUT {
        break;
      }

      o.read_exact(&mut status)?;
      match status {
        [SYNC] => {},
        _ => continue,
      }

      o.write_all(&[0x16, 0x00, 0x00])?;
      o.flush()?;

      o.read_exact(&mut status)?;
      match status {
        [ACK] => return Ok(()),
        [NACK] => {}
        _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "protocol negotiation failed")),
      }
    }

    Err(io::Error::new(io::ErrorKind::TimedOut, "negotiate timed out"))
  }

  fn get(o: &mut Optolink, addr: &[u8], buf: &mut [u8]) -> Result<(), io::Error> {
    let mut read_request = Vec::new();
    read_request.extend(&[REQUEST, READDATA]);
    read_request.extend(addr);
    read_request.extend(&[buf.len() as u8]);

    Self::write_telegram(o, &read_request)?;

    let read_response = Self::read_telegram(o)?;

    let expected_len: usize = 5 + buf.len();
    if expected_len != read_response.len() {
      return Err(io::Error::new(io::ErrorKind::InvalidData, "unexpected response length"))
    }
    if [RESPONSE, READDATA] != read_response[0..2] {
      return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid read data response"))
    }
    if *addr != read_response[2..4] {
      return Err(io::Error::new(io::ErrorKind::InvalidData, "wrong address"))
    }
    if buf.len() as u8 != read_response[4] {
      return Err(io::Error::new(io::ErrorKind::InvalidData, "wrong data length"))
    }

    buf.clone_from_slice(&read_response[5..(5 + buf.len())]);

    Ok(())
  }

  fn set(o: &mut Optolink, addr: &[u8], value: &[u8]) -> Result<(), io::Error> {
    let mut write_request = Vec::new();
    write_request.extend(&[REQUEST, WRITEDATA]);
    write_request.extend(addr);
    write_request.extend(&[value.len() as u8]);
    write_request.extend(value);

    Self::write_telegram(o, &write_request)?;

    let write_response = Self::read_telegram(o)?;

    let expected_len: usize = 5;
    if expected_len != write_response.len() {
      return Err(io::Error::new(io::ErrorKind::InvalidData, "unexpected response length"))
    }
    if [RESPONSE, WRITEDATA] != write_response[0..2] {
      return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid write data response"))
    }
    if *addr != write_response[2..4] {
      return Err(io::Error::new(io::ErrorKind::InvalidData, "wrong address"))
    }
    if value.len() as u8 != write_response[5] {
      return Err(io::Error::new(io::ErrorKind::InvalidData, "could not write data"))
    }

    Ok(())
  }
}
