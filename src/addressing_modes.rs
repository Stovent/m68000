//! Addressing mode-related structs, enums and functions.

use crate::M68000;
use crate::memory_access::MemoryIter;
use crate::instruction::Size;
use crate::utils::bits;

/// Addressing modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AddressingMode {
    /// Data Register Direct.
    Drd(u8),
    /// Address Register Direct.
    Ard(u8),
    /// Address Register Indirect.
    Ari(u8),
    /// Address Register Indirect With POstincrement.
    Ariwpo(u8),
    /// Address Register Indirect With PRedecrement.
    Ariwpr(u8),
    /// Address Register Indirect With Displacement (address reg, disp).
    Ariwd(u8, i16),
    /// Address Register Indirect With Index 8 (address reg, index reg).
    Ariwi8(u8, IndexRegister),
    /// Absolute Short.
    AbsShort(u16),
    /// Absolute Long.
    AbsLong(u32),
    /// Program Counter Indirect With Displacement.
    Pciwd(i16),
    /// Program Counter Indirect With Index 8.
    Pciwi8(IndexRegister),
    /// Immediate Data (cast this variant to the correct type when used).
    Immediate(u32),
}

impl AddressingMode {
    /// New addressing mode.
    pub fn new(mode: u16, reg: u8, size: Option<Size>, memory: &mut MemoryIter) -> Self {
        match mode {
            0 => Self::Drd(reg),
            1 => Self::Ard(reg),
            2 => Self::Ari(reg),
            3 => Self::Ariwpo(reg),
            4 => Self::Ariwpr(reg),
            5 => Self::Ariwd(reg, memory.next().unwrap().unwrap() as i16),
            6 => Self::Ariwi8(reg, IndexRegister(memory.next().unwrap().unwrap())),
            7 => match reg {
                0 => Self::AbsShort(memory.next().unwrap().unwrap()),
                1 => {
                    let high = (memory.next().unwrap().unwrap() as u32) << 16;
                    let low = memory.next().unwrap().unwrap() as u32;
                    Self::AbsLong(high | low)
                },
                2 => Self::Pciwd(memory.next().unwrap().unwrap() as i16),
                3 => Self::Pciwi8(IndexRegister(memory.next().unwrap().unwrap())),
                4 => {
                    if size.unwrap().is_long() {
                        let high = (memory.next().unwrap().unwrap() as u32) << 16;
                        let low = memory.next().unwrap().unwrap() as u32;
                        Self::Immediate(high | low)
                    } else {
                        let low = memory.next().unwrap().unwrap() as u32;
                        Self::Immediate(low as u32)
                    }
                },
                _ => panic!("[AddressingMode::new] Wrong register {}", reg),
            },
            _ => panic!("[AddressingMode::new] Wrong mode {}", mode),
        }
    }

    /// Return the register of the addressing mode, or None if the mode has no associated register.
    #[inline(always)]
    pub const fn register(self) -> Option<u8> {
        match self {
            AddressingMode::Drd(reg) => Some(reg),
            AddressingMode::Ard(reg) => Some(reg),
            AddressingMode::Ari(reg) => Some(reg),
            AddressingMode::Ariwpo(reg) => Some(reg),
            AddressingMode::Ariwpr(reg) => Some(reg),
            AddressingMode::Ariwd(reg, _)  => Some(reg),
            AddressingMode::Ariwi8(reg, _) => Some(reg),
            _ => None,
        }
    }

    /// Returns true if `self` is `Drd`, false otherwise.
    #[inline(always)]
    pub const fn is_drd(self) -> bool {
        match self {
            Self::Drd(_) => true,
            _ => false,
        }
    }

    /// Returns true if `self` is `Ard`, false otherwise.
    #[inline(always)]
    pub const fn is_ard(self) -> bool {
        match self {
            Self::Ard(_) => true,
            _ => false,
        }
    }

    /// Returns true if `self` is `Ariwpo`, false otherwise.
    #[inline(always)]
    pub const fn is_ariwpo(self) -> bool {
        match self {
            Self::Ariwpo(_) => true,
            _ => false,
        }
    }

    /// Returns true if `self` is `Ariwpr`, false otherwise.
    #[inline(always)]
    pub const fn is_ariwpr(self) -> bool {
        match self {
            Self::Ariwpr(_) => true,
            _ => false,
        }
    }
}

impl std::fmt::Display for AddressingMode {
    /// Disassembles the addressing mode.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddressingMode::Drd(reg) => write!(f, "D{}", reg),
            AddressingMode::Ard(reg) => write!(f, "A{}", reg),
            AddressingMode::Ari(reg) => write!(f, "(A{})", reg),
            AddressingMode::Ariwpo(reg) => write!(f, "(A{})+", reg),
            AddressingMode::Ariwpr(reg) => write!(f, "-(A{})", reg),
            AddressingMode::Ariwd(reg, disp) => write!(f, "({}, A{})", disp, reg),
            AddressingMode::Ariwi8(reg, index) => write!(f, "({}, A{}, {})", index.disp(), reg, index),
            AddressingMode::AbsShort(addr) => write!(f, "({:#X}).W", addr),
            AddressingMode::AbsLong(addr) => write!(f, "({:#X}).L", addr),
            AddressingMode::Pciwd(disp) => write!(f, "({}, PC)", disp),
            AddressingMode::Pciwi8(index) => write!(f, "({}, PC, {})", index.disp(), index),
            AddressingMode::Immediate(imm) => write!(f, "#{}", imm),
        }
    }
}

