use std::fmt;
use std::str::FromStr;

use byteorder::{ByteOrder, LittleEndian};

use crate::{FromBytes, AsBytes};

byte_types!(u32, UInt8, u8, UInt16, u16, read_u16, UInt32, u32, read_u32);
