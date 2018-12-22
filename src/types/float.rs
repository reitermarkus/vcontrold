use std::fmt;
use std::str::FromStr;

use byteorder::{ByteOrder, LittleEndian};

use crate::{FromBytes, AsBytes};

byte_types!(f32, Float8, i8, Float16, i16, read_i16, Float32, i32, read_i32);
