#![crate_type = "staticlib"]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

mod configuration;
pub use crate::configuration::*;

mod traits;
pub use crate::traits::{FromBytes, ToBytes};

pub mod types;

mod optolink;
pub use crate::optolink::*;

pub mod protocol;
