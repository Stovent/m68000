//! Motorola 68000 assembler, disassembler and interpreter.
//!
//! This library emulates the common user and supervisor instructions of the M68k ISA.
//! It is configurable at compile-time to behave like the given CPU type (see below), changing the instruction's
//! execution times and exception handling.
//!
//! This library has been designed to be used in two different contexts:
//! - It can be used to emulate a whole CPU, and the user of this library only have to call the interpreter methods,
//! [M68000::reset] and [M68000::exception] when an interrupt occurs. This is usually the case in an emulator.
//! - It can also be used as a M68k user-land interpreter to run an M68k program, but without the requirement of having an
//! operating system compiled to binary M68k. In this case, the application runs the program until an exception occurs (TRAP for
//! syscalls, zero divide, etc.) and treat the exception in Rust code (or any other language using the C interface), so the
//! application can implement the surrounding environment required by the M68k program in a high level language and not in M68k assembly.
//!
//! # Supported CPUs
//!
//! The CPU type is specified at compile-time as a feature. There must be one and only one feature specified.
//!
//! There are no default features. If you don't specify any feature or specify more than one, a compile-time error is raised.
//!
//! * MC68000 (feature `cpu-mc68000`)
//! * SCC68070 (feature `cpu-scc68070`)
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
//! ## C interface
//!
//! This library has a C interface, see the README and the [cinterface] module for how to use it.
//!
//! # TODO:
//! - Let memory access return extra read or write cycles for accuracy.
//! - Verify ABCD, NBCD, SBCD, DIVS and DIVU instructions.

#![feature(btree_drain_filter)]

#[cfg(any(
    all(feature = "cpu-mc68000", feature = "cpu-scc68070"),
    not(any(feature = "cpu-mc68000", feature = "cpu-scc68070")),
))]
compile_error!("You must specify one and only one CPU type feature.");

pub mod addressing_modes;
pub mod assembler;
#[cfg(doc)]
pub mod cinterface;
#[cfg(not(doc))]
mod cinterface;
pub mod decoder;
pub mod disassembler;
pub mod exception;
mod fast_interpreter;
mod fast_operands;
pub mod instruction;
mod interpreter;
mod instruction_interpreter;
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

use std::collections::BTreeSet;

/// M68000 registers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct Registers {
    /// Data registers.
    pub d: [u32; 8],
    /// Address registers.
    pub a: [u32; 7],
    /// User Stack Pointer.
    pub usp: u32,
    /// System Stack Pointer.
    pub ssp: u32,
    /// Status Register.
    pub sr: StatusRegister,
    /// Program Counter.
    pub pc: u32,
}

/// A M68000 core.
#[derive(Clone, Debug)]
pub struct M68000 {
    pub regs: Registers,

    current_opcode: u16,
    /// True if the CPU is stopped (after a STOP instruction), false to switch back to normal instruction execution.
    pub stop: bool,
    /// The pending exceptions. Low priority are popped first (MC68000UM 6.2.3 Multiple Exceptions).
    exceptions: BTreeSet<exception::Exception>,
    /// Number of cycles executed by the called interpreter method.
    cycles: usize,
}

impl M68000 {
    /// Creates a new M68000 core and resets it by fetching the reset vectors.
    pub fn new_reset(memory: &mut impl MemoryAccess) -> Self {
        let mut cpu = Self::new_no_reset();

        cpu.reset(memory);

        cpu
    }

    /// [Self::new_reset] but without the initial reset, so you can initialize the core as you want, or call [Self::reset] later.
    pub fn new_no_reset() -> Self {
        Self {
            regs: Registers {
                d: [0; 8],
                a: [0; 7],
                usp: 0,
                ssp: 0,
                sr: StatusRegister::default(),
                pc: 0,
            },

            current_opcode: 0xFFFF,
            stop: false,
            exceptions: BTreeSet::new(),
            cycles: 0,
        }
    }

    /// Resets the CPU by fetching the reset vectors.
    pub fn reset(&mut self, memory: &mut impl MemoryAccess) {
        self.regs.ssp = memory.get_long(0).expect("An exception occured when reading initial SSP.");
        self.regs.pc  = memory.get_long(4).expect("An exception occured when reading initial PC.");
        self.regs.sr.t = false;
        self.regs.sr.s = true;
        self.regs.sr.interrupt_mask = 7;
        self.stop = false;
        self.exceptions.clear(); // The reset vector clears all the pending interrupts.
        // return Ok(EXEC::VECTOR_RESET);
    }

    /// Sets the lower 8-bits of the given data register to the given value.
    /// The higher 24-bits remains untouched.
    pub fn d_byte(&mut self, reg: u8, value: u8) {
        self.regs.d[reg as usize] &= 0xFFFF_FF00;
        self.regs.d[reg as usize] |= value as u32;
    }

    /// Sets the lower 16-bits of the given data register to the given value.
    /// The higher 16-bits remains untouched.
    pub fn d_word(&mut self, reg: u8, value: u16) {
        self.regs.d[reg as usize] &= 0xFFFF_0000;
        self.regs.d[reg as usize] |= value as u32;
    }

    /// Returns an address register.
    pub const fn a(&self, reg: u8) -> u32 {
        if reg < 7 {
            self.regs.a[reg as usize]
        } else {
            self.sp()
        }
    }

    /// Returns a mutable reference to an address register.
    pub fn a_mut(&mut self, reg: u8) -> &mut u32 {
        if reg < 7 {
            &mut self.regs.a[reg as usize]
        } else {
            self.sp_mut()
        }
    }

    /// Returns the stack pointer, SSP if in supervisor mode, USP if in user mode.
    pub const fn sp(&self) -> u32 {
        if self.regs.sr.s {
            self.regs.ssp
        } else {
            self.regs.usp
        }
    }

    /// Returns a mutable reference to the stack pointer, SSP if in supervisor mode, USP if in user mode.
    pub fn sp_mut(&mut self) -> &mut u32 {
        if self.regs.sr.s {
            &mut self.regs.ssp
        } else {
            &mut self.regs.usp
        }
    }
}
