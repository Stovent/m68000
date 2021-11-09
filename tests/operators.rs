//! This program tests the various results of the `overflowing_add` and `overflowing_sub` functions.
//!
//! The goal is to ensure that the functions have the behaviour required to correctly
//! compute the CCR bits during instructions.

#![feature(bigint_helper_methods)]

macro_rules! test_operator {
    ($operator:expr, $expected:expr) => {
        let res = $operator;
        assert_eq!(res, $expected, "{} = {:?}, expected {:?}", stringify!($operator), res, $expected);
    }
}

#[test]
fn operators()
{
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
