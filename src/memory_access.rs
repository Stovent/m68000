//! Memory access-related traits and structs.

use crate::M68000;
use crate::addressing_modes::EffectiveAddress;
use crate::instruction::Size;
use crate::utils::SliceAs;

/// The trait to be implemented by the memory system that will be used by the core.
pub trait MemoryAccess {
    /// Returns a 8-bits integer from the given address.
    fn get_byte(&mut self, addr: u32) -> u8;
    /// Returns a big-endian 16-bits integer from the given address.
    fn get_word(&mut self, addr: u32) -> u16;
    /// Returns a big-endian 32-bits integer from the given address.
    fn get_long(&mut self, addr: u32) -> u32;

    /// Stores the given 8-bits value at the given address.
    fn set_byte(&mut self, addr: u32, value: u8);
    /// Stores the given 16-bits value at the given address, in big-endian format.
    fn set_word(&mut self, addr: u32, value: u16);
    /// Stores the given 32-bits value at the given address, in big-endian format.
    fn set_long(&mut self, addr: u32, value: u32);

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
pub trait U16Iter : Iterator<Item = u16> {
    fn next_addr(&self) -> u32;
}

pub(super) struct MemoryIter<'a> {
    /// The memory system that will be used to get the values.
    pub memory: &'a mut dyn MemoryAccess,
    /// The address of the next value to be returned.
    pub next_addr: u32,
}

impl Iterator for MemoryIter<'_> {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        let data = self.memory.get_word(self.next_addr);
        self.next_addr += 2;
        Some(data)
    }
}

impl U16Iter for MemoryIter<'_> {
    fn next_addr(&self) -> u32 {
        self.next_addr
    }
}

impl<M: MemoryAccess> M68000<M> {
    pub(super) fn get_byte(&mut self, ea: &mut EffectiveAddress) -> u8 {
        if ea.mode.drd() {
            self.d[ea.reg as usize] as u8
        } else if ea.mode.mode7() && ea.reg == 4 {
            ea.ext.u16_be() as u8
        } else {
            let addr = self.get_effective_address(ea).unwrap();
            self.memory.get_byte(addr)
        }
    }

    pub(super) fn get_word(&mut self, ea: &mut EffectiveAddress) -> u16 {
        if ea.mode.drd() {
            self.d[ea.reg as usize] as u16
        } else if ea.mode.ard() {
            self.a(ea.reg) as u16
        } else if ea.mode.mode7() && ea.reg == 4 {
            ea.ext.u16_be()
        } else {
            let addr = self.get_effective_address(ea).unwrap();
            self.memory.get_word(addr)
        }
    }

    pub(super) fn get_long(&mut self, ea: &mut EffectiveAddress) -> u32 {
        if ea.mode.drd() {
            self.d[ea.reg as usize]
        } else if ea.mode.ard() {
            self.a(ea.reg)
        } else if ea.mode.mode7() && ea.reg == 4 {
            ea.ext.u32_be()
        } else {
            let addr = self.get_effective_address(ea).unwrap();
            self.memory.get_long(addr)
        }
    }

    pub(super) fn set_byte(&mut self, ea: &mut EffectiveAddress, value: u8) {
        if ea.mode.drd() {
            self.d_byte(ea.reg, value);
        } else {
            let addr = self.get_effective_address(ea).unwrap();
            self.memory.set_byte(addr, value);
        }
    }

    pub(super) fn set_word(&mut self, ea: &mut EffectiveAddress, value: u16) {
        if ea.mode.drd() {
            self.d_word(ea.reg, value);
        } else if ea.mode.ard() {
            *self.a_mut(ea.reg) = value as i16 as u32;
        } else {
            let addr = self.get_effective_address(ea).unwrap();
            self.memory.set_word(addr, value);
        }
    }

    pub(super) fn set_long(&mut self, ea: &mut EffectiveAddress, value: u32) {
        if ea.mode.drd() {
            self.d[ea.reg as usize] = value;
        } else if ea.mode.ard() {
            *self.a_mut(ea.reg) = value;
        } else {
            let addr = self.get_effective_address(ea).unwrap();
            self.memory.set_long(addr, value);
        }
    }

    /// Returns the word at `self.pc` then advances `self.pc` by 2.
    ///
    /// Please note that this function advances the program counter. This function is public because ic can be useful
    /// in OS9 environments where the trap ID is the immediate next word after the TRAP instruction.
    pub fn get_next_word(&mut self) -> u16 {
        let data = self.memory.get_word(self.pc);
        self.pc += 2;
        data
    }

    /// Pops the 16-bits value from the stack.
    pub(super) fn pop_word(&mut self) -> u16 {
        let addr = self.ariwpo(7, Size::Word);
        self.memory.get_word(addr)
    }

    /// Pops the 32-bits value from the stack.
    pub(super) fn pop_long(&mut self) -> u32 {
        let addr = self.ariwpo(7, Size::Long);
        self.memory.get_long(addr)
    }

    /// Pushes the given 16-bits value on the stack.
    pub(super) fn push_word(&mut self, value: u16) {
        let addr = self.ariwpr(7, Size::Word);
        self.memory.set_word(addr, value);
    }

    /// Pushes the given 32-bits value on the stack.
    pub(super) fn push_long(&mut self, value: u32) {
        let addr = self.ariwpr(7, Size::Long);
        self.memory.set_long(addr, value);
    }
}
