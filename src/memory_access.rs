//! Memory access-related traits and structs.

use crate::M68000;
use crate::addressing_modes::EffectiveAddress;
use crate::instruction::Size;
use crate::utils::SliceAs;

/// Returns the value asked on success, an exception vector on error. Alias for `Result<T, u8>`.
#[must_use]
pub type GetResult<T> = Result<T, u8>;
/// Returns the value asked on success, an exception vector on error. Alias for `Result<(), u8>`.
#[must_use]
pub type SetResult = Result<(), u8>;

/// The trait to be implemented by the memory system that will be used by the core.
///
/// The return values are used to indicate whether an [Access Error (Bus Error)](crate::exception::Vector::AccessError)
/// or an [Address Error](crate::exception::Vector::AddressError) occured.
///
/// It is the implementor's responsibility to send those errors.
/// Access errors are sent when the accessed address is not in the memory range of the system.
/// Address errors are sent when the address of word and long data are not aligned on a word (2-bytes) boundary.
/// Byte data never generates address errors.
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
    fn reset(&mut self);

    /// Override this function to modify the exception processing behaviour.
    ///
    /// This function is called by the interpreter when an exception has to be processed.
    /// It has to return `true` if the CPU has to effectively create an exception stack frame.
    /// If it returns `false`, then the CPU won't create a stack frame and the exception will not be further processed.
    /// The default implementation simply returns `true` and does nothing else.
    ///
    /// The point of this is to allow this library to be used in HLE environments, where traps are caughts by the application
    /// and appears transparent to the process. In this case `false` has to be returned.
    fn exception(&mut self, _vector: u8) -> bool { true }
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
    pub(super) fn get_byte(&mut self, memory: &mut impl MemoryAccess, ea: &mut EffectiveAddress) -> GetResult<u8> {
        if ea.mode.drd() {
            Ok(self.d[ea.reg as usize] as u8)
        } else if ea.mode.mode7() && ea.reg == 4 {
            Ok(ea.ext.u16_be() as u8)
        } else {
            let addr = self.get_effective_address(ea).unwrap();
            memory.get_byte(addr)
        }
    }

    #[must_use]
    pub(super) fn get_word(&mut self, memory: &mut impl MemoryAccess, ea: &mut EffectiveAddress) -> GetResult<u16> {
        if ea.mode.drd() {
            Ok(self.d[ea.reg as usize] as u16)
        } else if ea.mode.ard() {
            Ok(self.a(ea.reg) as u16)
        } else if ea.mode.mode7() && ea.reg == 4 {
            Ok(ea.ext.u16_be())
        } else {
            let addr = self.get_effective_address(ea).unwrap();
            memory.get_word(addr)
        }
    }

    #[must_use]
    pub(super) fn get_long(&mut self, memory: &mut impl MemoryAccess, ea: &mut EffectiveAddress) -> GetResult<u32> {
        if ea.mode.drd() {
            Ok(self.d[ea.reg as usize])
        } else if ea.mode.ard() {
            Ok(self.a(ea.reg))
        } else if ea.mode.mode7() && ea.reg == 4 {
            Ok(ea.ext.u32_be())
        } else {
            let addr = self.get_effective_address(ea).unwrap();
            memory.get_long(addr)
        }
    }

    #[must_use]
    pub(super) fn set_byte(&mut self, memory: &mut impl MemoryAccess, ea: &mut EffectiveAddress, value: u8) -> SetResult {
        if ea.mode.drd() {
            self.d_byte(ea.reg, value);
            Ok(())
        } else {
            let addr = self.get_effective_address(ea).unwrap();
            memory.set_byte(addr, value)
        }
    }

    #[must_use]
    pub(super) fn set_word(&mut self, memory: &mut impl MemoryAccess, ea: &mut EffectiveAddress, value: u16) -> SetResult {
        if ea.mode.drd() {
            self.d_word(ea.reg, value);
            Ok(())
        } else if ea.mode.ard() {
            *self.a_mut(ea.reg) = value as i16 as u32;
            Ok(())
        } else {
            let addr = self.get_effective_address(ea).unwrap();
            memory.set_word(addr, value)
        }
    }

    #[must_use]
    pub(super) fn set_long(&mut self, memory: &mut impl MemoryAccess, ea: &mut EffectiveAddress, value: u32) -> SetResult {
        if ea.mode.drd() {
            self.d[ea.reg as usize] = value;
            Ok(())
        } else if ea.mode.ard() {
            *self.a_mut(ea.reg) = value;
            Ok(())
        } else {
            let addr = self.get_effective_address(ea).unwrap();
            memory.set_long(addr, value)
        }
    }

    /// Returns the word at `self.pc` then advances `self.pc` by 2.
    ///
    /// Please note that this function advances the program counter so be careful when using it.
    /// This function is public because it can be useful in some contexts such as OS9 environments
    /// where the trap ID is the immediate next word after the TRAP instruction.
    #[must_use]
    pub fn get_next_word(&mut self, memory: &mut impl MemoryAccess) -> GetResult<u16> {
        let data = memory.get_word(self.pc);
        self.pc += 2;
        data
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
