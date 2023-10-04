use std::io::SeekFrom;
use std::num::TryFromIntError;
use std::{fmt::Display, ops::RangeBounds};

use snafu::prelude::*;

pub const BYTES: &str = "bytes";

/// A single range in a `Range` request.
///
/// The [ByteRange::Range] variant can be created from rust ranges, like
///
/// ```rust
/// # use object_store::ByteRange;
/// let range: HttpRange = (50..150).into();
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ByteRange {
    /// A range with a given start point and possibly an end point (otherwise EOF).
    Range {
        /// Offset of the first byte in the range, 0-based inclusive.
        start: usize,
        /// Offset of the last byte in the range, 0-based inclusive.
        /// [None] if the range should go to the last byte.
        end: Option<usize>,
    },
    /// A range defined as the number of bytes at the end of the resource.
    Suffix(usize),
}

impl ByteRange {
    pub fn expected_len(&self, total_length: Option<usize>) -> Option<usize> {
        match self {
            ByteRange::Range { start, end } => {
                let e = end.or(total_length)?;
                e.checked_sub(*start)
            }
            ByteRange::Suffix(n) => Some(*n),
        }
    }
}

impl Display for ByteRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{BYTES}="))?;
        match self {
            ByteRange::Range { start, end } => {
                f.write_fmt(format_args!("{start}-"))?;
                if let Some(e) = end {
                    f.write_fmt(format_args!("{e}"))?;
                }
                Ok(())
            }
            ByteRange::Suffix(len) => f.write_fmt(format_args!("-{len}")),
        }
    }
}

impl<T: RangeBounds<usize>> From<T> for ByteRange {
    fn from(value: T) -> Self {
        use std::ops::Bound::*;
        let start = match value.start_bound() {
            Included(i) => *i,
            Excluded(i) => i + 1,
            Unbounded => 0,
        };
        let end = match value.end_bound() {
            Included(i) => Some(*i),
            Excluded(i) => Some(i - 1),
            Unbounded => None,
        };
        ByteRange::Range { start, end }
    }
}

impl TryInto<SeekFrom> for &ByteRange {
    fn try_into(self) -> Result<SeekFrom, Self::Error> {
        let res = match self {
            ByteRange::Range { start, end: _ } => SeekFrom::Start(u64::try_from(*start)?),
            ByteRange::Suffix(o) => SeekFrom::End(-i64::try_from(*o)?),
        };
        Ok(res)
    }

    type Error = TryFromIntError;
}
