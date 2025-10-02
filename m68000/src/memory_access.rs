// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Memory access-related traits and structs.

use crate::{CpuDetails, M68000};
use crate::addressing_modes::{EffectiveAddress, AddressingMode};
use crate::exception::{ACCESS_ERROR, ADDRESS_ERROR};
use crate::instruction::Size;
use crate::utils::IsEven;

/// Return type of M68000's read memory methods. `Err(Vector::AddressError or AccessError as u8)` if an address or
/// access (bus) error occured. Alias for `Result<T, u8>`.
type GetResult<T> = Result<T, u8>;
/// Return type of M68000's write memory methods. `Err(Vector::AddressError or AccessError as u8)` if an address or
/// access (bus) error occured. Alias for `Result<(), u8>`.
type SetResult = Result<(), u8>;

/// The trait to be implemented by the memory system that will be used by the core.
///
/// If the read is successful, return a `Some()` with the requested value.
/// If the address of the value asked is not in the memory map of the system, return `None`.
/// This will trigger an Access (Bus) Error, which will interrupt the current instruction processing.
///
/// For word and long accesses, the address is guaranted to be even (16-bits word aligned),
/// as odd addresses are detected by the library and automatically trigger an Address Error.
///
/// The trait is implemented for `[u8]`, `&[u8]`, `[u16]` and `&[u16]`, interpreted as big-endian.
/// A call of a `set_XXXX` method on a non-mutable slice panics.
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
        Some((self.get_word(addr)? as u32) << 16 | self.get_word(addr.wrapping_add(2))? as u32)
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
        self.set_word(addr.wrapping_add(2), value as u16)
    }

    /// Not meant to be overridden.
    /// Returns a [MemoryIter] starting at the given address that will be used to decode instructions.
    #[must_use]
    fn iter_u16(&mut self, addr: u32) -> MemoryIter<'_, Self> {
        MemoryIter { memory: self, next_addr: addr }
    }

    /// Called when the CPU executes a RESET instruction.
    fn reset_instruction(&mut self);
}

/// Iterator over 16-bits values in memory.
pub struct MemoryIter<'a, M: MemoryAccess + ?Sized> {
    /// The memory system that will be used to get the values.
    pub memory: &'a mut M,
    /// The address of the next value to be returned.
    pub next_addr: u32,
}

impl<M: MemoryAccess + ?Sized> Iterator for MemoryIter<'_, M> {
    type Item = GetResult<u16>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_addr.is_even() {
            let data = self.memory.get_word(self.next_addr);
            self.next_addr = self.next_addr.wrapping_add(2);
            Some(data.ok_or(ACCESS_ERROR))
        } else {
            Some(Err(ADDRESS_ERROR))
        }
    }
}

