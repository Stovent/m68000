// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Addressing mode-related structs, enums and functions.

use crate::{CpuDetails, M68000, MemoryAccess};
use crate::memory_access::MemoryIter;
use crate::instruction::Size;
use crate::utils::{bit, bits};

/// Addressing modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "ffi", repr(C))]
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
    /// Address Register Indirect With Displacement (address reg, displacement).
    Ariwd(u8, i16),
    /// Address Register Indirect With Index 8 (address reg, brief extension word).
    Ariwi8(u8, BriefExtensionWord),
    /// Absolute Short.
    AbsShort(u16),
    /// Absolute Long.
    AbsLong(u32),
    /// Program Counter Indirect With Displacement (PC value, displacement).
    ///
    /// When using it with the assembler, the PC value is ignored.
    Pciwd(u32, i16),
    /// Program Counter Indirect With Index 8 (PC value, brief extension word).
    ///
    /// When using it with the assembler, the PC value is ignored.
    Pciwi8(u32, BriefExtensionWord),
    /// Immediate Data (cast this variant to the correct type when used).
    Immediate(u32),
}

impl AddressingMode {
    /// New addressing mode from memory iterator.
    pub fn from_memory<M: MemoryAccess + ?Sized>(mode: u16, reg: u8, size: Option<Size>, memory: &mut MemoryIter<M>) -> Self {
        match mode {
            0 => Self::Drd(reg),
            1 => Self::Ard(reg),
            2 => Self::Ari(reg),
            3 => Self::Ariwpo(reg),
            4 => Self::Ariwpr(reg),
            5 => Self::Ariwd(reg, memory.next().unwrap().expect("An Access Error occured in Ariwd.") as i16),
            6 => Self::Ariwi8(reg, BriefExtensionWord(memory.next().unwrap().expect("An Access Error occured in Ariwi8."))),
            7 => match reg {
                0 => Self::AbsShort(memory.next().unwrap().expect("An Access Error occured in AbsShort.")),
                1 => {
                    let high = (memory.next().unwrap().expect("An Access Error occured in AbsLong high.") as u32) << 16;
                    let low = memory.next().unwrap().expect("An Access Error occured in AbsLong low.") as u32;
                    Self::AbsLong(high | low)
                },
                2 => Self::Pciwd(memory.next_addr, memory.next().unwrap().expect("An Access Error occured in Pciwd.") as i16),
                3 => Self::Pciwi8(memory.next_addr, BriefExtensionWord(memory.next().unwrap().expect("An Access Error occured in Pciwi8."))),
                4 => {
                    if size.unwrap().is_long() {
                        let high = (memory.next().unwrap().expect("An Access Error occured in Immediate high.") as u32) << 16;
                        let low = memory.next().unwrap().expect("An Access Error occured in Immediate low.") as u32;
                        Self::Immediate(high | low)
                    } else {
                        let low = memory.next().unwrap().expect("An Access Error occured in Immediate.");
                        Self::Immediate(low as u32)
                    }
                },
                _ => panic!("[AddressingMode::from_memory] Wrong register {reg}"),
            },
            _ => panic!("[AddressingMode::from_memory] Wrong mode {mode}"),
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
        matches!(self, Self::Drd(_))
    }

    /// Returns true if `self` is `Ard`, false otherwise.
    #[inline(always)]
    pub const fn is_ard(self) -> bool {
        matches!(self, Self::Ard(_))
    }

    /// Returns true if `self` is `Drd` or `Ard`, false otherwise.
    #[inline(always)]
    pub const fn is_dard(self) -> bool {
        matches!(self, Self::Drd(_) | Self::Ard(_))
    }

    /// Returns true if `self` is `Ariwpo`, false otherwise.
    #[inline(always)]
    pub const fn is_ariwpo(self) -> bool {
        matches!(self, Self::Ariwpo(_))
    }

    /// Returns true if `self` is `Ariwpr`, false otherwise.
    #[inline(always)]
    pub const fn is_ariwpr(self) -> bool {
        matches!(self, Self::Ariwpr(_))
    }

    /// Returns true if `self` is `Immediate`, false otherwise.
    #[inline(always)]
    pub const fn is_immediate(self) -> bool {
        matches!(self, Self::Immediate(_))
    }

    /// Assembles `self` as an opcode effective address field.
    ///
    /// Set `long` to true if the immediate operand is long, false for byte and word sizes.
    ///
    /// Left tuple contains the mode and register encoded as in the low 6 bits of the opcode.
    /// Right tuple contains the extension words.
    pub fn assemble(self, long: bool) -> (u16, Box<[u16]>) {
        match self {
            AddressingMode::Drd(reg) => (reg as u16, Box::new([])),
            AddressingMode::Ard(reg) => (1 << 3 | reg as u16, Box::new([])),
            AddressingMode::Ari(reg) => (2 << 3 | reg as u16, Box::new([])),
            AddressingMode::Ariwpo(reg) => (3 << 3 | reg as u16, Box::new([])),
            AddressingMode::Ariwpr(reg) => (4 << 3 | reg as u16, Box::new([])),
            AddressingMode::Ariwd(reg, disp) => (5 << 3 | reg as u16, Box::new([disp as u16])),
            AddressingMode::Ariwi8(reg, bew) => (6 << 3 | reg as u16, Box::new([bew.0])),
            AddressingMode::AbsShort(addr) => (7 << 3, Box::new([addr])),
            AddressingMode::AbsLong(addr) => (7 << 3 | 1, Box::new([(addr >> 16) as u16, addr as u16])),
            AddressingMode::Pciwd(_, disp) => (7 << 3 | 2, Box::new([disp as u16])),
            AddressingMode::Pciwi8(_, bew) => (7 << 3 | 3, Box::new([bew.0])),
            AddressingMode::Immediate(imm) => {
                if long {
                    (7 << 3 | 4, Box::new([(imm >> 16) as u16, imm as u16]))
                } else {
                    (7 << 3 | 4, Box::new([imm as u16]))
                }
            },
        }
    }

    /// Assembles `self` as an opcode effective address field for MOVE or MOVEA destination field.
    ///
    /// Set `long` to true if the immediate operand is long, false for byte and word sizes.
    ///
    /// Left tuple contains the mode and register encoded as in the destination (bits 6 to 11).
    /// Right tuple contains the extension words.
    pub fn assemble_move_dst(self) -> (u16, Box<[u16]>) {
        match self {
            AddressingMode::Drd(reg) => ((reg as u16) << 9, Box::new([])),
            AddressingMode::Ard(reg) => ((reg as u16) << 9 | 1 << 6, Box::new([])),
            AddressingMode::Ari(reg) => ((reg as u16) << 9 | 2 << 6, Box::new([])),
            AddressingMode::Ariwpo(reg) => ((reg as u16) << 9 | 3 << 6, Box::new([])),
            AddressingMode::Ariwpr(reg) => ((reg as u16) << 9 | 4 << 6, Box::new([])),
            AddressingMode::Ariwd(reg, disp) => ((reg as u16) << 9 | 5 << 6, Box::new([disp as u16])),
            AddressingMode::Ariwi8(reg, bew) => ((reg as u16) << 9 | 6 << 6, Box::new([bew.0])),
            AddressingMode::AbsShort(addr) => (7 << 6, Box::new([addr])),
            AddressingMode::AbsLong(addr) => (1 << 9 | 7 << 6, Box::new([(addr >> 16) as u16, addr as u16])),
            _ => panic!("{self:?} mode cannot be used as a destination mode."),
        }
    }

    /// Verifies that `self` is one of the given modes.
    ///
    /// `modes` contains the list of valid addressing modes.
    /// `regs` contains the valid register values for Mode 7, if is in `modes`.
    pub fn verify(self, modes: &[u8], regs: &[u8]) -> bool {
        match self {
            AddressingMode::Drd(reg) => reg <= 7 && modes.contains(&0),
            AddressingMode::Ard(reg) => reg <= 7 && modes.contains(&1),
            AddressingMode::Ari(reg) => reg <= 7 && modes.contains(&2),
            AddressingMode::Ariwpo(reg) => reg <= 7 && modes.contains(&3),
            AddressingMode::Ariwpr(reg) => reg <= 7 && modes.contains(&4),
            AddressingMode::Ariwd(reg, _) => reg <= 7 && modes.contains(&5),
            AddressingMode::Ariwi8(reg,_) => reg <= 7 && modes.contains(&6),
            AddressingMode::AbsShort(_) => modes.contains(&7) && regs.contains(&0),
            AddressingMode::AbsLong(_) => modes.contains(&7) && regs.contains(&1),
            AddressingMode::Pciwd(_, _) => modes.contains(&7) && regs.contains(&2),
            AddressingMode::Pciwi8(_, _) => modes.contains(&7) && regs.contains(&3),
            AddressingMode::Immediate(_) => modes.contains(&7) && regs.contains(&4),
        }
    }
}

impl std::fmt::Display for AddressingMode {
    /// Disassembles the addressing mode.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddressingMode::Drd(reg) => write!(f, "D{reg}"),
            AddressingMode::Ard(reg) => write!(f, "A{reg}"),
            AddressingMode::Ari(reg) => write!(f, "(A{reg})"),
            AddressingMode::Ariwpo(reg) => write!(f, "(A{reg})+"),
            AddressingMode::Ariwpr(reg) => write!(f, "-(A{reg})"),
            AddressingMode::Ariwd(reg, disp) => write!(f, "({disp}, A{reg})"),
            AddressingMode::Ariwi8(reg, bew) => write!(f, "({}, A{reg}, {bew})", bew.disp()),
            AddressingMode::AbsShort(addr) => write!(f, "({addr:#X}).W"),
            AddressingMode::AbsLong(addr) => write!(f, "({addr:#X}).L"),
            AddressingMode::Pciwd(_, disp) => write!(f, "({disp}, PC)"),
            AddressingMode::Pciwi8(_, bew) => write!(f, "({}, PC, {bew})", bew.disp()),
            AddressingMode::Immediate(imm) => write!(f, "#{imm}"),
        }
    }
}

