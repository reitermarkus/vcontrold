use std::num::ParseIntError;
use std::slice;

use super::*;

fn hex_to_u8(hex: &str) -> Result<u8, ParseIntError> {
  let without_prefix = hex.clone().trim_start_matches("0x");
  u8::from_str_radix(without_prefix, 16)
}

#[no_mangle]
pub unsafe extern fn hex2chr(hex: *const c_char) -> c_char {
  let string = CStr::from_ptr(hex).to_str().unwrap();

  match hex_to_u8(&string) {
    Ok(c) => c as _,
    Err(_) => {
      log_it!(LOG_WARNING, "Invalid hex char in {}", string);

      -1
    },
  }
}

fn u8_to_hex(bytes: &[u8]) -> String {
  bytes.iter().map(|byte| format!("{:02X}", byte))
    .collect::<Vec<String>>()
    .join(" ")
}

#[no_mangle]
pub unsafe extern fn char2hex(outString: *mut c_char, charPtr: *const c_char, len: c_int) -> c_int {
  let bytes = slice::from_raw_parts(charPtr as *const u8, len as usize);
  let string = CString::new(u8_to_hex(bytes).as_bytes()).unwrap();
  strcat(outString, string.as_ptr());
  len
}

fn string_to_u8(string: &str) -> Result<Vec<u8>, ParseIntError> {
  string.split_whitespace().map(|part| hex_to_u8(&part)).collect()
}

#[no_mangle]
pub unsafe extern fn string2chr(line: *mut c_char, buf: *mut c_char, bufsize: c_short) -> c_short {
  let line = CStr::from_ptr(line).to_str().unwrap();

  let bytes = string_to_u8(line).unwrap_or_else(|_| Vec::new());

  if bytes.len() < bufsize as _ {
    memcpy(buf as *mut _, bytes.as_ptr() as *mut _, bytes.len());
    bytes.len() as _
  } else {
    memcpy(buf as *mut _, bytes.as_ptr() as *mut _, bufsize as _);
    bufsize
  }
}
