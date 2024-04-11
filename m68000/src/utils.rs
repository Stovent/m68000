// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Utility traits and functions.

use crate::exception::ADDRESS_ERROR;
use crate::instruction::Size;

use std::num::Wrapping;
use std::ops::{BitAnd, BitOr, BitXor, Shl};

/// Checks if the given bit of the given data is set.
#[inline(always)]
pub const fn bit(data: u16, bit: u16) -> bool {
    data & (1 << bit) != 0
}

/// Returns bits `[beg, end]` inclusive, starting at 0.
#[inline(always)]
pub const fn bits(d: u16, beg: u16, end: u16) -> u16 {
    let mask = (1 << (end + 1 - beg)) - 1;
    d >> beg & mask
}

/// Trait to see if an integer is even or not.
pub trait IsEven : Sized {
    fn is_even(self) -> bool;
    fn even(self) -> Result<Self, u8>;
}

impl IsEven for u32 {
    #[inline(always)]
    fn is_even(self) -> bool {
        self & 1 == 0
    }

    #[inline(always)]
    fn even(self) -> Result<Self, u8> {
        if self.is_even() {
            Ok(self)
        } else {
            Err(ADDRESS_ERROR)
        }
    }
}

impl IsEven for Wrapping<u32> {
    #[inline(always)]
    fn is_even(self) -> bool {
        self.0 & 1 == 0
    }

    #[inline(always)]
    fn even(self) -> Result<Self, u8> {
        if self.is_even() {
            Ok(self)
        } else {
            Err(ADDRESS_ERROR)
        }
    }
}

impl IsEven for usize {
    #[inline(always)]
    fn is_even(self) -> bool {
        self & 1 == 0
    }

    #[inline(always)]
    fn even(self) -> Result<Self, u8> {
        if self.is_even() {
            Ok(self)
        } else {
            Err(ADDRESS_ERROR)
        }
    }
}

/// Trait that convert the first bytes of slices to their big-endian integer value.
pub trait SliceAs {
    /// Interprets the first elements of the silce as a big-endian 32 bits integer.
    fn u32_be(self) -> u32;
    /// Casts the first elements as a signed i8, i16 or i32 depending on the size, then casts it to i32 and returns it.
    fn i32_be_sized(self, size: Size) -> i32;
    /// Interprets the first elements of the silce as a big-endian 16 bits integer then advances self by two bytes in the slice.
    fn get_next_word(&mut self) -> u16;
}

impl SliceAs for &[u16] {
    fn u32_be(self) -> u32 {
        (self[0] as u32) << 16 | self[1] as u32
    }

    /// Returns `self[0] as i8 as i32` for Byte size, `self[0] as i16 as i32` for Word size or `self.u32_be() as i32` for Long size.
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

pub trait CarryingOps<S, U> : Sized + Integer {
    fn signed_carrying_add(self, rhs: Self, carry: bool) -> (S, bool);
    fn unsigned_carrying_add(self, rhs: Self, carry: bool) -> (U, bool);

    fn signed_borrowing_sub(self, rhs: Self, carry: bool) -> (S, bool);
    fn unsigned_borrowing_sub(self, rhs: Self, carry: bool) -> (U, bool);
}

impl CarryingOps<i8, u8> for u8 {
    fn signed_carrying_add(self, rhs: Self, carry: bool) -> (i8, bool) {
        (self as i8).carrying_add(rhs as i8, carry)
    }

    fn unsigned_carrying_add(self, rhs: Self, carry: bool) -> (u8, bool) {
        self.carrying_add(rhs, carry)
    }

    fn signed_borrowing_sub(self, rhs: Self, carry: bool) -> (i8, bool) {
        (self as i8).borrowing_sub(rhs as i8, carry)
    }

    fn unsigned_borrowing_sub(self, rhs: Self, carry: bool) -> (u8, bool) {
        self.borrowing_sub(rhs, carry)
    }
}

impl CarryingOps<i16, u16> for u16 {
    fn signed_carrying_add(self, rhs: Self, carry: bool) -> (i16, bool) {
        (self as i16).carrying_add(rhs as i16, carry)
    }

    fn unsigned_carrying_add(self, rhs: Self, carry: bool) -> (u16, bool) {
        self.carrying_add(rhs, carry)
    }

    fn signed_borrowing_sub(self, rhs: Self, carry: bool) -> (i16, bool) {
        (self as i16).borrowing_sub(rhs as i16, carry)
    }

    fn unsigned_borrowing_sub(self, rhs: Self, carry: bool) -> (u16, bool) {
        self.borrowing_sub(rhs, carry)
    }
}

impl CarryingOps<i32, u32> for u32 {
    fn signed_carrying_add(self, rhs: Self, carry: bool) -> (i32, bool) {
        (self as i32).carrying_add(rhs as i32, carry)
    }

    fn unsigned_carrying_add(self, rhs: Self, carry: bool) -> (u32, bool) {
        self.carrying_add(rhs, carry)
    }

    fn signed_borrowing_sub(self, rhs: Self, carry: bool) -> (i32, bool) {
        (self as i32).borrowing_sub(rhs as i32, carry)
    }

    fn unsigned_borrowing_sub(self, rhs: Self, carry: bool) -> (u32, bool) {
        self.borrowing_sub(rhs, carry)
    }
}

pub trait Integer : Copy + PartialEq + PartialOrd +
                    BitAnd<Output = Self> + BitOr<Output = Self> + BitXor<Output = Self> +
                    Shl<Output = Self> {
    const ZERO: Self;
    const SIGN_BIT_MASK: Self;
}

macro_rules! impl_integer {
    ($t:ty, $sign:expr) => {
        impl Integer for $t {
            const ZERO: Self = 0;
            const SIGN_BIT_MASK: Self = $sign;
        }
    };
}

impl_integer!(i8, -0x80);
impl_integer!(u8, 0x80);
impl_integer!(i16, -0x8000);
impl_integer!(u16, 0x8000);
impl_integer!(i32, -0x8000_0000);
impl_integer!(u32, 0x8000_0000);
