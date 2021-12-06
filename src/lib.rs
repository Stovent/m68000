//! Motorola 68000 emulator.
//!
//! Applications creates their memory management system, implementing the
//! [MemoryAccess](memory_access::MemoryAccess) trait, and passes it to the core.
//!
//! # Example
//!
//! For a basic example of how to use, see the [main.rs](https://github.com/Stovent/m68000/blob/master/src/main.rs) file in the repo.
//!
//! # TODO:
//! - Exceptions.
//! - Calculation times.
//! - Bus and Address errors.

#![feature(bigint_helper_methods)]

pub mod addressing_modes;
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
pub struct M68000<M: MemoryAccess> {
    d: [u32; 8],
    a_: [u32; 7],
    usp: u32,
    ssp: u32,
    sr: StatusRegister,
    pc: u32,

    current_opcode: u16,
    stop: bool,
    exceptions: VecDeque<u8>,
    memory: M,
    /// Stores the number of extra cycles executed during the last call to execute_cycles.
    extra_cycles: usize,
    stack_format: StackFormat,
}

impl<M: MemoryAccess> M68000<M> {
    /// Creates a new M68000 core, with the given memory.
    pub fn new(memory: M, stack_format: StackFormat) -> Self {
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
            memory,
            extra_cycles: 0,
            stack_format,
        };

        cpu.exception(exception::Vector::ResetSspPc as u8);

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

/// The stack frame format to use, which will decide the behaviour of exception handling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StackFormat {
    Stack68000,
    Stack68010,
}
