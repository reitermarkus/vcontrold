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
  timeout: Option<Duration>,
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
  pub fn open(port: impl AsRef<OsStr>) -> io::Result<Optolink> {
    log::trace!("Optolink::open(…)");

    let mut tty = serial::open(&port)
      .map_err(|err| {
        let err = io::Error::from(err);

        if err.kind() == io::ErrorKind::NotFound {
          return io::Error::new(err.kind(), format!("{}: {}", err, port.as_ref().to_string_lossy()))
        }

        err
      })?;

    tty.set_timeout(Self::TIMEOUT)?;

    tty.reconfigure(&|settings: &mut dyn SerialPortSettings| -> Result<(), serial_core::Error> {
      settings.set_parity(ParityEven);
      settings.set_stop_bits(Stop2);
      settings.set_char_size(Bits8);
      settings.set_baud_rate(Baud4800)
    })?;

    Ok(Optolink { device: Device::Tty(tty), timeout: None })
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
  pub fn connect(addr: impl ToSocketAddrs) -> io::Result<Optolink> {
    log::trace!("Optolink::connect(…)");

    let addrs: Vec<SocketAddr> = addr.to_socket_addrs()?.collect();

    let stream = TcpStream::connect(&addrs as &[SocketAddr])
      .map_err(|err| {
        io::Error::new(err.kind(), format!("{}: {}", err, addrs.iter().map(|addr| addr.to_string()).collect::<Vec<String>>().join(", ")))
      })?;
    stream.set_read_timeout(Some(Self::TIMEOUT))?;
    Ok(Optolink { device: Device::Stream(stream), timeout: None })
  }

  /// Purge all contents of the input buffer.
  pub fn purge(&mut self) -> Result<(), io::Error> {
    log::trace!("Optolink::purge()");

    match &mut self.device {
      Device::Tty(tty) => {
        tty.set_timeout(Duration::new(0, 0))?;
        let _ = tty.bytes().try_for_each(|b| b.map(|_| ()));
        tty.set_timeout(self.timeout.unwrap_or(Self::TIMEOUT))?;
      }
      Device::Stream(stream) => {
        stream.set_nonblocking(true)?;
        let _ = stream.bytes().try_for_each(|b| b.map(|_| ()));
        stream.set_nonblocking(false)?;
      },
    }

    Ok(())
  }

  pub fn set_timeout(&mut self, timeout: Option<Duration>) -> io::Result<()> {
    self.timeout = timeout;

    match &mut self.device {
      Device::Tty(tty) => Ok(tty.set_timeout(timeout.unwrap_or(Self::TIMEOUT))?),
      Device::Stream(stream) => stream.set_read_timeout(timeout),
    }
  }
}

impl Write for Optolink {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    log::trace!("Optolink::write(…)");

    match &mut self.device {
      Device::Tty(tty) => tty.write(buf),
      Device::Stream(stream) => stream.write(buf),
    }
  }

  fn flush(&mut self) -> Result<(), io::Error> {
    log::trace!("Optolink::flush()");

    match &mut self.device {
      // This is a workaround for `tcdrain`, which `SystemPort::flush`
      // uses under the hood. If a device is disconnected, `tcdrain`
      // will block forever instead of timing out, so we spawn it in a
      // separate thread and manually create a timeout.
      #[cfg(unix)]
      Device::Tty(ref mut tty) => {
        use std::mem;
        use std::os::unix::thread::JoinHandleExt;
        use std::sync::mpsc::channel;
        use std::thread;
        use std::time::Instant;

        let start = Instant::now();

        // Allow moving this to the helper thread. This is safe because we either
        // join the thread in this scope or cancel it if it times out.
        let tty: &mut SystemPort = tty;
        let tty: &'static mut SystemPort = unsafe { mem::transmute(tty) };

        let (tx, rx) = channel();

        let t = thread::spawn(move || {
          let res = tty.flush();
          tx.send(()).unwrap();
          res
        });

        loop {
          log::trace!("Optolink::flush() loop");

          if rx.try_recv().is_ok() {
            return t.join().unwrap()
          }

          let stop = Instant::now();

          if (stop - start) > Self::TIMEOUT {
            assert_eq!(unsafe { libc::pthread_cancel(t.into_pthread_t() as _) }, 0);
            return Err(io::Error::new(io::ErrorKind::TimedOut, "flush timed out"))
          }

          thread::yield_now();
        }
      },
      #[cfg(not(unix))]
      Device::Tty(tty) => tty.flush(),
      Device::Stream(stream) => stream.flush(),
    }
  }
}

impl Read for Optolink {
  fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
    log::trace!("Optolink::read(…)");

    match &mut self.device {
      Device::Tty(tty) => tty.read(buf),
      Device::Stream(stream) => stream.read(buf),
    }
  }
}
