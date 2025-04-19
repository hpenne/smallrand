use core::ops::{Bound, Range, RangeBounds};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct GenerateRange<T> {
    pub start: T,
    pub end_inclusive: T,
}

macro_rules! from_range_bounds {
    ($value_type: ty) => {
        impl<R> From<R> for GenerateRange<$value_type>
        where
            R: RangeBounds<$value_type>,
        {
            #[inline(always)]
            fn from(range: R) -> Self {
                Self {
                    start: match range.start_bound() {
                        Bound::Included(start) => *start,
                        Bound::Excluded(start) => {
                            start.checked_add(1).expect("Range start overflow")
                        }
                        Bound::Unbounded => <$value_type>::MIN,
                    },
                    end_inclusive: match range.end_bound() {
                        Bound::Included(end) => *end,
                        Bound::Excluded(end) => end.checked_sub(1).expect("Range end underflow"),
                        Bound::Unbounded => <$value_type>::MAX,
                    },
                }
            }
        }
    };
}

from_range_bounds!(u8);
from_range_bounds!(u16);
from_range_bounds!(u32);
from_range_bounds!(u64);
from_range_bounds!(u128);
from_range_bounds!(usize);
from_range_bounds!(i8);
from_range_bounds!(i16);
from_range_bounds!(i32);
from_range_bounds!(i64);
from_range_bounds!(i128);
from_range_bounds!(isize);

macro_rules! from_range {
    ($value_type: ty) => {
        impl From<Range<$value_type>> for GenerateRange<$value_type> {
            #[inline(always)]
            fn from(range: Range<$value_type>) -> Self {
                Self {
                    start: range.start,
                    end_inclusive: range.end,
                }
            }
        }
    };
}

from_range!(f32);
from_range!(f64);

#[cfg(test)]
mod test {
    use super::*;

    fn to_range(range: impl RangeBounds<u8>) -> GenerateRange<u8> {
        range.into()
    }

    #[test]
    fn integer_conversions() {
        assert_eq!(
            GenerateRange {
                start: 2_u8,
                end_inclusive: 41
            },
            to_range(2..42)
        );
        assert_eq!(
            GenerateRange {
                start: 2_u8,
                end_inclusive: 42
            },
            to_range(2..=42)
        );
    }
}
