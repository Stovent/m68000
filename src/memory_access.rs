//! Memory access-related traits and structs.

use crate::M68000;
use crate::addressing_modes::{EffectiveAddress, AddressingMode};
use crate::exception::Vector;
use crate::execution_times as EXEC;
use crate::instruction::Size;
use crate::utils::IsEven;

/// Returns the value asked on success, `Err(Vector::AccessError as u8)` if an access (bus) error occured. Alias for `Result<T, u8>`.
pub type GetResult<T> = Result<T, u8>;
/// Returns `Ok(())` on success, `Err(Vector::AccessError as u8)` if an access (bus) error occured. Alias for `Result<(), u8>`.
pub type SetResult = Result<(), u8>;

/// The trait to be implemented by the memory system that will be used by the core.
///
/// Return `Err(Vector::AccessError as u8)` if an [Access Error (Bus Error)](crate::exception::Vector::AccessError) occured.
/// It is mandatory to return only this value in an Err.
///
/// It is the implementor's responsibility to send this error.
/// Access errors are sent when the accessed address is not in the memory range of the system.
pub trait MemoryAccess {
    /// Returns a 8-bits integer from the given address.
    #[must_use]
    fn get_byte(&mut self, addr: u32) -> GetResult<u8>;
    /// Returns a big-endian 16-bits integer from the given address.
    #[must_use]
    fn get_word(&mut self, addr: u32) -> GetResult<u16>;
    /// Returns a big-endian 32-bits integer from the given address.
    #[must_use]
    fn get_long(&mut self, addr: u32) -> GetResult<u32>;

    /// Stores the given 8-bits value at the given address.
    #[must_use]
    fn set_byte(&mut self, addr: u32, value: u8) -> SetResult;
    /// Stores the given 16-bits value at the given address, in big-endian format.
    #[must_use]
    fn set_word(&mut self, addr: u32, value: u16) -> SetResult;
    /// Stores the given 32-bits value at the given address, in big-endian format.
    #[must_use]
    fn set_long(&mut self, addr: u32, value: u32) -> SetResult;

    /// Not meant to be overridden. Returns a [MemoryIter] starting at the given address that will be used to decode instructions.
    #[must_use]
    fn iter_u16(&mut self, addr: u32) -> MemoryIter where Self: Sized { MemoryIter { memory: self, next_addr: addr } }

    /// Called when the CPU executes a RESET instruction.
    fn reset_instruction(&mut self);

    /// If `M68000::disassemble` is true, called by the interpreter with the address and the disassembly of the next instruction that will be executed.
    fn disassembler(&mut self, _pc: u32, _inst_string: String);
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
        let data = self.memory.get_word(self.next_addr);
        self.next_addr += 2;
        Some(data)
    }
}

impl M68000 {
    #[must_use]
    pub(super) fn get_byte(&mut self, memory: &mut impl MemoryAccess, ea: &mut EffectiveAddress, exec_time: &mut usize) -> GetResult<u8> {
        match ea.mode {
            AddressingMode::Drd(reg) => Ok(self.regs.d[reg as usize] as u8),
            AddressingMode::Immediate(imm) => {
                *exec_time += EXEC::EA_IMMEDIATE;
                Ok(imm as u8)
            },
            _ => memory.get_byte(self.get_effective_address(ea, exec_time)),
        }
    }

    #[must_use]
    pub(super) fn get_word(&mut self, memory: &mut impl MemoryAccess, ea: &mut EffectiveAddress, exec_time: &mut usize) -> GetResult<u16> {
        match ea.mode {
            AddressingMode::Drd(reg) => Ok(self.regs.d[reg as usize] as u16),
            AddressingMode::Ard(reg) => Ok(self.a(reg) as u16),
            AddressingMode::Immediate(imm) => {
                *exec_time += EXEC::EA_IMMEDIATE;
                Ok(imm as u16)
            },
            _ => {
                let addr = self.get_effective_address(ea, exec_time);
                if addr.is_even() {
                    memory.get_word(addr)
                } else {
                    Err(Vector::AddressError as u8)
                }
            },
        }
    }

