//! Motorola 68000 emulator.
//!
//! Applications creates their memory management system, implementing the
//! [MemoryAccess](memory_access::MemoryAccess) trait, and passing it to the core.
//!
//! # Example
//!
//! For a basic example of how to use, see the [main.rs](https://github.com/Stovent/m68000/blob/master/src/main.rs) file in the repo.
//!
//! # TODO:
//! - Exceptions.
//! - Documentation.
//! - Calculation times.
//! - Read-Modify-Write cycles to only have get/set_word.

#![feature(bigint_helper_methods)]

mod addressing_modes;
mod decoder;
mod disassembler;
mod exception;
mod instruction;
mod interpreter;
pub mod isa;
pub mod memory_access;
pub mod status_register;
mod utils;

use memory_access::MemoryAccess;
use status_register::StatusRegister;

const SR_UPPER_MASK: u16 = 0xA700;
const CCR_MASK: u16 = 0x001F;
// const SR_MASK: u16 = SR_UPPER_MASK | CCR_MASK;

/// A M68000 core.
#[derive(Clone, Copy, Debug)]
pub struct M68000<M: MemoryAccess> {
    d: [u32; 8],
    a_: [u32; 7],
    usp: u32,
    ssp: u32,
    sr: StatusRegister,
    pc: u32,

    current_opcode: u16,

    memory: M,
    config: Config,
}

impl<M: MemoryAccess> M68000<M> {
    /// Creates a new M68000 core, with the given memory.
    pub fn new(memory: M, config: Config) -> Self {
        let mut cpu = Self {
            d: [0; 8],
            a_: [0; 7],
            usp: 0,
            ssp: 0,
            sr: StatusRegister::default(),
            pc: 0,

            current_opcode: 0xFFFF,

            memory,
            config,
        };

        cpu.exception(exception::Vector::ResetSsp as u32);
        cpu.exception(exception::Vector::ResetPc as u32);

        cpu
    }

    /// Sets the lower 8-bits to the given register to the given value.
    /// The higher 24-bits remains untouched.
    fn d_byte(&mut self, reg: u8, value: u8) {
        self.d[reg as usize] &= 0xFFFF_FF00;
        self.d[reg as usize] |= value as u32;
    }

    /// Sets the lower 16-bits to the given register to the given value.
    /// The higher 16-bits remains untouched.
    fn d_word(&mut self, reg: u8, value: u16) {
        self.d[reg as usize] &= 0xFFFF_0000;
        self.d[reg as usize] |= value as u32;
    }

    fn a(&self, reg: u8) -> u32 {
        if reg < 7 {
            self.a_[reg as usize]
        } else {
            self.sp()
        }
    }

    fn a_mut(&mut self, reg: u8) -> &mut u32 {
        if reg < 7 {
            &mut self.a_[reg as usize]
        } else {
            self.sp_mut()
        }
    }

    fn sp(&self) -> u32 {
        if self.sr.s {
            self.ssp
        } else {
            self.usp
        }
    }

    fn sp_mut(&mut self) -> &mut u32 {
        if self.sr.s {
            &mut self.ssp
        } else {
            &mut self.usp
        }
    }
}

/// Configuration of the core.
#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub stack: StackFrame,
}

/// Stack frame format based on the processor type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StackFrame {
    // Stack68000,
    Stack68070,
}
