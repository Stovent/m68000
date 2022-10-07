// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! M68000 status register.

use crate::utils::bits;

/// M68000 status register.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "ffi", repr(C))]
pub struct StatusRegister {
    /// Trace
    pub t: bool,
    /// Supervisor
    pub s: bool,
    /// Interrupt Priority Mask
    pub interrupt_mask: u8,
    /// Extend
    pub x: bool,
    /// Negate
    pub n: bool,
    /// Zero
    pub z: bool,
    /// Overflow
    pub v: bool,
    /// Carry
    pub c: bool,
}

impl StatusRegister {
    const fn t(&self) -> bool {
        true
    }

    const fn f(&self) -> bool {
        false
    }

    const fn hi(&self) -> bool {
        !self.c && !self.z
    }

    const fn ls(&self) -> bool {
        self.c || self.z
    }

    const fn cc(&self) -> bool {
        !self.c
    }

    const fn cs(&self) -> bool {
        self.c
    }

    const fn ne(&self) -> bool {
        !self.z
    }

    const fn eq(&self) -> bool {
        self.z
    }

    const fn vc(&self) -> bool {
        !self.v
    }

    const fn vs(&self) -> bool {
        self.v
    }

    const fn pl(&self) -> bool {
        !self.n
    }

    const fn mi(&self) -> bool {
        self.n
    }

    const fn ge(&self) -> bool {
        self.n && self.v || !self.n && !self.v
    }

    const fn lt(&self) -> bool {
        self.n && !self.v || !self.n && self.v
    }

    const fn gt(&self) -> bool {
        self.n && self.v && !self.z || !self.n && !self.v && !self.z
    }

    const fn le(&self) -> bool {
        self.z || self.n && !self.v || !self.n && self.v
    }

    const CONDITIONS: [fn(&Self) -> bool; 16] = [
        Self::t, Self::f, Self::hi, Self::ls, Self::cc, Self::cs, Self::ne, Self::eq,
        Self::vc, Self::vs, Self::pl, Self::mi, Self::ge, Self::lt, Self::gt, Self::le,
    ];

    pub fn condition(&self, cc: u8) -> bool {
        Self::CONDITIONS[cc as usize](self)
    }

    pub fn set_ccr(&mut self, sr: u16) {
        self.x = bits(sr, 4, 4) != 0;
        self.n = bits(sr, 3, 3) != 0;
        self.z = bits(sr, 2, 2) != 0;
        self.v = bits(sr, 1, 1) != 0;
        self.c = bits(sr, 0, 0) != 0;
    }
}

impl From<u16> for StatusRegister {
    fn from(sr: u16) -> Self {
        Self {
            t: bits(sr, 15, 15) != 0,
            s: bits(sr, 13, 13) != 0,
            interrupt_mask: bits(sr, 8, 10) as u8,
            x: bits(sr, 4, 4) != 0,
            n: bits(sr, 3, 3) != 0,
            z: bits(sr, 2, 2) != 0,
            v: bits(sr, 1, 1) != 0,
            c: bits(sr, 0, 0) != 0,
        }
    }
}

impl Into<u16> for StatusRegister {
    fn into(self) -> u16 {
        (self.t as u16) << 15 |
        (self.s as u16) << 13 |
        (self.interrupt_mask as u16) << 8 |
        (self.x as u16) << 4 |
        (self.n as u16) << 3 |
        (self.z as u16) << 2 |
        (self.v as u16) << 1 |
        (self.c as u16)
    }
}

impl std::ops::BitAndAssign<u16> for StatusRegister {
    fn bitand_assign(&mut self, rhs: u16) {
        self.t = self.t && bits(rhs, 15, 15) != 0;
        self.s = self.s && bits(rhs, 13, 13) != 0;
        self.interrupt_mask &= bits(rhs, 8, 10) as u8;
        self.x = self.x && bits(rhs, 4, 4) != 0;
        self.n = self.n && bits(rhs, 3, 3) != 0;
        self.z = self.z && bits(rhs, 2, 2) != 0;
        self.v = self.v && bits(rhs, 1, 1) != 0;
        self.c = self.c && bits(rhs, 0, 0) != 0;
    }
}

impl std::ops::BitOrAssign<u16> for StatusRegister {
    fn bitor_assign(&mut self, rhs: u16) {
        self.t = self.t || bits(rhs, 15, 15) != 0;
        self.s = self.s || bits(rhs, 13, 13) != 0;
        self.interrupt_mask |= bits(rhs, 8, 10) as u8;
        self.x = self.x || bits(rhs, 4, 4) != 0;
        self.n = self.n || bits(rhs, 3, 3) != 0;
        self.z = self.z || bits(rhs, 2, 2) != 0;
        self.v = self.v || bits(rhs, 1, 1) != 0;
        self.c = self.c || bits(rhs, 0, 0) != 0;
    }
}

impl std::ops::BitXorAssign<u16> for StatusRegister {
    fn bitxor_assign(&mut self, rhs: u16) {
        self.t = (self.t as u16 ^ bits(rhs, 15, 15)) != 0;
        self.s = (self.s as u16 ^ bits(rhs, 13, 13)) != 0;
        self.interrupt_mask ^= bits(rhs, 8, 10) as u8;
        self.x = (self.x as u16 ^ bits(rhs, 4, 4)) != 0;
        self.n = (self.n as u16 ^ bits(rhs, 3, 3)) != 0;
        self.z = (self.z as u16 ^ bits(rhs, 2, 2)) != 0;
        self.v = (self.v as u16 ^ bits(rhs, 1, 1)) != 0;
        self.c = (self.c as u16 ^ bits(rhs, 0, 0)) != 0;
    }
}

pub(crate) fn disassemble_conditional_test(test: u8) -> &'static str {
    match test {
        0  => "T",
        1  => "F",
        2  => "HI",
        3  => "LS",
        4  => "CC",
        5  => "CS",
        6  => "NE",
        7  => "EQ",
        8  => "VC",
        9  => "VS",
        10 => "PL",
        11 => "MI",
        12 => "GE",
        13 => "LT",
        14 => "GT",
        15 => "LE",
        _ => "Unknown",
    }
}
