//! Definition of strongly typed units, like `Line`, `Column`, or `Location`. Used to express type
//! level dependencies in the whole library.

use crate::prelude::*;
use enso_types::unit;
use enso_types::newtype;



// ===============
// === Exports ===
// ===============

/// Common traits.
pub mod traits {
    pub use super::bytes::Into  as TRAIT_bytes_into;
    pub use super::column::Into as TRAIT_column_into;
    pub use super::line::Into   as TRAIT_line_into;
}
pub use traits::*;



// =============
// === Bytes ===
// =============

unit! {
/// An offset in the buffer in bytes.
Bytes::bytes(i32)
}

impl Bytes {
    pub fn as_usize(self) -> usize {
        self.value.max(0) as usize
    }
}

impl<T:Into<Bytes>> bytes::Into for Range<T> {
    type Output = Range<Bytes>;
    fn bytes(self) -> Self::Output {
        let start = self.start.into();
        let end   = self.end.into();
        Range {start,end}
    }
}

impl From<usize> for Bytes {
    fn from(t:usize) -> Self {
        (t as i32).into()
    }
}

impl From<&usize> for Bytes {
    fn from(t:&usize) -> Self {
        (*t as i32).into()
    }
}



// ============
// === Line ===
// ============

unit! {
/// A type representing vertical measurements.
Line::line(i32)
}

impl Line {
    pub fn as_usize(self) -> usize {
        self.value.max(0) as usize
    }

    // FIXME
    pub fn abs(self) -> Self {
        self.value.saturating_abs().into()
    }
}

impl From<usize> for Line {
    fn from(t:usize) -> Self {
        (t as i32).into()
    }
}

impl From<&usize> for Line {
    fn from(t:&usize) -> Self {
        (*t as i32).into()
    }
}



// ==============
// === Column ===
// ==============

unit! {
/// A type representing horizontal measurements expressed as number of grapheme clusters.
Column::column(i32)
}

impl Column {
    pub fn as_usize(self) -> usize {
        self.value.max(0) as usize
    }

    // FIXME
    pub fn abs(self) -> Self {
        self.value.saturating_abs().into()
    }
}

impl From<usize> for Column {
    fn from(t:usize) -> Self {
        (t as i32).into()
    }
}

impl From<&usize> for Column {
    fn from(t:&usize) -> Self {
        (*t as i32).into()
    }
}



// ================
// === Location ===
// ================

newtype! {
/// A type representing 2d measurements.
Location {
    line   : Line,
    column : Column,
}}

impl Location {
    pub fn with_line(self,line:Line) -> Self {
        Self {line,..self}
    }

    pub fn with_column(self,column:Column) -> Self {
        Self {column,..self}
    }
}
