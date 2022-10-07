// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Tests for the `overflowing_add`, `overflowing_sub`, `carrying_add` and `borrowing_sub` functions.
//!
//! The goal is to ensure that the functions have the correct behaviours on carry/overflow detection
//! to correctly compute the CCR bits during instructions.
//!
//! Currently, `carrying_add` and `borrowing_sub` does not have the correct behaviours on signed operands.

#![feature(bigint_helper_methods)]

macro_rules! test_operator {
    ($operator:expr, $expected:expr) => {
        let res = $operator;
        assert_eq!(res, $expected, "{} -> {:?}, expected {:?}", stringify!($operator), res, $expected);
    }
}

#[test]
fn std_overflowing() {
    // add u8 1
    test_operator!(255u8.overflowing_add(1), (0, true));
    test_operator!(127u8.overflowing_add(1), (128, false));
    // add i8 1
    test_operator!((255u8 as i8).overflowing_add(1), (0, false));
    test_operator!(127i8.overflowing_add(1), (-128, true));

    // add u8 -1
    test_operator!(0u8.overflowing_add(255), (255, false));
    test_operator!(128u8.overflowing_add(255), (127, true));
    // add i8 -1
    test_operator!(0i8.overflowing_add(-1), (-1, false));
    test_operator!((-128i8).overflowing_add(-1), (127, true));

    // sub u8 1
    test_operator!(0u8.overflowing_sub(1), (255, true));
    test_operator!(128u8.overflowing_sub(1), (127, false));
    // sub i8 1
    test_operator!(0i8.overflowing_sub(1), (-1, false));
    test_operator!((-128i8).overflowing_sub(1), (127, true));

    // sub u8 -1
    test_operator!(255u8.overflowing_sub(255), (0, false));
    test_operator!(127u8.overflowing_sub(255), (128, true));
    // sub i8 -1
    test_operator!((255u8 as i8).overflowing_sub(-1), (0, false));
    test_operator!(127i8.overflowing_sub(-1), (-128, true));
}

#[test]
fn std_big_int() {
    // add u8
    test_operator!(255u8.carrying_add(1, false), (0, true));
    test_operator!(255u8.carrying_add(0, true), (0, true));
    test_operator!(255u8.carrying_add(1, true), (1, true));
    test_operator!(0u8.carrying_add(255, false), (255, false));
    test_operator!(0u8.carrying_add(255, true), (0, true));

    // add i8
    test_operator!(127i8.carrying_add(1, false), (-128, true));
    test_operator!(127i8.carrying_add(0, true), (-128, true));
    test_operator!(127i8.carrying_add(1, true), (-127, true));
    test_operator!(127i8.carrying_add(-1, false), (126, false));
    test_operator!(127i8.carrying_add(-1, true), (127, false)); // no intermediate overflow
    test_operator!((-128i8).carrying_add(-1, false), (127, true));
    test_operator!((-128i8).carrying_add(-1, true), (-128, false)); // no intermediate overflow

    // sub u8
    test_operator!(0u8.borrowing_sub(1, false), (255u8, true));
    test_operator!(0u8.borrowing_sub(0, true), (255u8, true));
    test_operator!(0u8.borrowing_sub(1, true), (254u8, true));
    test_operator!(255u8.borrowing_sub(255, false), (0, false));
    test_operator!(255u8.borrowing_sub(255, true), (255u8, true));

    // sub i8
    test_operator!((-128i8).borrowing_sub(1, false), (127, true));
    test_operator!((-128i8).borrowing_sub(0, true), (127, true));
    test_operator!((-128i8).borrowing_sub(1, true), (126, true));
    test_operator!((-128i8).borrowing_sub(-1, false), (-127, false));
    test_operator!((-128i8).borrowing_sub(-1, true), (-128, false)); // no intermediate overflow
    test_operator!(127i8.borrowing_sub(-1, false), (-128, true));
    test_operator!(127i8.borrowing_sub(-1, true), (127, false)); // no intermediate overflow
}

#[test]
fn custom_extended() {
    // add u8
    test_operator!(255u8.extended_add(1, false), (0, true));
    test_operator!(255u8.extended_add(0, true), (0, true));
    test_operator!(255u8.extended_add(1, true), (1, true));
    test_operator!(0u8.extended_add(255, false), (255, false));
    test_operator!(0u8.extended_add(255, true), (0, true));

    // add i8
    test_operator!(127i8.extended_add(1, false), (-128, true));
    test_operator!(127i8.extended_add(0, true), (-128, true));
    test_operator!(127i8.extended_add(1, true), (-127, true));
    test_operator!(127i8.extended_add(-1, false), (126, false));
    test_operator!(127i8.extended_add(-1, true), (127, false)); // no intermediate overflow
    test_operator!((-128i8).extended_add(-1, false), (127, true));
    test_operator!((-128i8).extended_add(-1, true), (-128, false)); // no intermediate overflow
    test_operator!((0i8).extended_add(127, true), (-128, true));

    // sub u8
    test_operator!(0u8.extended_sub(1, false), (255u8, true));
    test_operator!(0u8.extended_sub(0, true), (255u8, true));
    test_operator!(0u8.extended_sub(1, true), (254u8, true));
    test_operator!(255u8.extended_sub(255, false), (0, false));
    test_operator!(255u8.extended_sub(255, true), (255u8, true));

    // sub i8
    test_operator!((-128i8).extended_sub(1, false), (127, true));
    test_operator!((-128i8).extended_sub(0, true), (127, true));
    test_operator!((-128i8).extended_sub(1, true), (126, true));
    test_operator!((-128i8).extended_sub(-1, false), (-127, false));
    test_operator!((-128i8).extended_sub(-1, true), (-128, false)); // no intermediate overflow
    test_operator!(127i8.extended_sub(-1, false), (-128, true));
    test_operator!(127i8.extended_sub(-1, true), (127, false)); // no intermediate overflow
    test_operator!((0i8).extended_sub(-128, true), (127, false));
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