impl<CPU: CpuDetails> M68000<CPU> {
    pub(super) fn get_byte<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, ea: &mut EffectiveAddress, exec_time: &mut usize) -> GetResult<u8> {
        match ea.mode {
            AddressingMode::Drd(reg) => Ok(self.regs.d[reg as usize].0 as u8),
            AddressingMode::Immediate(imm) => {
                *exec_time += CPU::EA_IMMEDIATE;
                Ok(imm as u8)
            },
            _ => memory.get_byte(self.get_effective_address(ea, exec_time)).ok_or(ACCESS_ERROR),
        }
    }

    pub(super) fn get_word<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, ea: &mut EffectiveAddress, exec_time: &mut usize) -> GetResult<u16> {
        match ea.mode {
            AddressingMode::Drd(reg) => Ok(self.regs.d[reg as usize].0 as u16),
            AddressingMode::Ard(reg) => Ok(self.regs.a(reg) as u16),
            AddressingMode::Immediate(imm) => {
                *exec_time += CPU::EA_IMMEDIATE;
                Ok(imm as u16)
            },
            _ => {
                let addr = self.get_effective_address(ea, exec_time).even()?;
                memory.get_word(addr).ok_or(ACCESS_ERROR)
            },
        }
    }

    pub(super) fn get_long<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, ea: &mut EffectiveAddress, exec_time: &mut usize) -> GetResult<u32> {
        match ea.mode {
            AddressingMode::Drd(reg) => Ok(self.regs.d[reg as usize].0),
            AddressingMode::Ard(reg) => Ok(self.regs.a(reg)),
            AddressingMode::Immediate(imm) => {
                *exec_time += CPU::EA_IMMEDIATE + 4;
                Ok(imm)
            },
            _ => {
                let addr = self.get_effective_address(ea, exec_time).even()?;
                let r = memory.get_long(addr).ok_or(ACCESS_ERROR);
                *exec_time += 4;
                r
            },
        }
    }

    pub(super) fn set_byte<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, ea: &mut EffectiveAddress, exec_time: &mut usize, value: u8) -> SetResult {
        match ea.mode {
            AddressingMode::Drd(reg) => { self.regs.d_byte(reg, value); Ok(()) },
            _ => memory.set_byte(self.get_effective_address(ea, exec_time), value).ok_or(ACCESS_ERROR),
        }
    }

    pub(super) fn set_word<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, ea: &mut EffectiveAddress, exec_time: &mut usize, value: u16) -> SetResult {
        match ea.mode {
            AddressingMode::Drd(reg) => { self.regs.d_word(reg, value); Ok(()) },
            AddressingMode::Ard(reg) => { self.regs.a_mut(reg).0 = value as i16 as u32; Ok(()) },
            _ => {
                let addr = self.get_effective_address(ea, exec_time).even()?;
                memory.set_word(addr, value).ok_or(ACCESS_ERROR)
            },
        }
    }

    pub(super) fn set_long<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, ea: &mut EffectiveAddress, exec_time: &mut usize, value: u32) -> SetResult {
        match ea.mode {
            AddressingMode::Drd(reg) => { self.regs.d[reg as usize].0 = value; Ok(()) },
            AddressingMode::Ard(reg) => { self.regs.a_mut(reg).0 = value; Ok(()) },
            _ => {
                let addr = self.get_effective_address(ea, exec_time).even()?;
                let r = memory.set_long(addr, value).ok_or(ACCESS_ERROR);
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
    pub fn get_next_word<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> GetResult<u16> {
        let data = memory.get_word(self.regs.pc.even()?.0).ok_or(ACCESS_ERROR);
        self.regs.pc += 2;
        data
    }

    /// Returns the long at `self.regs.pc` then advances `self.regs.pc` by 4.
    ///
    /// Please note that this function advances the program counter so be careful when using it.
    pub fn get_next_long<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> GetResult<u32> {
        let data = memory.get_long(self.regs.pc.even()?.0).ok_or(ACCESS_ERROR);
        self.regs.pc += 4;
        data
    }

    /// Returns the word at `self.regs.pc`.
    ///
    /// This function is public because it can be useful in some contexts such as OS-9 environments
    /// where the trap ID is the immediate next word after the TRAP instruction.
    pub fn peek_next_word<M: MemoryAccess + ?Sized>(&self, memory: &mut M) -> GetResult<u16> {
        memory.get_word(self.regs.pc.even()?.0).ok_or(ACCESS_ERROR)
    }

    /// Pops the 16-bits value from the stack.
    pub(super) fn pop_word<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> GetResult<u16> {
        let addr = self.ariwpo(7, Size::Word);
        memory.get_word(addr.even()?).ok_or(ACCESS_ERROR)
    }

    /// Pops the 32-bits value from the stack.
    pub(super) fn pop_long<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> GetResult<u32> {
        let addr = self.ariwpo(7, Size::Long);
        memory.get_long(addr.even()?).ok_or(ACCESS_ERROR)
    }

    /// Pushes the given 16-bits value on the stack.
    pub(super) fn push_word<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, value: u16) -> SetResult {
        let addr = self.ariwpr(7, Size::Word);
        memory.set_word(addr.even()?, value).ok_or(ACCESS_ERROR)
    }

    /// Pushes the given 32-bits value on the stack.
    pub(super) fn push_long<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, value: u32) -> SetResult {
        let addr = self.ariwpr(7, Size::Long);
        memory.set_long(addr.even()?, value).ok_or(ACCESS_ERROR)
    }

    /// Creates a new memory iterator starting at the current Program Counter.
    pub(super) fn iter_from_pc<'a, M: MemoryAccess + ?Sized>(&self, memory: &'a mut M) -> MemoryIter<'a, M> {
        memory.iter_u16(self.regs.pc.0)
    }
}

