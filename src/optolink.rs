use std::ffi::OsStr;
use std::io::{self, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use serial_core::{SerialDevice, SerialPortSettings, BaudRate::Baud4800, Parity::ParityEven, StopBits::Stop2, CharSize::Bits8};
use serial;

trait ReadWrite: Read + Write {}
impl<T> ReadWrite for T where T: Read + Write {}

pub struct OptoLink {
  device: Box<ReadWrite>,
}

impl OptoLink {
  const TIMEOUT: Duration = Duration::from_secs(10);
}

impl OptoLink {
  pub fn open(port: impl AsRef<OsStr>) -> Result<OptoLink, io::Error> {
    let mut tty = serial::open(&port)?;

    tty.set_timeout(Self::TIMEOUT)?;

    let mut tty_settings = tty.read_settings()?;

    tty_settings.set_baud_rate(Baud4800)?;
    tty_settings.set_parity(ParityEven);
    tty_settings.set_stop_bits(Stop2);
    tty_settings.set_char_size(Bits8);

    tty.write_settings(&tty_settings)?;

    Ok(OptoLink { device: Box::new(tty) })
  }

  pub fn connect(addr: impl ToSocketAddrs) -> Result<OptoLink, io::Error> {
    let stream = TcpStream::connect(addr)?;
    stream.set_read_timeout(Some(Self::TIMEOUT))?;
    Ok(OptoLink { device: Box::new(stream) })
  }
}

impl Write for OptoLink {
  fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
    self.device.write(buf)
  }

  fn flush(&mut self) -> Result<(), io::Error> {
    self.device.flush()
  }
}

impl Read for OptoLink {
  fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
    self.device.read(buf)
  }
}