impl std::fmt::LowerHex for AddressingMode {
    /// Same as Display but with the mode 7 immediate value written in lower hex format.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddressingMode::Immediate(imm) => write!(f, "#{imm:#x}"),
            _ => std::fmt::Display::fmt(self, f),
        }
    }
}

impl std::fmt::UpperHex for AddressingMode {
    /// Same as Display but with the mode 7 immediate value written in upper hex format.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddressingMode::Immediate(imm) => write!(f, "#{imm:#X}"),
            _ => std::fmt::Display::fmt(self, f),
        }
    }
}

/// Raw Brief Extension Word.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "ffi", repr(C))]
pub struct BriefExtensionWord(pub u16);

impl BriefExtensionWord {
    /// Creates a new brief extension word, to be used when using the assembler.
    ///
    /// - `address`: true if the index register is an address register, false for a data register.
    /// - `reg`: the register number (must be <= 7).
    /// - `long`: true if long size, false for word size.
    /// - `disp:`: the associated displacement value.
    pub const fn new(address: bool, reg: u8, long: bool, disp: i8) -> Self {
        assert!(reg <= 7, "Invalid register");
        let a = (address as u16) << 15;
        let r = (reg as u16) << 12;
        let s = (long as u16) << 11;
        let d = disp as u8 as u16;
        Self(a | r | s | d)
    }