    #[must_use]
    pub(super) fn get_long(&mut self, memory: &mut impl MemoryAccess, ea: &mut EffectiveAddress, exec_time: &mut usize) -> GetResult<u32> {
        match ea.mode {
            AddressingMode::Drd(reg) => Ok(self.regs.d[reg as usize]),
            AddressingMode::Ard(reg) => Ok(self.a(reg)),
            AddressingMode::Immediate(imm) => {
                *exec_time += EXEC::EA_IMMEDIATE + 4;
                Ok(imm)
            },
            _ => {
                let addr = self.get_effective_address(ea, exec_time);
                if addr.is_even() {
                    let r = memory.get_long(addr);
                    *exec_time += 4;
                    r
                } else {
                    Err(Vector::AddressError as u8)
                }
            },
        }
    }

    #[must_use]
    pub(super) fn set_byte(&mut self, memory: &mut impl MemoryAccess, ea: &mut EffectiveAddress, exec_time: &mut usize, value: u8) -> SetResult {
        match ea.mode {
            AddressingMode::Drd(reg) => Ok(self.d_byte(reg, value)),
            _ => memory.set_byte(self.get_effective_address(ea, exec_time), value),
        }
    }

    #[must_use]
    pub(super) fn set_word(&mut self, memory: &mut impl MemoryAccess, ea: &mut EffectiveAddress, exec_time: &mut usize, value: u16) -> SetResult {
        match ea.mode {
            AddressingMode::Drd(reg) => Ok(self.d_word(reg, value)),
            AddressingMode::Ard(reg) => Ok(*self.a_mut(reg) = value as i16 as u32),
            _ => {
                let addr = self.get_effective_address(ea, exec_time);
                if addr.is_even() {
                    memory.set_word(addr, value)
                } else {
                    Err(Vector::AddressError as u8)
                }
            },
        }
    }

    #[must_use]
    pub(super) fn set_long(&mut self, memory: &mut impl MemoryAccess, ea: &mut EffectiveAddress, exec_time: &mut usize, value: u32) -> SetResult {
        match ea.mode {
            AddressingMode::Drd(reg) => Ok(self.regs.d[reg as usize] = value),
            AddressingMode::Ard(reg) => Ok(*self.a_mut(reg) = value),
            _ => {
                let addr = self.get_effective_address(ea, exec_time);
                if addr.is_even() {
                    let r = memory.set_long(addr, value);
                    *exec_time += 4;
                    r
                } else {
                    Err(Vector::AddressError as u8)
                }
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
        let data = memory.get_word(self.regs.pc);
        self.regs.pc += 2;
        data
    }

    /// Returns the long at `self.regs.pc` then advances `self.regs.pc` by 4.
    ///
    /// Please note that this function advances the program counter so be careful when using it.
    #[must_use]
    pub fn get_next_long(&mut self, memory: &mut impl MemoryAccess) -> GetResult<u32> {
        let data = memory.get_long(self.regs.pc);
        self.regs.pc += 4;
        data
    }

    /// Returns the word at `self.regs.pc`.
    ///
    /// This function is public because it can be useful in some contexts such as OS-9 environments
    /// where the trap ID is the immediate next word after the TRAP instruction.
    #[must_use]
    pub fn peek_next_word(&self, memory: &mut impl MemoryAccess) -> GetResult<u16> {
        memory.get_word(self.regs.pc)
    }

    /// Pops the 16-bits value from the stack.
    #[must_use]
    pub(super) fn pop_word(&mut self, memory: &mut impl MemoryAccess) -> GetResult<u16> {
        let addr = self.ariwpo(7, Size::Word);
        memory.get_word(addr)
    }

    /// Pops the 32-bits value from the stack.
    #[must_use]
    pub(super) fn pop_long(&mut self, memory: &mut impl MemoryAccess) -> GetResult<u32> {
        let addr = self.ariwpo(7, Size::Long);
        memory.get_long(addr)
    }

    /// Pushes the given 16-bits value on the stack.
    #[must_use]
    pub(super) fn push_word(&mut self, memory: &mut impl MemoryAccess, value: u16) -> SetResult {
        let addr = self.ariwpr(7, Size::Word);
        memory.set_word(addr, value)
    }

    /// Pushes the given 32-bits value on the stack.
    #[must_use]
    pub(super) fn push_long(&mut self, memory: &mut impl MemoryAccess, value: u32) -> SetResult {
        let addr = self.ariwpr(7, Size::Long);
        memory.set_long(addr, value)
    }
}
