use std::fmt;
use std::hash::Hasher;

pub(crate) trait FromBytes {
  fn from_bytes(bytes: &[u8]) -> Self;
}

pub(crate) trait ToBytes {
  fn to_bytes(&self) -> Vec<u8>;
}

impl FromBytes for Vec<u8> {
  fn from_bytes(bytes: &[u8]) -> Vec<u8> {
    bytes.to_vec()
  }
}

impl ToBytes for Vec<u8> {
  fn to_bytes(&self) -> Vec<u8> {
    self.clone()
  }
}

macro_rules! from_bytes_le {
  ($($t:ty),+) => {
    $(
      impl FromBytes for $t {
        fn from_bytes(bytes: &[u8]) -> Self {
          let mut buf = [0; std::mem::size_of::<Self>()];
          buf.copy_from_slice(&bytes);
          Self::from_le_bytes(buf)
        }
      }
    )+
  };
}

macro_rules! to_bytes_le {
  ($t:ty, [u8; 1]) => {
    impl ToBytes for $t {
      fn to_bytes(&self) -> Vec<u8> {
        vec![*self as u8]
      }
    }
  };
  ($t:ty, $n:ty) => {
    impl ToBytes for $t {
      fn to_bytes(&self) -> Vec<u8> {
        unsafe { std::mem::transmute::<$t, $n>(self.to_le()) }.to_vec()
      }
    }
  };
}

from_bytes_le!(i8, i16, i32);
to_bytes_le!(i8,  [u8; 1]);
to_bytes_le!(i16, [u8; 2]);
to_bytes_le!(i32, [u8; 4]);

from_bytes_le!(u8, u16, u32);
to_bytes_le!(u8,  [u8; 1]);
to_bytes_le!(u16, [u8; 2]);
to_bytes_le!(u32, [u8; 4]);

macro_rules! byte_type {
  ($t:ident, $len:expr) => {
    #[derive(Debug, Clone)]
    pub struct $t([u8; $len]);

    impl $crate::FromBytes for $t {
      fn from_bytes(bytes: &[u8]) -> $t {
        let mut buf = [0; std::mem::size_of::<$t>()];
        buf.copy_from_slice(&bytes);
        $t(buf)
      }
    }

    impl $crate::ToBytes for $t {
      fn to_bytes(&self) -> Vec<u8> {
        self.0.to_vec()
      }
    }
  }
}

mod cycletime;
pub use self::cycletime::CycleTime;

mod systime;
pub use self::systime::SysTime;

#[derive(PartialEq, Eq, Hash)]
pub(crate) enum Bytes {
  One([u8; 1]),
  Two([u8; 2]),
}

impl fmt::Debug for Bytes {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Bytes::One(bytes) => write!(f, "Bytes::One({:?})", bytes),
      Bytes::Two(bytes) => write!(f, "Bytes::Two({:?})", bytes),
    }
  }
}

impl FromBytes for Bytes {
  fn from_bytes(bytes: &[u8]) -> Bytes {
    match bytes.len() {
      1 => Bytes::One([bytes[0]]),
      2 => Bytes::Two([bytes[0], bytes[1]]),
      _ => unreachable!(),
    }
  }
}

impl ToBytes for Bytes {
  fn to_bytes(&self) -> Vec<u8> {
    match self {
      Bytes::One(bytes) => bytes.to_vec(),
      Bytes::Two(bytes) => bytes.to_vec(),
    }
  }
}

impl phf_shared::PhfHash for Bytes {
  #[inline]
  fn phf_hash<H: Hasher>(&self, state: &mut H) {
    match self {
      Bytes::One(bytes) => bytes.to_vec().phf_hash(state),
      Bytes::Two(bytes) => bytes.to_vec().phf_hash(state),
    }
  }
}

impl phf_shared::FmtConst for Bytes {
  #[inline]
  fn fmt_const(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}
