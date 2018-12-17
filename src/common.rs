use std::num::ParseIntError;

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
