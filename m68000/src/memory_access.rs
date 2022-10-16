// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Memory access-related traits and structs.

use crate::{CpuDetails, M68000};
use crate::addressing_modes::{EffectiveAddress, AddressingMode};
use crate::exception::{ACCESS_ERROR, ADDRESS_ERROR};
use crate::instruction::Size;
use crate::utils::IsEven;

/// Return type of M68000's read memory methods. `Err(Vector::AddressError or AccessError as u8)` if an address or access (bus) error occured. Alias for `Result<T, u8>`.
type GetResult<T> = Result<T, u8>;
/// Return type of M68000's write memory methods. `Err(Vector::AddressError or AccessError as u8)` if an address or access (bus) error occured. Alias for `Result<(), u8>`.
type SetResult = Result<(), u8>;

/// The trait to be implemented by the memory system that will be used by the core.
///
/// If the read is successful, return a `Some()` with the requested value.
/// If the address of the value asked is not in the memory map of the system, return `None`.
/// This will trigger an Access (Bus) Error, which will interrupt the current instruction processing.
///
/// For word and long accesses, the address is guaranted to be even (16-bits word aligned),
/// as odd addresses are detected by the library and automatically trigger an Address Error.
pub trait MemoryAccess {
    /// Returns a 8-bits integer from the given address.
    #[must_use]
    fn get_byte(&mut self, addr: u32) -> Option<u8>;

    /// Returns a big-endian 16-bits integer from the given address.
    #[must_use]
    fn get_word(&mut self, addr: u32) -> Option<u16>;

    /// Returns a big-endian 32-bits integer from the given address.
    ///
    /// The default implementation is doing 2 calls to [Self::get_word] with the high and low words.
    #[must_use]
    fn get_long(&mut self, addr: u32) -> Option<u32> {
        Some((self.get_word(addr)? as u32) << 16 | self.get_word(addr + 2)? as u32)
    }

    /// Stores the given 8-bits value at the given address.
    #[must_use]
    fn set_byte(&mut self, addr: u32, value: u8) -> Option<()>;

    /// Stores the given 16-bits value at the given address, in big-endian format.
    #[must_use]
    fn set_word(&mut self, addr: u32, value: u16) -> Option<()>;

    /// Stores the given 32-bits value at the given address, in big-endian format.
    ///
    /// The default implementation is doing 2 calls to [Self::set_word] with the high and low words.
    #[must_use]
    fn set_long(&mut self, addr: u32, value: u32) -> Option<()> {
        self.set_word(addr, (value >> 16) as u16)?;
        self.set_word(addr + 2, value as u16)
    }

    /// Not meant to be overridden. Returns a [MemoryIter] starting at the given address that will be used to decode instructions.
    #[must_use]
    fn iter_u16(&mut self, addr: u32) -> MemoryIter where Self: Sized {
        MemoryIter { memory: self, next_addr: addr }
    }

    /// Called when the CPU executes a RESET instruction.
    fn reset_instruction(&mut self);
}

/// Iterator over 16-bits values in memory.
pub struct MemoryIter<'a> {
    /// The memory system that will be used to get the values.
    pub memory: &'a mut dyn MemoryAccess,
    /// The address of the next value to be returned.
    pub next_addr: u32,
}

impl Iterator for MemoryIter<'_> {
    type Item = GetResult<u16>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_addr.is_even() {
            let data = self.memory.get_word(self.next_addr);
            self.next_addr += 2;
            Some(data.ok_or(ACCESS_ERROR))
        } else {
            Some(Err(ADDRESS_ERROR))
        }
    }
}

impl<CPU: CpuDetails> M68000<CPU> {
    #[must_use]
    pub(super) fn get_byte(&mut self, memory: &mut impl MemoryAccess, ea: &mut EffectiveAddress, exec_time: &mut usize) -> GetResult<u8> {
        match ea.mode {
            AddressingMode::Drd(reg) => Ok(self.regs.d[reg as usize] as u8),
            AddressingMode::Immediate(imm) => {
                *exec_time += CPU::EA_IMMEDIATE;
                Ok(imm as u8)
            },
            _ => memory.get_byte(self.get_effective_address(ea, exec_time)).ok_or(ACCESS_ERROR),
        }
    }

    #[must_use]
    pub(super) fn get_word(&mut self, memory: &mut impl MemoryAccess, ea: &mut EffectiveAddress, exec_time: &mut usize) -> GetResult<u16> {
        match ea.mode {
            AddressingMode::Drd(reg) => Ok(self.regs.d[reg as usize] as u16),
            AddressingMode::Ard(reg) => Ok(self.regs.a(reg) as u16),
            AddressingMode::Immediate(imm) => {
                *exec_time += CPU::EA_IMMEDIATE;
                Ok(imm as u16)
            },
            _ => {
                let addr = self.get_effective_address(ea, exec_time);
                memory.get_word(addr.even()?).ok_or(ACCESS_ERROR)
            },
        }
    }

