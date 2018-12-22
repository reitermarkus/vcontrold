macro_rules! byte_type {
  ($t:ident, $len:expr) => {
    #[derive(Debug, Clone)]
    pub struct $t([u8; $len]);

    impl FromBytes for $t {
      fn from_bytes(bytes: &[u8]) -> $t {
        assert_eq!(bytes.len(), std::mem::size_of::<$t>());
        let mut buf = [0; std::mem::size_of::<$t>()];
        buf.copy_from_slice(&bytes);
        $t(buf)
      }
    }

    impl AsBytes for $t {
      #[inline]
      fn as_bytes(&self) -> &[u8] {
        &self.0
      }
    }
  }
}

macro_rules! impl_display {
  ($t:ty, $proxy:ident) => {
    impl std::fmt::Display for $t {
      fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        $proxy::from(self.clone()).fmt(f)
      }
    }
  }
}

macro_rules! byte_types {
  ($repr:ident, $t8:ident, $r8:ident, $t16:ident, $r16:ident, $read16:ident, $t32:ident, $r32:ident, $read32:ident) => {
    byte_type!($t8,  1);
    byte_type!($t16, 2);
    byte_type!($t32, 4);

    impl From<$repr> for $t8 {
      fn from(i: $repr) -> $t8 {
        let i = i as $r32;
        $t8([i as u8])
      }
    }

    impl From<$repr> for $t16 {
      fn from(i: $repr) -> $t16 {
        let i = i as $r32;
        $t16([i as u8, (i >> 2) as u8])
      }
    }

    impl From<$repr> for $t32 {
      fn from(i: $repr) -> $t32 {
        let i = i as $r32;
        $t32([i as u8, (i >> 2) as u8, (i >> 4) as u8, (i >> 6) as u8])
      }
    }

    impl From<$t8> for $repr {
      fn from(i: $t8) -> $repr {
        i.as_bytes()[0] as $r8 as $repr
      }
    }

    impl From<$t16> for $repr {
      fn from(i: $t16) -> $repr {
        LittleEndian::$read16(&i.as_bytes()) as $repr
      }
    }

    impl From<$t32> for $repr {
      fn from(i: $t32) -> $repr {
        LittleEndian::$read32(&i.as_bytes()) as $repr
      }
    }

    impl_display!($t8, $repr);
    impl_display!($t16, $repr);
    impl_display!($t32, $repr);
  }
}

mod cycletime;
pub use self::cycletime::CycleTime;

mod errstate;
pub use self::errstate::ErrState;

mod float;
pub use self::float::{Float8, Float16, Float32};

mod int;
pub use self::int::{Int8, Int16, Int32};

mod uint;
pub use self::uint::{UInt8, UInt16, UInt32};

mod systime;
pub use self::systime::SysTime;
