use std::fmt;
use std::ffi::OsStr;
use std::io::{self, Read, Write};
use std::net::{TcpStream, ToSocketAddrs, SocketAddr};
use std::time::Duration;
use std::os::unix::io::AsRawFd;

use serial_core::{SerialPort, SerialPortSettings, BaudRate::Baud4800, Parity::ParityEven, StopBits::Stop2, CharSize::Bits8};
use serial::SystemPort;

enum Device {
  Tty(SystemPort),
  Stream(TcpStream),
}

impl fmt::Debug for Device   {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Device::Tty(tty) => write!(f, "TTYPort {{ fd: {:?} }}", tty.as_raw_fd()),
      Device::Stream(stream) => stream.fmt(f),
    }
  }
}

/// An Optolink connection via either a serial or TCP connection.
#[derive(Debug)]
pub struct Optolink {
  device: Device,
}

impl Optolink {
  pub(crate) const TIMEOUT: Duration = Duration::from_secs(60);

  /// Opens a serial device.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// use vcontrol::Optolink;
  ///
  /// # fn main() -> Result<(), Box<std::error::Error>> {
  /// let mut device = Optolink::open("/dev/ttyUSB0")?;
  /// # Ok(())
  /// # }
  /// ```
  pub fn open(port: impl AsRef<OsStr>) -> Result<Optolink, io::Error> {
    let mut tty = serial::open(&port)
      .map_err(|err| {
        let err = io::Error::from(err);

        if err.kind() == io::ErrorKind::NotFound {
          return io::Error::new(err.kind(), format!("{}: {}", err, port.as_ref().to_string_lossy()))
        }

        err
      })?;

    tty.set_timeout(Self::TIMEOUT)?;

    tty.reconfigure(&|settings: &mut SerialPortSettings| -> Result<(), serial_core::Error> {
      settings.set_parity(ParityEven);
      settings.set_stop_bits(Stop2);
      settings.set_char_size(Bits8);
      settings.set_baud_rate(Baud4800)
    })?;

    Ok(Optolink { device: Device::Tty(tty) })
  }

  /// Connects to a device via TCP.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// use vcontrol::Optolink;
  ///
  /// # fn main() -> Result<(), Box<std::error::Error>> {
  /// let mut device = Optolink::connect(("localhost", 1234))?;
  /// # Ok(())
  /// # }
  /// ```
  pub fn connect(addr: impl ToSocketAddrs) -> Result<Optolink, io::Error> {
    let addrs: Vec<SocketAddr> = addr.to_socket_addrs()?.collect();

    let stream = TcpStream::connect(&addrs as &[SocketAddr])
      .map_err(|err| {
        io::Error::new(err.kind(), format!("{}: {}", err, addrs.iter().map(|addr| addr.to_string()).collect::<Vec<String>>().join(", ")))
      })?;
    stream.set_read_timeout(Some(Self::TIMEOUT))?;
    Ok(Optolink { device: Device::Stream(stream) })
  }

  /// Purge all contents of the input buffer.
  pub fn purge(&mut self) -> Result<(), io::Error> {
    match &mut self.device {
      Device::Tty(tty) => {
        tty.set_timeout(Duration::new(0, 0))?;
        let _ = tty.bytes().try_for_each(|b| b.map(|_| ()));
        tty.set_timeout(Self::TIMEOUT)?;
      }
      Device::Stream(stream) => {
        stream.set_nonblocking(true)?;
        let _ = stream.bytes().try_for_each(|b| b.map(|_| ()));
        stream.set_nonblocking(false)?;
      },
    }

    Ok(())
  }
}

impl Write for Optolink {
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

impl Read for Optolink {
  fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
    match &mut self.device {
      Device::Tty(tty) => tty.read(buf),
      Device::Stream(stream) => stream.read(buf),
    }
  }
}