    #[must_use]
    pub(super) fn get_long(&mut self, memory: &mut impl MemoryAccess, ea: &mut EffectiveAddress, exec_time: &mut usize) -> GetResult<u32> {
        match ea.mode {
            AddressingMode::Drd(reg) => Ok(self.regs.d[reg as usize]),
            AddressingMode::Ard(reg) => Ok(self.regs.a(reg)),
            AddressingMode::Immediate(imm) => {
                *exec_time += CPU::EA_IMMEDIATE + 4;
                Ok(imm)
            },
            _ => {
                let addr = self.get_effective_address(ea, exec_time);
                let r = memory.get_long(addr.even()?).ok_or(ACCESS_ERROR);
                *exec_time += 4;
                r
            },
        }
    }

    #[must_use]
    pub(super) fn set_byte(&mut self, memory: &mut impl MemoryAccess, ea: &mut EffectiveAddress, exec_time: &mut usize, value: u8) -> SetResult {
        match ea.mode {
            AddressingMode::Drd(reg) => Ok(self.regs.d_byte(reg, value)),
            _ => memory.set_byte(self.get_effective_address(ea, exec_time), value).ok_or(ACCESS_ERROR),
        }
    }

    #[must_use]
    pub(super) fn set_word(&mut self, memory: &mut impl MemoryAccess, ea: &mut EffectiveAddress, exec_time: &mut usize, value: u16) -> SetResult {
        match ea.mode {
            AddressingMode::Drd(reg) => Ok(self.regs.d_word(reg, value)),
            AddressingMode::Ard(reg) => Ok(*self.regs.a_mut(reg) = value as i16 as u32),
            _ => {
                let addr = self.get_effective_address(ea, exec_time);
                memory.set_word(addr.even()?, value).ok_or(ACCESS_ERROR)
            },
        }
    }

    #[must_use]
    pub(super) fn set_long(&mut self, memory: &mut impl MemoryAccess, ea: &mut EffectiveAddress, exec_time: &mut usize, value: u32) -> SetResult {
        match ea.mode {
            AddressingMode::Drd(reg) => Ok(self.regs.d[reg as usize] = value),
            AddressingMode::Ard(reg) => Ok(*self.regs.a_mut(reg) = value),
            _ => {
                let addr = self.get_effective_address(ea, exec_time);
                let r = memory.set_long(addr.even()?, value).ok_or(ACCESS_ERROR);
                *exec_time += 4;
                r
            },
        }
    }

    /// Returns the word at `self.regs.pc` then advances `self.regs.pc` by 2.
    ///
    /// Please note that this function advances the program counter so be careful when using it.
    /// This function is public because it can be useful in some contexts such as OS-9 environments
    /// where the trap ID is the immediate next word after the TRAP instruction.
    #[must_use]
    pub fn get_next_word(&mut self, memory: &mut impl MemoryAccess) -> GetResult<u16> {
        let data = memory.get_word(self.regs.pc.even()?).ok_or(ACCESS_ERROR);
        self.regs.pc += 2;
        data
    }

    /// Returns the long at `self.regs.pc` then advances `self.regs.pc` by 4.
    ///
    /// Please note that this function advances the program counter so be careful when using it.
    #[must_use]
    pub fn get_next_long(&mut self, memory: &mut impl MemoryAccess) -> GetResult<u32> {
        let data = memory.get_long(self.regs.pc.even()?).ok_or(ACCESS_ERROR);
        self.regs.pc += 4;
        data
    }

    /// Returns the word at `self.regs.pc`.
    ///
    /// This function is public because it can be useful in some contexts such as OS-9 environments
    /// where the trap ID is the immediate next word after the TRAP instruction.
    #[must_use]
    pub fn peek_next_word(&self, memory: &mut impl MemoryAccess) -> GetResult<u16> {
        memory.get_word(self.regs.pc.even()?).ok_or(ACCESS_ERROR)
    }

    /// Pops the 16-bits value from the stack.
    #[must_use]
    pub(super) fn pop_word(&mut self, memory: &mut impl MemoryAccess) -> GetResult<u16> {
        let addr = self.ariwpo(7, Size::Word);
        memory.get_word(addr.even()?).ok_or(ACCESS_ERROR)
    }

    /// Pops the 32-bits value from the stack.
    #[must_use]
    pub(super) fn pop_long(&mut self, memory: &mut impl MemoryAccess) -> GetResult<u32> {
        let addr = self.ariwpo(7, Size::Long);
        memory.get_long(addr.even()?).ok_or(ACCESS_ERROR)
    }

    /// Pushes the given 16-bits value on the stack.
    #[must_use]
    pub(super) fn push_word(&mut self, memory: &mut impl MemoryAccess, value: u16) -> SetResult {
        let addr = self.ariwpr(7, Size::Word);
        memory.set_word(addr.even()?, value).ok_or(ACCESS_ERROR)
    }

    /// Pushes the given 32-bits value on the stack.
    #[must_use]
    pub(super) fn push_long(&mut self, memory: &mut impl MemoryAccess, value: u32) -> SetResult {
        let addr = self.ariwpr(7, Size::Long);
        memory.set_long(addr.even()?, value).ok_or(ACCESS_ERROR)
    }
}
