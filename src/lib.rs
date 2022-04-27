//! Motorola 68000 emulator.
//!
//! Applications creates their memory management system, implementing the
//! [MemoryAccess](memory_access::MemoryAccess) trait, and passes it to the core.
//!
//! # How to use
//!
//! m68000 is intended to be used as a general Motorola 68000 interpreter, surrounded by a more complex environment.
//!
//! # Example
//!
//! The standard use case is when having to emulate a M68k-based microcontroller (such as the SCC68070), that includes a 68000 CPU
//! along with other peripherals.
//! In this context, see the [main.rs](https://github.com/Stovent/m68000/blob/master/src/main.rs) file in the repo.
//!
//! # TODO:
//! - Calculation times.
//! - How to restore MC68000 Bus and Address errors?
//! - Better management of exceptions and STOP mode.
//! - Exception priority.

pub mod addressing_modes;
pub mod assembler;
pub mod decoder;
pub mod disassembler;
pub mod exception;
pub mod instruction;
mod interpreter;
pub mod isa;
pub mod memory_access;
pub mod status_register;
pub mod utils;

use memory_access::MemoryAccess;
use status_register::StatusRegister;

use std::collections::VecDeque;

/// A M68000 core.
#[derive(Clone, Debug)]
pub struct M68000 {
    /// The data registers.
    pub d: [u32; 8],
    a_: [u32; 7],
    usp: u32,
    ssp: u32,
    /// The status register.
    pub sr: StatusRegister,
    pc: u32,

    current_opcode: u16,
    stop: bool,
    exceptions: VecDeque<u8>,
    /// Stores the number of extra cycles executed during the last call to execute_cycles.
    extra_cycles: usize,
    /// True to disassemble instructions and call [MemoryAccess::disassembler].
    pub disassemble: bool,
    stack_format: StackFormat,
}

impl M68000 {
    /// Creates a new M68000 core.
    pub fn new(stack_format: StackFormat) -> Self {
        let mut cpu = Self {
            d: [0; 8],
            a_: [0; 7],
            usp: 0,
            ssp: 0,
            sr: StatusRegister::default(),
            pc: 0,

            current_opcode: 0xFFFF,
            stop: false,
            exceptions: VecDeque::new(),
            extra_cycles: 0,
            disassemble: false,
            stack_format,
        };

        cpu.exception(exception::Vector::ResetSspPc as u8);

        cpu
    }

    /// Sets the lower 8-bits of the given register to the given value.
    /// The higher 24-bits remains untouched.
    pub fn d_byte(&mut self, reg: u8, value: u8) {
        self.d[reg as usize] &= 0xFFFF_FF00;
        self.d[reg as usize] |= value as u32;
    }

    /// Sets the lower 16-bits of the given register to the given value.
    /// The higher 16-bits remains untouched.
    pub fn d_word(&mut self, reg: u8, value: u16) {
        self.d[reg as usize] &= 0xFFFF_0000;
        self.d[reg as usize] |= value as u32;
    }

    /// Returns an address register.
    pub const fn a(&self, reg: u8) -> u32 {
        if reg < 7 {
            self.a_[reg as usize]
        } else {
            self.sp()
        }
    }

    /// Returns a mutable reference to an address register.
    pub fn a_mut(&mut self, reg: u8) -> &mut u32 {
        if reg < 7 {
            &mut self.a_[reg as usize]
        } else {
            self.sp_mut()
        }
    }

    const fn sp(&self) -> u32 {
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

/// The stack frame format to use, which will decide the behaviour of exception handling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StackFormat {
    Stack68000,
    Stack68010,
    Stack68070,
}

impl StackFormat {
    /// Returns true if self is a M68000 stack.
    pub fn is_68000(self) -> bool {
        self == StackFormat::Stack68000
    }

    /// Returns true if self is a M68010 stack.
    pub fn is_68010(self) -> bool {
        self == StackFormat::Stack68010
    }

    /// Returns true if self is a M68070 stack.
    pub fn is_68070(self) -> bool {
        self == StackFormat::Stack68070
    }
}