impl MemoryAccess for [u8] {
    fn get_byte(&mut self, addr: u32) -> Option<u8> {
        let addr = addr as usize;
        if addr < self.len() {
            Some(self[addr])
        } else {
            None
        }
    }

    fn get_word(&mut self, addr: u32) -> Option<u16> {
        let addr = addr as usize;
        if addr < self.len() {
            Some((self[addr] as u16) << 8 | self[addr.wrapping_add(1)] as u16)
        } else {
            None
        }
    }

    fn set_byte(&mut self, addr: u32, data: u8) -> Option<()> {
        let addr = addr as usize;
        if addr < self.len() {
            self[addr] = data;
            Some(())
        } else {
            None
        }
    }

    fn set_word(&mut self, addr: u32, data: u16) -> Option<()> {
        let addr = addr as usize;
        if addr < self.len() {
            self[addr] = (data >> 8) as u8;
            self[addr.wrapping_add(1)] = data as u8;
            Some(())
        } else {
            None
        }
    }

    fn reset_instruction(&mut self) {}
}

impl MemoryAccess for &[u8] {
    fn get_byte(&mut self, addr: u32) -> Option<u8> {
        let addr = addr as usize;
        if addr < self.len() {
            Some(self[addr])
        } else {
            None
        }
    }

    fn get_word(&mut self, addr: u32) -> Option<u16> {
        let addr = addr as usize;
        if addr < self.len() {
            Some((self[addr] as u16) << 8 | self[addr.wrapping_add(1)] as u16)
        } else {
            None
        }
    }

    fn set_byte(&mut self, _: u32, _: u8) -> Option<()> {
        panic!("Can't write in non-mutable buffer");
    }

    fn set_word(&mut self, _: u32, _: u16) -> Option<()> {
        panic!("Can't write in non-mutable buffer");
    }

    fn reset_instruction(&mut self) {}
}

impl MemoryAccess for [u16] {
    fn get_byte(&mut self, addr: u32) -> Option<u8> {
        let a = addr as usize >> 1;
        if a < self.len() {
            if addr.is_even() {
                Some((self[a] >> 8) as u8)
            } else {
                Some(self[a] as u8)
            }
        } else {
            None
        }
    }

    fn get_word(&mut self, addr: u32) -> Option<u16> {
        let addr = addr as usize >> 1;
        if addr < self.len() {
            Some(self[addr])
        } else {
            None
        }
    }

    fn set_byte(&mut self, addr: u32, data: u8) -> Option<()> {
        let a = addr as usize >> 1;
        if a < self.len() {
            if addr.is_even() {
                self[a] &= 0x00FF;
                self[a] |= (data as u16) << 8;
            } else {
                self[a] &= 0xFF00;
                self[a] |= data as u16;
            }
            Some(())
        } else {
            None
        }
    }

    fn set_word(&mut self, addr: u32, data: u16) -> Option<()> {
        let addr = addr as usize >> 1;
        if addr < self.len() {
            self[addr] = data;
            Some(())
        } else {
            None
        }
    }

    fn reset_instruction(&mut self) {}
}

impl MemoryAccess for &[u16] {
    fn get_byte(&mut self, addr: u32) -> Option<u8> {
        let a = addr as usize >> 1;
        if a < self.len() {
            if addr.is_even() {
                Some((self[a] >> 8) as u8)
            } else {
                Some(self[a] as u8)
            }
        } else {
            None
        }
    }

    fn get_word(&mut self, addr: u32) -> Option<u16> {
        let addr = addr as usize >> 1;
        if addr < self.len() {
            Some(self[addr])
        } else {
            None
        }
    }

    fn set_byte(&mut self, _: u32, _: u8) -> Option<()> {
        panic!("Can't write in non-mutable buffer");
    }

    fn set_word(&mut self, _: u32, _: u16) -> Option<()> {
        panic!("Can't write in non-mutable buffer");
    }

    fn reset_instruction(&mut self) {}
}
