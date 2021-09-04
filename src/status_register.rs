use super::utils::Bits;

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
    // pub fn set_ccr(&mut self, ccr: u8) {
    //     self.x = ccr & 0x10 != 0;
    //     self.n = ccr & 0x08 != 0;
    //     self.z = ccr & 0x04 != 0;
    //     self.v = ccr & 0x02 != 0;
    //     self.c = ccr & 0x01 != 0;
    // }
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
            interrupt_mask: sr.bits::<8, 10>() as u8,
            x: sr & 0x0010 != 0,
            n: sr & 0x0008 != 0,
            z: sr & 0x0004 != 0,
            v: sr & 0x0002 != 0,
            c: sr & 0x0001 != 0,
        }
    }
}
