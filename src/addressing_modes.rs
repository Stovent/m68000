use super::{M68000, MemoryAccess};
use super::operand::Size;
use super::utils::{AsArray, Bits, SliceAs};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum AddressingMode {
    /// Data Register Direct
    Drd = 0,
    // Address Register Direct
    Ard = 1,
    /// Address Register Indirect
    Ari = 2,
    /// Address Register Indirect With POstincrement
    Ariwpo = 3,
    /// Address Register Indirect With PRedecrement
    Ariwpr = 4,
    /// Address Register Indirect With Displacement
    Ariwd = 5,
    /// Address Register Indirect With Index 8
    Ariwi8 = 6,
    /// Mode 7
    Mode7 = 7,
}

impl From<u16> for AddressingMode {
    fn from(d: u16) -> Self {
        match d {
            0 => Self::Drd,
            1 => Self::Ard,
            2 => Self::Ari,
            3 => Self::Ariwpo,
            4 => Self::Ariwpr,
            5 => Self::Ariwd,
            6 => Self::Ariwi8,
            7 => Self::Mode7,
            _ => panic!("[AddressingMode::from<u16>] Wrong addressing mode {}", d),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EffectiveAddress {
    /// The address of the operand. None if the value is not in memory.
    pub address: Option<u32>,
    /// The addressing mode.
    pub mode: AddressingMode,
    /// The addressing register.
    pub reg: usize,
    /// The size of the data.
    pub size: Size,
    /// The extension words.
    pub ext: Box<[u8]>
}

impl std::fmt::Display for EffectiveAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.mode {
            AddressingMode::Drd => write!(f, "D{}", self.reg),
            AddressingMode::Ard => write!(f, "A{}", self.reg),
            AddressingMode::Ari => write!(f, "(A{})", self.reg),
            AddressingMode::Ariwpo => write!(f, "(A{})+", self.reg),
            AddressingMode::Ariwpr => write!(f, "-(A{})", self.reg),
            AddressingMode::Ariwd => write!(f, "({}, A{})", self.ext.u16_be() as i16, self.reg),
            AddressingMode::Ariwi8 => write!(f, "({}, A{}, {})", self.ext[1] as i8, self.reg, disassemble_index_register(self.ext.u16_be())),
            AddressingMode::Mode7 => match self.reg {
                0 => write!(f, "({:#X}).W", self.ext.u16_be()),
                1 => write!(f, "({:#X}).L", self.ext.u32_be()),
                2 => write!(f, "({}, PC)", self.ext.u16_be() as i16),
                3 => write!(f, "({}, PC, {}", self.ext[1] as i8, disassemble_index_register(self.ext.u16_be())),
                4 => write!(f, "#{}", self.ext.i32_be_sized(self.size)),
                _ => write!(f, "Unknown addressing mode {} reg {}", self.mode as usize, self.reg),
            },
        }
    }
}

fn disassemble_index_register(bew: u16) -> String {
    let x = if bew & 0x8000 != 0 { "A" } else { "D" };
    let reg = bew.bits::<12, 14>();
    let size = if bew & 0x0800 != 0 { "L" } else { "W" };
    format!("{}{}.{}", x, reg, size)
}

impl<M: MemoryAccess> M68000<M> {
    /// Returns the effective address based on the addressing mode.
    ///
    /// Self::address contains the effective address, or None if the addressing mode is not in memory.
    pub(super) fn get_effective_address(&mut self, mode: AddressingMode, reg: usize, size: Size) -> EffectiveAddress {
        let (address, ext): (Option<u32>, Box<[u8]>) = match mode {
            AddressingMode::Ari => (Some(self.a(reg)), Box::new([])),
            AddressingMode::Ariwpo => (Some(self.ariwpo(reg, size)), Box::new([])),
            AddressingMode::Ariwpr => (Some(self.ariwpr(reg, size)), Box::new([])),
            AddressingMode::Ariwd  => {
                let (a, ext) = self.ariwd(reg);
                (Some(a), Box::new(ext.as_array_be()))
            },
            AddressingMode::Ariwi8 => {
                let (a, ext) = self.ariwi8(reg);
                (Some(a), Box::new(ext.as_array_be()))
            },
            AddressingMode::Mode7 => match reg {
                0 => {
                    let a = self.asa();
                    (Some(a), Box::new((a as u16).as_array_be()))
                },
                1 => {
                    let a = self.ala();
                    (Some(a), Box::new(a.as_array_be()))
                },
                2 => {
                    let (a, ext) = self.pciwd();
                    (Some(a), Box::new(ext.as_array_be()))
                },
                3 => {
                    let (a, ext) = self.pciwi8();
                    (Some(a), Box::new(ext.as_array_be()))
                },
                _ => (None, Box::new([])),
            },
            _ => (None, Box::new([])),
        };
        EffectiveAddress {
            address,
            mode,
            reg,
            size,
            ext,
        }
    }

    fn get_index_register(&self, bew: u16) -> u32 {
        let reg = bew.bits::<12, 14>() as usize;
        if bew & 0x8000 != 0 { // Address register
            if bew & 0x0800 != 0 { // Long
                self.a(reg)
            } else { // Word
                self.a(reg) as i16 as u32
            }
        } else { // Data register
            if bew & 0x0800 != 0 { // Long
                self.d[reg]
            } else { // Word
                self.d[reg] as i16 as u32
            }
        }
    }

    /// Address Register Indirect With POstincrement
    fn ariwpo(&mut self, reg: usize, size: Size) -> u32 {
        let areg = self.a_mut(reg);
        let addr = *areg;
        *areg += if reg == 7 { size.as_word_long() } else { size } as u32;
        addr
    }

    /// Address Register Indirect With PRedecrement
    fn ariwpr(&mut self, reg: usize, size: Size) -> u32 {
        let areg = self.a_mut(reg);
        *areg -= if reg == 7 { size.as_word_long() } else { size } as u32;
        *areg
    }

    /// Address Register Indirect With Displacement
    ///
    /// Returns the effective address and the extension word.
    fn ariwd(&mut self, reg: usize) -> (u32, u16) {
        let ext = self.get_next_word();
        let disp = ext as i16 as u32;
        (self.a(reg) + disp, ext)
    }

    /// Address Register Indirect With Index 8
    ///
    /// Returns the effective address and the extension word.
    fn ariwi8(&mut self, reg: usize) -> (u32, u16) {
        let bew = self.get_next_word();
        let disp = bew as i8 as u32;
        (self.a(reg) + disp + self.get_index_register(bew), bew)
    }

    /// Program Counter Indirect With Displacement
    ///
    /// Returns the effective address and the extension word.
    fn pciwd(&mut self) -> (u32, u16) {
        let pc = self.pc;
        let ext = self.get_next_word();
        let disp = ext as i16 as u32;
        (pc + disp, ext)
    }

    /// Program Counter Indirect With Index 8
    ///
    /// Returns the effective address and the extension word.
    fn pciwi8(&mut self) -> (u32, u16) {
        let pc = self.pc;
        let bew = self.get_next_word();
        let disp = bew as i8 as u32;
        (pc + disp + self.get_index_register(bew), bew)
    }

    /// Absolute Short Addressing
    fn asa(&mut self) -> u32 {
        self.get_next_word() as i16 as u32
    }

    /// Absolute Long Addressing
    fn ala(&mut self) -> u32 {
        self.get_next_long()
    }
}
