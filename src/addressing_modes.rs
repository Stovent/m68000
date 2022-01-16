//! Addressing mode-related structs, enums and functions.

use crate::M68000;
use crate::memory_access::U16Iter;
use crate::instruction::Size;
use crate::utils::{AsArray, bits, SliceAs};

/// The `mode` part of an effective address field.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum AddressingMode {
    /// Data Register Direct
    Drd = 0,
    /// Address Register Direct
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

/// Register number for Absolute Short addressing mode.
pub const ABSOLUTE_SHORT: u8 = 0;
/// Register number for Absolute Long addressing mode.
pub const ABSOLUTE_LONG: u8 = 1;
/// Register number for Program Counter Indirect With Displacement addressing mode.
pub const PCIWD: u8 = 2;
/// Register number for Program Counter Indirect With Index 8 addressing mode.
pub const PCIWI8: u8 = 3;
/// Register number for Immediate Data addressing mode.
pub const IMMEDIATE_DATA: u8 = 4;

impl AddressingMode {
    /// Returns true if `self` is `Drd`, false otherwise.
    #[inline(always)]
    pub fn drd(self) -> bool {
        self == Self::Drd
    }

    /// Returns true if `self` is `Ard`, false otherwise.
    #[inline(always)]
    pub fn ard(self) -> bool {
        self == Self::Ard
    }

    /// Returns true if `self` is `Ariwpo`, false otherwise.
    #[inline(always)]
    pub fn ariwpo(self) -> bool {
        self == Self::Ariwpo
    }

    /// Returns true if `self` is `Ariwpr`, false otherwise.
    #[inline(always)]
    pub fn ariwpr(self) -> bool {
        self == Self::Ariwpr
    }

    /// Returns true if `self` is `Mode7`, false otherwise.
    #[inline(always)]
    pub fn mode7(self) -> bool {
        self == Self::Mode7
    }
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

/// Represents an effective address, with mode, register, size and extension words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EffectiveAddress {
    /// The addressing mode.
    pub mode: AddressingMode,
    /// The addressing register.
    pub reg: u8,
    /// The address of the extension word.
    pub pc: u32,
    /// Where this effective address points to. `None` if the value is not in memory.
    pub address: Option<u32>,
    /// The size of the data.
    pub size: Option<Size>,
    /// The extension words.
    pub ext: Box<[u8]>
}

impl EffectiveAddress {
    /// New effective address with an empty `address` field.
    pub fn new(mode: AddressingMode, reg: u8, size: Option<Size>, memory: &mut dyn U16Iter) -> Self {
        let (ext, pc): (Box<[u8]>, u32) = match mode {
            AddressingMode::Ari => (Box::new([]), 0),
            AddressingMode::Ariwpo => (Box::new([]), 0),
            AddressingMode::Ariwpr => (Box::new([]), 0),
            AddressingMode::Ariwd  => {
                let pc = memory.next_addr();
                (Box::new(memory.next().unwrap().unwrap_or_else(|e| panic!("Failed to get displacement at {:#X}: {}", pc, e)).as_array_be()), pc)
            },
            AddressingMode::Ariwi8 => {
                let pc = memory.next_addr();
                (Box::new(memory.next().unwrap().unwrap_or_else(|e| panic!("Failed to get index 8 at {:#X}: {}", pc, e)).as_array_be()), pc)
            },
            AddressingMode::Mode7 => match reg {
                ABSOLUTE_SHORT => {
                    let pc = memory.next_addr();
                    (Box::new(memory.next().unwrap().unwrap_or_else(|e| panic!("Failed to get absolute short at {:#X}: {}", pc, e)).as_array_be()), pc)
                },
                ABSOLUTE_LONG => {
                    let pc = memory.next_addr();
                    let high = memory.next().unwrap().unwrap_or_else(|e| panic!("Failed to get absolute long high at {:#X}: {}", pc, e));
                    let low = memory.next().unwrap().unwrap_or_else(|e| panic!("Failed to get absolute long low at {:#X}: {}", pc, e));
                    (Box::new(((high as u32) << 16 | low as u32).as_array_be()), pc)
                },
                PCIWD => {
                    let pc = memory.next_addr();
                    (Box::new(memory.next().unwrap().unwrap_or_else(|e| panic!("Failed to get PC displacement at {:#X}: {}", pc, e)).as_array_be()), pc)
                },
                PCIWI8 => {
                    let pc = memory.next_addr();
                    (Box::new(memory.next().unwrap().unwrap_or_else(|e| panic!("Failed to get PC index 8 at {:#X}: {}", pc, e)).as_array_be()), pc)
                },
                IMMEDIATE_DATA => {
                    let pc = memory.next_addr();
                    if size.unwrap().long() {
                        let high = memory.next().unwrap().unwrap_or_else(|e| panic!("Failed to get immediate data high at {:#X}: {}", pc, e));
                        let low = memory.next().unwrap().unwrap_or_else(|e| panic!("Failed to get immediate data low at {:#X}: {}", pc, e));
                        (Box::new(((high as u32) << 16 | low as u32).as_array_be()), pc)
                    } else {
                        (Box::new(memory.next().unwrap().unwrap_or_else(|e| panic!("Failed to get immediate data at {:#X}: {}", pc, e)).as_array_be()), pc)
                    }
                },
                _ => (Box::new([]), 0),
            },
            _ => (Box::new([]), 0),
        };

        Self {
            mode,
            reg,
            pc,
            address: None,
            size,
            ext,
        }
    }

