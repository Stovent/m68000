//! Motorola 68000 assembler, disassembler and interpreter.
//!
//! # How to use
//!
//! When creating a new core, the [CpuType] is used to configure the instruction's execution times and exception handling.
//!
//! Since the memory map is application-dependant, you have to implement the [MemoryAccess] trait on your memory management
//! structure, and pass it to the core when executing instructions.
//!
//! ## Basic usage:
//!
//! ```
//! const MEM_SIZE: u32 = 65536;
//! struct Memory([u8; MEM_SIZE as usize]); // Define your memory management system.
//!
//! impl MemoryAccess for Memory { // Implement the MemoryAccess trait.
//!     fn get_byte(&mut self, addr: u32) -> GetResult<u8> {
//!         if addr < MEM_SIZE {
//!             Ok(self.0[addr as usize])
//!         } else {
//!             Err(Vector::AccessError as u8)
//!         }
//!     }
//!
//!     // And so on...
//! }
//!
//! fn main() {
//!     let mut memory = Memory([0; MEM_SIZE as usize]);
//!     // Load the program in memory here.
//!     let mut cpu = M68000::new(CpuType::M68000);
//!
//!     // Execute instructions
//!     cpu.interpreter(&mut memory);
//! }
//! ```
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
    /// Data registers.
    pub d: [u32; 8],
    a_: [u32; 7],
    /// User Stack Pointer.
    pub usp: u32,
    /// System Stack Pointer.
    pub ssp: u32,
    /// Status Register.
    pub sr: StatusRegister,
    /// Program Counter.
    pub pc: u32,

    current_opcode: u16,
    stop: bool,
    exceptions: VecDeque<u8>,
    /// Stores the number of extra cycles executed during the last call to execute_cycles.
    extra_cycles: usize,
    /// True to disassemble instructions and call [MemoryAccess::disassembler].
    pub disassemble: bool,
    cpu_type: CpuType,
}

impl M68000 {
    /// Creates a new M68000 core.
    ///
    /// The created core has a [exception::Vector::ResetSspPc] pushed, so that the first call to an interpreter method
    /// will first fetch the reset vectors, then will execute the first instruction.
    pub fn new(cpu_type: CpuType) -> Self {
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
            cpu_type,
        };

        cpu.exception(exception::Vector::ResetSspPc as u8);

        cpu
    }

    /// [Self::new] but without the initial reset vectors, so you can initialize the core as you want.
    pub fn new_no_reset(cpu_type: CpuType) -> Self {
        Self {
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
            cpu_type,
        }
    }

    /// Sets the lower 8-bits of the given data register to the given value.
    /// The higher 24-bits remains untouched.
    pub fn d_byte(&mut self, reg: u8, value: u8) {
        self.d[reg as usize] &= 0xFFFF_FF00;
        self.d[reg as usize] |= value as u32;
    }

    /// Sets the lower 16-bits of the given data register to the given value.
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

    /// Returns the stack pointer, SSP if in supervisor mode, USP if in user mode.
    pub const fn sp(&self) -> u32 {
        if self.sr.s {
            self.ssp
        } else {
            self.usp
        }
    }

    /// Returns a mutable reference to the stack pointer, SSP if in supervisor mode, USP if in user mode.
    pub fn sp_mut(&mut self) -> &mut u32 {
        if self.sr.s {
            &mut self.ssp
        } else {
            &mut self.usp
        }
    }
}

/// The CPU type, which defines the behaviour of the CPU.
///
/// The behaviors include exception handling and instruction execution time.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CpuType {
    M68000,
    M68070,
}

impl CpuType {
    /// Returns true if self is a M68000 kind.
    pub fn is_68000(self) -> bool {
        self == Self::M68000
    }

    /// Returns true if self is a M68070 kind.
    pub fn is_68070(self) -> bool {
        self == Self::M68070
    }
}
