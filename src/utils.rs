use crate::instruction::Size;

/// Returns bits [beg, end] inclusive, starting at 0.
#[inline(always)]
pub const fn bits(d: u16, beg: u16, end: u16) -> u16 {
    let mask = (1 << end + 1 - beg) - 1;
    d >> beg & mask
}

/// Trait that convert the first bytes of slices to their big-endian integer value.
pub trait SliceAs {
    /// Interprets the first elements of the silce as an big-endian 32 bits integer.
    fn u32_be(self) -> u32;
    /// Returns `self[0] as i8` for Byte size, `self[0] as i16` for Word size or `self.u32_be() as i32` for Long size.
    fn i32_be_sized(self, size: Size) -> i32;
    /// Interprets the first elements of the silce as an big-endian 16 bits integer then advances self by two bytes (one word) in the slice.
    fn get_next_word(&mut self) -> u16;
}

impl SliceAs for &[u16] {
    fn u32_be(self) -> u32 {
        (self[0] as u32) << 16 | self[1] as u32
    }

    fn i32_be_sized(self, size: Size) -> i32 {
        match size {
            Size::Byte => self[0] as i8 as i32,
            Size::Word => self[0] as i16 as i32,
            Size::Long => self.u32_be() as i32,
        }
    }

    fn get_next_word(&mut self) -> u16 {
        let d = self[0];
        *self = &self[1..];
        d
    }
}

/// Converts integers to their big-endian array.
pub trait AsArray<const N: usize> {
    fn as_array_be(self) -> [u8; N];
}

impl AsArray<2> for u16 {
    fn as_array_be(self) -> [u8; 2] {
        [(self >> 8) as u8, self as u8]
    }
}

impl AsArray<4> for u32 {
    fn as_array_be(self) -> [u8; 4] {
        [(self >> 24) as u8, (self >> 16) as u8, (self >> 8) as u8, self as u8]
    }
}

pub trait BigInt {
    fn extended_add(self, rhs: Self, carry: bool) -> (Self, bool) where Self: Sized;
    fn extended_sub(self, rhs: Self, carry: bool) -> (Self, bool) where Self: Sized;
}

macro_rules! impl_bigint {
    ($type:ty, $bigtype:ty) => {
        impl BigInt for $type {
            fn extended_add(self, rhs: Self, carry: bool) -> (Self, bool)
            where Self: Sized {
                let res = self as $bigtype + rhs as $bigtype + carry as $bigtype;
                (res as Self, res < <$type>::MIN as $bigtype || res > <$type>::MAX as $bigtype)
            }

            fn extended_sub(self, rhs: Self, carry: bool) -> (Self, bool)
            where Self: Sized {
                let res = self as $bigtype - rhs as $bigtype - carry as $bigtype;
                (res as Self, res < <$type>::MIN as $bigtype || res > <$type>::MAX as $bigtype)
            }
        }
    };
}

impl_bigint!(u8, i16);
impl_bigint!(i8, i16);

impl_bigint!(u16, i32);
impl_bigint!(i16, i32);

impl_bigint!(u32, i64);
impl_bigint!(i32, i64);
