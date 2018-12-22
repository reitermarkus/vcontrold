use std::fmt;
use std::str::FromStr;

use byteorder::{ByteOrder, LittleEndian};

use crate::{FromBytes, AsBytes};

byte_types!(i32, Int8, i8, Int16, i16, read_i16, Int32, i32, read_i32);
