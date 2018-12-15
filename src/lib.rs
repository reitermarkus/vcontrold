#![crate_type = "staticlib"]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::CStr;

use std::os::raw::{c_int, c_char};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[no_mangle]
pub unsafe extern fn openDevice(device: *const c_char) -> c_int {
  let device = CStr::from_ptr(device).to_str().unwrap();

  let fd;

  if !device.starts_with("/") && device.contains(":") {
    let parts: Vec<_> = device.splitn(2, ':').collect();

    let hostname = String::from(parts[0]);
    let port: c_int = match parts[1].parse() {
      Ok(port) => port,
      _ => return -1,
    };

    fd = openCliSocket(hostname.as_ptr() as _, port, 1);
  } else {
    fd = opentty(device.as_ptr() as _);
  }

  if fd < 0 {
    return -1
  }

  fd
}
