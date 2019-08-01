use crate::{Error, Optolink, Device, Protocol, Value};

#[derive(Debug)]
pub struct VControl<D: Device> {
  device: Optolink,
  phantom: std::marker::PhantomData<D>,
}

impl<D: Device> VControl<D> {
  pub fn connect(mut device: Optolink) -> Result<VControl<D>, Error> {
    D::Protocol::negotiate(&mut device)?;
    Ok(VControl { device, phantom: std::marker::PhantomData })
  }

  /// Gets the value for the given command.
  ///
  /// If the command specified is not available, an IO error of the kind `AddrNotAvailable` is returned.
  pub fn get(&mut self, command: &str) -> Result<Value, Error> {
    if let Some(command) = D::command(command) {
      command.get::<D::Protocol>(&mut self.device)
    } else {
      Err(Error::UnsupportedCommand(command.to_owned()))
    }
  }

  /// Sets the value for the given command.
  ///
  /// If the command specified is not available, an IO error of the kind `AddrNotAvailable` is returned.
  pub fn set(&mut self, command: &str, input: &Value) -> Result<(), Error> {
    if let Some(command) = D::command(command) {
      command.set::<D::Protocol>(&mut self.device, input)
    } else {
      Err(Error::UnsupportedCommand(command.to_owned()))
    }
  }
}