impl std::fmt::UpperHex for AddressingMode {
    /// Same as Display but with the mode 7 immediate value written in upper hex format.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddressingMode::Immediate(imm) => write!(f, "#{:#X}", imm),
            _ => std::fmt::Display::fmt(self, f),
        }
    }
}

/// Index register.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IndexRegister(u16);

impl IndexRegister {
    pub const fn disp(self) -> i8 {
        self.0 as i8
    }
}

impl std::fmt::Display for IndexRegister {
    /// Disassembles the index register field of a brief extension word.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x = if self.0 & 0x8000 != 0 { "A" } else { "D" };
        let reg = bits(self.0, 12, 14);
        let size = if self.0 & 0x0800 != 0 { "L" } else { "W" };
        write!(f, "{}{}.{}", x, reg, size)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EffectiveAddress {
    /// The addressing mode.
    pub mode: AddressingMode,
    /// The address of the extension word.
    pub pc: u32,
    /// Where this effective address points to. `None` if the value is not in memory.
    pub address: Option<u32>,
    /// The size of the data.
    pub size: Option<Size>,
}

impl EffectiveAddress {
    /// New effective address with mode and reg pulled from the lower 6 bits with an empty `address` field.
    pub fn from_opcode(opcode: u16, size: Option<Size>, memory: &mut MemoryIter) -> Self {
        let pc = memory.next_addr;
        let mode = bits(opcode, 3, 5);
        let reg = bits(opcode, 0, 2) as u8;
        let am = AddressingMode::new(mode, reg, size, memory);

        Self {
            mode: am,
            pc,
            address: None,
            size,
        }
    }

    /// Returns the destination (left tuple) and source (right tuple) effective addresses from a `MOVE` instruction opcode.
    pub fn from_move(opcode: u16, size: Option<Size>, memory: &mut MemoryIter) -> (Self, Self) {
        // First read the source operand then the destination.
        let pc = memory.next_addr;
        let mode = bits(opcode, 3, 5);
        let reg = bits(opcode, 0, 2) as u8;
        let am = AddressingMode::new(mode, reg, size, memory);
        let src = Self {
            mode: am,
            pc,
            address: None,
            size,
        };

        let pc = memory.next_addr;
        let reg = bits(opcode, 9, 11) as u8;
        let mode = bits(opcode, 6, 8);
        let am = AddressingMode::new(mode, reg, size, memory);
        let dst = Self {
            mode: am,
            pc,
            address: None,
            size,
        };

        (dst, src)
    }
}

impl M68000 {
    /// Calculates the value of the given effective address.
    ///
    /// If the address has already been calculated (`ea.address` is Some), it is returned and no computation is performed.
    /// Otherwise the address is computed and assigned to `ea.address` and returned, or None if the addressing mode is not in memory.
    pub(super) fn get_effective_address(&mut self, ea: &mut EffectiveAddress) -> Option<u32> {
        if ea.address.is_none() {
            ea.address = match ea.mode {
                AddressingMode::Ari(reg) => Some(self.a(reg)),
                AddressingMode::Ariwpo(reg) => Some(self.ariwpo(reg, ea.size.expect("ariwpo must have a size"))),
                AddressingMode::Ariwpr(reg) => Some(self.ariwpr(reg, ea.size.expect("ariwpr must have a size"))),
                AddressingMode::Ariwd(reg, disp)  => Some(self.a(reg) + disp as u32),
                AddressingMode::Ariwi8(reg, index) => Some(self.a(reg) + index.disp() as u32 + self.get_index_register(index)),
                AddressingMode::AbsShort(addr) => Some(addr as i16 as u32),
                AddressingMode::AbsLong(addr) => Some(addr),
                AddressingMode::Pciwd(disp) => Some(ea.pc + disp as u32),
                AddressingMode::Pciwi8(index) => Some(ea.pc + index.disp() as u32 + self.get_index_register(index)),
                _ => None,
            };
        }

        ea.address
    }

    const fn get_index_register(&self, index: IndexRegister) -> u32 {
        let reg = bits(index.0, 12, 14) as u8;

        if index.0 & 0x8000 != 0 { // Address register
            if index.0 & 0x0800 != 0 { // Long
                self.a(reg)
            } else { // Word
                self.a(reg) as i16 as u32
            }
        } else { // Data register
            if index.0 & 0x0800 != 0 { // Long
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