    /// Returns the displacement associated with the brief extension word.
    pub const fn disp(self) -> i8 {
        self.0 as i8
    }

    #[inline(always)]
    const fn is_address_reg(self) -> bool {
        bit(self.0, 15)
    }

    #[inline(always)]
    const fn reg(self) -> u8 {
        bits(self.0, 12, 14) as u8
    }

    #[inline(always)]
    const fn is_long(self) -> bool {
        bit(self.0, 11)
    }
}

impl std::fmt::Display for BriefExtensionWord {
    /// Disassembles the index register field of a brief extension word.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x = if self.is_address_reg() { "A" } else { "D" };
        let reg = self.reg();
        let size = if self.is_long() { "L" } else { "W" };
        write!(f, "{x}{reg}.{size}")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct EffectiveAddress {
    /// The addressing mode.
    pub mode: AddressingMode,
    /// Where this effective address points to. `None` if the address has not been calculated yet.
    pub address: Option<u32>,
    /// The size of the data.
    pub size: Option<Size>,
}

impl EffectiveAddress {
    pub fn new(am: AddressingMode, size: Option<Size>) -> Self {
        Self {
            mode: am,
            address: None,
            size,
        }
    }
}

impl<CPU: CpuDetails> M68000<CPU> {
    /// Calculates the value of the given effective address.
    ///
    /// If the address has already been calculated (`ea.address` is Some), it is returned and no computation is performed.
    /// Otherwise the address is computed and assigned to `ea.address` and returned, or panic if the addressing mode is not in memory.
    pub(super) fn get_effective_address(&mut self, ea: &mut EffectiveAddress, exec_time: &mut usize) -> u32 {
        if ea.address.is_none() {
            ea.address = match ea.mode {
                AddressingMode::Ari(reg) => {
                    *exec_time += CPU::EA_ARI;
                    Some(self.regs.a(reg))
                },
                AddressingMode::Ariwpo(reg) => {
                    *exec_time += CPU::EA_ARIWPO;
                    Some(self.ariwpo(reg, ea.size.expect("ariwpo must have a size")))
                },
                AddressingMode::Ariwpr(reg) => {
                    *exec_time += CPU::EA_ARIWPR;
                    Some(self.ariwpr(reg, ea.size.expect("ariwpr must have a size")))
                },
                AddressingMode::Ariwd(reg, disp)  => {
                    *exec_time += CPU::EA_ARIWD;
                    Some(self.regs.a(reg).wrapping_add(disp as u32))
                },
                AddressingMode::Ariwi8(reg, bew) => {
                    *exec_time += CPU::EA_ARIWI8;
                    Some(self.regs.a(reg).wrapping_add(bew.disp() as u32).wrapping_add(self.get_index_register(bew)))
                },
                AddressingMode::AbsShort(addr) => {
                    *exec_time += CPU::EA_ABSSHORT;
                    Some(addr as i16 as u32)
                },
                AddressingMode::AbsLong(addr) => {
                    *exec_time += CPU::EA_ABSLONG;
                    Some(addr)
                },
                AddressingMode::Pciwd(pc, disp) => {
                    *exec_time += CPU::EA_PCIWD;
                    Some(pc.wrapping_add(disp as u32))
                },
                AddressingMode::Pciwi8(pc, bew) => {
                    *exec_time += CPU::EA_PCIWI8;
                    Some(pc.wrapping_add(bew.disp() as u32).wrapping_add(self.get_index_register(bew)))
                },
                _ => None,
            };
        }

        ea.address.expect("[get_effective_address] Trying to read effective address of a value not in memory.")
    }

    const fn get_index_register(&self, bew: BriefExtensionWord) -> u32 {
        let reg = bew.reg();
        let long = bew.is_long();

        if bew.is_address_reg() {
            if long {
                self.regs.a(reg)
            } else { // Word
                self.regs.a(reg) as i16 as u32
            }
        } else { // Data register
            if long {
                self.regs.d[reg as usize].0
            } else { // Word
                self.regs.d[reg as usize].0 as i16 as u32
            }
        }
    }

    /// Address Register Indirect With POstincrement
    pub(super) fn ariwpo(&mut self, reg: u8, size: Size) -> u32 {
        let areg = self.regs.a_mut(reg);
        let addr = areg.0;
        *areg += if reg == 7 { size.as_word_long() } else { size } as u32;
        addr
    }

    /// Address Register Indirect With PRedecrement
    pub(super) fn ariwpr(&mut self, reg: u8, size: Size) -> u32 {
        let areg = self.regs.a_mut(reg);
        *areg -= if reg == 7 { size.as_word_long() } else { size } as u32;
        areg.0
    }
}
