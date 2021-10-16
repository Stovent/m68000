use super::utils::bits;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    fn t(&self) -> bool {
        true
    }

    fn f(&self) -> bool {
        false
    }

    fn hi(&self) -> bool {
        !self.c && !self.v
    }

    fn ls(&self) -> bool {
        self.c || self.v
    }

    fn cc(&self) -> bool {
        !self.c
    }

    fn cs(&self) -> bool {
        self.c
    }

    fn ne(&self) -> bool {
        !self.z
    }

    fn eq(&self) -> bool {
        self.z
    }

    fn vc(&self) -> bool {
        !self.v
    }

    fn vs(&self) -> bool {
        self.v
    }

    fn pl(&self) -> bool {
        !self.n
    }

    fn mi(&self) -> bool {
        self.n
    }

    fn ge(&self) -> bool {
        self.n && self.v || !self.n && !self.v
    }

    fn lt(&self) -> bool {
        self.n && !self.v || !self.n && self.v
    }

    fn gt(&self) -> bool {
        self.n && self.v && !self.z || !self.n && !self.v && !self.z
    }

    fn le(&self) -> bool {
        self.z || self.n && !self.v || !self.n && self.v
    }

    pub const CONDITIONS: [fn(&Self) -> bool; 16] = [
        Self::t, Self::f, Self::hi, Self::ls, Self::cc, Self::cs, Self::ne, Self::eq,
        Self::vc, Self::vs, Self::pl, Self::mi, Self::ge, Self::lt, Self::gt, Self::le,
    ];
}

impl Default for StatusRegister {
    fn default() -> Self {
        Self {
            t: false,
            s: false,
            interrupt_mask: 0,
            x: false,
            n: false,
            z: false,
            v: false,
            c: false,
        }
    }
}

impl From<u16> for StatusRegister {
    fn from(sr: u16) -> Self {
        Self {
            t: sr & 0x8000 != 0,
            s: sr & 0x2000 != 0,
            interrupt_mask: bits(sr, 8, 10) as u8,
            x: sr & 0x0010 != 0,
            n: sr & 0x0008 != 0,
            z: sr & 0x0004 != 0,
            v: sr & 0x0002 != 0,
            c: sr & 0x0001 != 0,
        }
    }
}

pub(super) fn disassemble_conditional_test(test: u8) -> &'static str {
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
