use std::ffi::OsStr;
use std::io::{self, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use serial_core::{SerialPort, SerialPortSettings, BaudRate::Baud4800, Parity::ParityEven, StopBits::Stop2, CharSize::Bits8};
use serial::SystemPort;

enum Device {
  Tty(SystemPort),
  Stream(TcpStream),
}

pub struct OptoLink {
  device: Device,
}

impl OptoLink {
  const TIMEOUT: Duration = Duration::from_secs(10);

  pub fn open(port: impl AsRef<OsStr>) -> Result<OptoLink, io::Error> {
    let mut tty = serial::open(&port)?;

    tty.set_timeout(Self::TIMEOUT)?;

    tty.reconfigure(&|settings: &mut SerialPortSettings| -> Result<(), serial_core::Error> {
      settings.set_parity(ParityEven);
      settings.set_stop_bits(Stop2);
      settings.set_char_size(Bits8);
      settings.set_baud_rate(Baud4800)
    })?;

    Ok(OptoLink { device: Device::Tty(tty) })
  }

  pub fn connect(addr: impl ToSocketAddrs) -> Result<OptoLink, io::Error> {
    let stream = TcpStream::connect(addr)?;
    stream.set_read_timeout(Some(Self::TIMEOUT))?;
    Ok(OptoLink { device: Device::Stream(stream) })
  }

  pub fn purge(&mut self) -> Result<(), io::Error> {
    match &mut self.device {
      Device::Tty(tty) => { tty.set_timeout(Duration::new(0, 0))?; }
      Device::Stream(stream) => { stream.set_nonblocking(true)?; },
    }

    let mut buf = [0];
    while self.read_exact(&mut buf).is_ok() { }

    match &mut self.device {
      Device::Tty(tty) => { tty.set_timeout(Self::TIMEOUT)?; },
      Device::Stream(stream) => { stream.set_nonblocking(false)?; },
    }

    Ok(())
  }
}

impl Write for OptoLink {
  fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
    match &mut self.device {
      Device::Tty(tty) => tty.write(buf),
      Device::Stream(stream) => stream.write(buf),
    }
  }

  fn flush(&mut self) -> Result<(), io::Error> {
    match &mut self.device {
      Device::Tty(tty) => tty.flush(),
      Device::Stream(stream) => stream.flush(),
    }
  }
}

impl Read for OptoLink {
  fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
    match &mut self.device {
      Device::Tty(tty) => tty.read(buf),
      Device::Stream(stream) => stream.read(buf),
    }
  }
}