    /// New effective address with mode and reg pulled from the lower 6 bits.
    pub fn from_opcode(opcode: u16, size: Option<Size>, memory: &mut dyn U16Iter) -> Self {
        let reg = bits(opcode, 0, 2) as u8;
        let mode = AddressingMode::from(bits(opcode, 3, 5));
        Self::new(mode, reg, size, memory)
    }

    /// Returns the destination (left tuple) and source (right tuple) effective addresses from a `MOVE` instruction opcode.
    pub fn from_move(opcode: u16, size: Option<Size>, memory: &mut dyn U16Iter) -> (Self, Self) {
        let reg = bits(opcode, 0, 2) as u8;
        let mode = AddressingMode::from(bits(opcode, 3, 5));
        let src = Self::new(mode, reg, size, memory);

        let mode = AddressingMode::from(bits(opcode, 6, 8));
        let reg = bits(opcode, 9, 11) as u8;
        let dst = Self::new(mode, reg, size, memory);
        (dst, src)
    }
}

impl std::fmt::Display for EffectiveAddress {
    /// Disassembles the effective address field.
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
                3 => write!(f, "({}, PC, {})", self.ext[1] as i8, disassemble_index_register(self.ext.u16_be())),
                4 => write!(f, "#{}", self.ext.i32_be_sized(self.size.expect("No associated size with immediate operand"))),
                _ => write!(f, "Unknown addressing mode {} reg {}", self.mode as usize, self.reg),
            },
        }
    }
}

impl std::fmt::UpperHex for EffectiveAddress {
    /// Same as Display but with the immediate value written in upper hex format.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.mode, self.reg) {
            (AddressingMode::Mode7, 4) => write!(f, "#{:#X}", self.ext.i32_be_sized(self.size.expect("No associated size with immediate operand"))),
            _ => std::fmt::Display::fmt(&self, f),
        }
    }
}

/// Disassembles the index register field of a brief extension word.
fn disassemble_index_register(bew: u16) -> String {
    let x = if bew & 0x8000 != 0 { "A" } else { "D" };
    let reg = bits(bew, 12, 14);
    let size = if bew & 0x0800 != 0 { "L" } else { "W" };
    format!("{}{}.{}", x, reg, size)
}

impl M68000 {
    /// Calculates the value of the given effective address.
    ///
    /// If the address has already been calculated (`ea.address` is Some), it is returned and no computation is performed.
    /// Otherwise the address is computed and assigned to `ea.address` and returned, or None if the addressing mode is not in memory.
    pub(super) fn get_effective_address(&mut self, ea: &mut EffectiveAddress) -> Option<u32> {
        if ea.address == None {
            ea.address = match ea.mode {
                AddressingMode::Ari => Some(self.a(ea.reg)),
                AddressingMode::Ariwpo => Some(self.ariwpo(ea.reg, ea.size.expect("ariwpo must have a size"))),
                AddressingMode::Ariwpr => Some(self.ariwpr(ea.reg, ea.size.expect("ariwpr must have a size"))),
                AddressingMode::Ariwd  => {
                    let a = self.a(ea.reg);
                    let disp = ea.ext.u16_be() as i16 as u32;
                    Some(a + disp)
                },
                AddressingMode::Ariwi8 => {
                    let a = self.a(ea.reg);
                    let bew = ea.ext.u16_be();
                    let disp = bew as i8 as u32;
                    Some(a + disp + self.get_index_register(bew))
                },
                AddressingMode::Mode7 => match ea.reg {
                    0 => {
                        let a = ea.ext.u16_be() as i16 as u32;
                        Some(a)
                    },
                    1 => {
                        let a = ea.ext.u32_be();
                        Some(a)
                    },
                    2 => {
                        let disp = ea.ext.u16_be() as i16 as u32;
                        Some(ea.pc + disp)
                    },
                    3 => {
                        let bew = ea.ext.u16_be();
                        let disp = bew as i8 as u32;
                        Some(ea.pc + disp + self.get_index_register(bew))
                    },
                    _ => None,
                },
                _ => None,
            };
        }
        ea.address
    }

    fn get_index_register(&self, bew: u16) -> u32 {
        let reg = bits(bew, 12, 14) as u8;
        if bew & 0x8000 != 0 { // Address register
            if bew & 0x0800 != 0 { // Long
                self.a(reg)
            } else { // Word
                self.a(reg) as i16 as u32
            }
        } else { // Data register
            if bew & 0x0800 != 0 { // Long
                self.d[reg as usize]
            } else { // Word
                self.d[reg as usize] as i16 as u32
            }
        }
    }

    /// Address Register Indirect With POstincrement
    pub(super) fn ariwpo(&mut self, reg: u8, size: Size) -> u32 {
        let areg = self.a_mut(reg);
        let addr = *areg;
        *areg += if reg == 7 { size.as_word_long() } else { size } as u32;
        addr
    }

    /// Address Register Indirect With PRedecrement
    pub(super) fn ariwpr(&mut self, reg: u8, size: Size) -> u32 {
        let areg = self.a_mut(reg);
        *areg -= if reg == 7 { size.as_word_long() } else { size } as u32;
        *areg
    }
}
