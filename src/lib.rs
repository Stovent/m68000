//! Motorola 68000 assembler, disassembler and interpreter.
//!
//! This library emulates the common user and supervisor instructions of the M68k ISA.
//! It is configurable at compile-time to behave like the given CPU type (see below), changing the instruction's
//! execution times and exception handling.
//!
//! # Supported CPUs
//!
//! The CPU type is specified at compile-time as a feature. There must be one and only one feature specified.
//!
//! There are no default features. If you don't specify any feature or specify more than one, a compile-time error is raised.
//!
//! * MC68000 (feature "cpu-mc68000")
//! * SCC68070 (feature "cpu-scc68070")
//!
//! # How to use
//!
//! Include this library in your project and configure the CPU type by specifying the correct feature.
//!
//! Since the memory map is application-dependant, you have to implement the [MemoryAccess] trait on your memory management
//! structure, and pass it to the core when executing instructions.
//!
//! ## Basic usage:
//!
//! ```ignore
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
//!     let mut cpu = M68000::new();
//!
//!     // Execute instructions
//!     cpu.interpreter(&mut memory);
//! }
//! ```
//!
//! # TODO:
//! - Let memory access return extra read or write cycles for accuracy.
//! - Allow execution of other exception vectors (like SCC68070 on-chip interrupts).
//! - Verify ABCD, NBCD, SBCD, DIVS and DIVU instructions.

#[cfg(any(
    all(feature = "cpu-mc68000", feature = "cpu-scc68070"),
    not(any(feature = "cpu-mc68000", feature = "cpu-scc68070")),
))]
compile_error!("You must specify one and only one CPU type feature.");

pub mod addressing_modes;
pub mod assembler;
mod cinterface;
pub mod decoder;
pub mod disassembler;
pub mod exception;
pub mod instruction;
mod interpreter;
pub mod isa;
pub mod memory_access;
pub mod status_register;
pub mod utils;

#[cfg(feature = "cpu-mc68000")]
#[path = "cpu/mc68000.rs"]
pub(crate) mod execution_times;
#[cfg(feature = "cpu-scc68070")]
#[path = "cpu/scc68070.rs"]
pub(crate) mod execution_times;

use memory_access::MemoryAccess;
use status_register::StatusRegister;

use std::collections::BinaryHeap;

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

    #[allow(dead_code)]
    current_opcode: u16,
    /// True if the CPU is stopped (after a STOP instruction), false to switch back to normal instruction execution.
    pub stop: bool,
    /// The pending exceptions. Low priority are popped first (MC68000UM 6.2.3 Multiple Exceptions).
    exceptions: BinaryHeap<exception::Exception>,
    /// Number of cycles executed by the called interpreter method.
    cycles: usize,
    /// True to disassemble instructions and call [MemoryAccess::disassembler].
    pub disassemble: bool,
}

impl M68000 {
    /// Creates a new M68000 core.
    ///
    /// The created core has a [exception::Vector::ResetSspPc] pushed, so that the first call to an interpreter method
    /// will first fetch the reset vectors, then will execute the first instruction.
    pub fn new() -> Self {
        let mut cpu = Self::new_no_reset();

        cpu.exception(exception::Vector::ResetSspPc as u8);

        cpu
    }

    /// [Self::new] but without the initial reset vectors, so you can initialize the core as you want.
    pub fn new_no_reset() -> Self {
        Self {
            d: [0; 8],
            a_: [0; 7],
            usp: 0,
            ssp: 0,
            sr: StatusRegister::default(),
            pc: 0,

            current_opcode: 0xFFFF,
            stop: false,
            exceptions: BinaryHeap::new(),
            cycles: 0,
            disassemble: false,
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
