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

mod errstate;
pub use self::errstate::ErrState;

mod systime;
pub use self::systime::SysTime;
