#![crate_type = "staticlib"]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::{CStr, CString};
use std::mem;
use std::ptr;

use libc::{c_int, c_short, c_char, c_float, c_uchar, c_void, addrinfo, freeaddrinfo, close, gai_strerror, getnameinfo, fork, ssize_t, size_t, memcpy, strcat, sprintf, calloc};
use libc::{AI_PASSIVE, AI_ALL, AI_V4MAPPED, TCP_NODELAY, LOG_INFO, LOG_NOTICE, LOG_ERR, LOG_WARNING, SIG_IGN, signal, NI_NUMERICHOST, NI_MAXSERV, NI_MAXHOST};
use libc::getaddrinfo;
use libc::exit;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

macro_rules! log_it {
  ($level:expr, $($arg:tt)*) => {{
    let string = format!($($arg)*);
    logIT($level, CString::new(string).unwrap().into_raw());
  }}
}

mod arithmetic;
pub use crate::arithmetic::*;

mod client;
pub use crate::client::*;

mod common;
pub use crate::common::*;

mod io;
pub use crate::io::*;

mod socket;
pub use crate::socket::*;
