// automatically generated by the FlatBuffers compiler, do not modify



use std::mem;
use std::cmp::Ordering;

extern crate flatbuffers;
use self::flatbuffers::EndianScalar;

#[allow(unused_imports, dead_code)]
pub mod org {

  use std::mem;
  use std::cmp::Ordering;

  extern crate flatbuffers;
  use self::flatbuffers::EndianScalar;
#[allow(unused_imports, dead_code)]
pub mod enso {

  use std::mem;
  use std::cmp::Ordering;

  extern crate flatbuffers;
  use self::flatbuffers::EndianScalar;
#[allow(unused_imports, dead_code)]
pub mod languageserver {

  use std::mem;
  use std::cmp::Ordering;

  extern crate flatbuffers;
  use self::flatbuffers::EndianScalar;
#[allow(unused_imports, dead_code)]
pub mod protocol {

  use std::mem;
  use std::cmp::Ordering;

  extern crate flatbuffers;
  use self::flatbuffers::EndianScalar;
#[allow(unused_imports, dead_code)]
pub mod data {

  use std::mem;
  use std::cmp::Ordering;

  extern crate flatbuffers;
  use self::flatbuffers::EndianScalar;
#[allow(unused_imports, dead_code)]
pub mod util {

  use std::mem;
  use std::cmp::Ordering;

  extern crate flatbuffers;
  use self::flatbuffers::EndianScalar;

// struct EnsoUUID, aligned to 8
#[repr(C, align(8))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnsoUUID {
  leastSigBits_: u64,
  mostSigBits_: u64,
} // pub struct EnsoUUID
impl flatbuffers::SafeSliceAccess for EnsoUUID {}
impl<'a> flatbuffers::Follow<'a> for EnsoUUID {
  type Inner = &'a EnsoUUID;
  #[inline]
  fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
    <&'a EnsoUUID>::follow(buf, loc)
  }
}
impl<'a> flatbuffers::Follow<'a> for &'a EnsoUUID {
  type Inner = &'a EnsoUUID;
  #[inline]
  fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
    flatbuffers::follow_cast_ref::<EnsoUUID>(buf, loc)
  }
}
impl<'b> flatbuffers::Push for EnsoUUID {
    type Output = EnsoUUID;
    #[inline]
    fn push(&self, dst: &mut [u8], _rest: &[u8]) {
        let src = unsafe {
            ::std::slice::from_raw_parts(self as *const EnsoUUID as *const u8, Self::size())
        };
        dst.copy_from_slice(src);
    }
}
impl<'b> flatbuffers::Push for &'b EnsoUUID {
    type Output = EnsoUUID;

    #[inline]
    fn push(&self, dst: &mut [u8], _rest: &[u8]) {
        let src = unsafe {
            ::std::slice::from_raw_parts(*self as *const EnsoUUID as *const u8, Self::size())
        };
        dst.copy_from_slice(src);
    }
}


impl EnsoUUID {
  pub fn new<'a>(_leastSigBits: u64, _mostSigBits: u64) -> Self {
    EnsoUUID {
      leastSigBits_: _leastSigBits.to_little_endian(),
      mostSigBits_: _mostSigBits.to_little_endian(),

    }
  }
  pub fn leastSigBits<'a>(&'a self) -> u64 {
    self.leastSigBits_.from_little_endian()
  }
  pub fn mostSigBits<'a>(&'a self) -> u64 {
    self.mostSigBits_.from_little_endian()
  }
}

pub enum ErrorOffset {}
#[derive(Copy, Clone, Debug, PartialEq)]

pub struct Error<'a> {
  pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for Error<'a> {
    type Inner = Error<'a>;
    #[inline]
    fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
        Self {
            _tab: flatbuffers::Table { buf: buf, loc: loc },
        }
    }
}

impl<'a> Error<'a> {
    #[inline]
    pub fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
        Error {
            _tab: table,
        }
    }
    #[allow(unused_mut)]
    pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
        _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
        args: &'args ErrorArgs<'args>) -> flatbuffers::WIPOffset<Error<'bldr>> {
      let mut builder = ErrorBuilder::new(_fbb);
      if let Some(x) = args.message { builder.add_message(x); }
      builder.add_code(args.code);
      builder.finish()
    }

    pub const VT_CODE: flatbuffers::VOffsetT = 4;
    pub const VT_MESSAGE: flatbuffers::VOffsetT = 6;

  #[inline]
  pub fn code(&self) -> i32 {
    self._tab.get::<i32>(Error::VT_CODE, Some(0)).unwrap()
  }
  #[inline]
  pub fn message(&self) -> Option<&'a str> {
    self._tab.get::<flatbuffers::ForwardsUOffset<&str>>(Error::VT_MESSAGE, None)
  }
}

pub struct ErrorArgs<'a> {
    pub code: i32,
    pub message: Option<flatbuffers::WIPOffset<&'a  str>>,
}
impl<'a> Default for ErrorArgs<'a> {
    #[inline]
    fn default() -> Self {
        ErrorArgs {
            code: 0,
            message: None,
        }
    }
}
pub struct ErrorBuilder<'a: 'b, 'b> {
  fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
  start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b> ErrorBuilder<'a, 'b> {
  #[inline]
  pub fn add_code(&mut self, code: i32) {
    self.fbb_.push_slot::<i32>(Error::VT_CODE, code, 0);
  }
  #[inline]
  pub fn add_message(&mut self, message: flatbuffers::WIPOffset<&'b  str>) {
    self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(Error::VT_MESSAGE, message);
  }
  #[inline]
  pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> ErrorBuilder<'a, 'b> {
    let start = _fbb.start_table();
    ErrorBuilder {
      fbb_: _fbb,
      start_: start,
    }
  }
  #[inline]
  pub fn finish(self) -> flatbuffers::WIPOffset<Error<'a>> {
    let o = self.fbb_.end_table(self.start_);
    flatbuffers::WIPOffset::new(o.value())
  }
}

}  // pub mod util
}  // pub mod data
}  // pub mod protocol
}  // pub mod languageserver
}  // pub mod enso
}  // pub mod org

