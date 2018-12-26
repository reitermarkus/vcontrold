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
          Self::from_le(unsafe { std::mem::transmute(buf) })
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
