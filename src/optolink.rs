use std::ffi::OsStr;
use std::io::{self, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use serial_core::{SerialDevice, SerialPortSettings, BaudRate::Baud4800, Parity::ParityEven, StopBits::Stop2, CharSize::Bits8};
use serial::{self, SystemPort};
pub struct OptoLink<T> {
  pub device: T,
}

impl<T> OptoLink<T> {
  const TIMEOUT: Duration = Duration::from_secs(10);
}

impl OptoLink<SystemPort> {
  pub fn open(port: impl AsRef<OsStr>) -> Result<OptoLink<SystemPort>, io::Error> {
    let mut tty = serial::open(&port)?;

    tty.set_timeout(Self::TIMEOUT)?;

    let mut tty_settings = tty.read_settings()?;

    tty_settings.set_baud_rate(Baud4800)?;
    tty_settings.set_parity(ParityEven);
    tty_settings.set_stop_bits(Stop2);
    tty_settings.set_char_size(Bits8);

    tty.write_settings(&tty_settings)?;

    Ok(OptoLink { device: tty })
  }
}

impl OptoLink<TcpStream> {
  pub fn connect(addr: impl ToSocketAddrs) -> Result<OptoLink<TcpStream>, io::Error> {
    let stream = TcpStream::connect(addr)?;
    stream.set_read_timeout(Some(Self::TIMEOUT))?;
    Ok(OptoLink { device: stream })
  }
}

impl<T> Write for OptoLink<T> where T: Write {
  fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
    self.device.write(buf)
  }

  fn flush(&mut self) -> Result<(), io::Error> {
    self.device.flush()
  }
}

impl<T> Read for OptoLink<T> where T: Read {
  fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
    self.device.read(buf)
  }
}
