// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Motorola 68000 interpreter, disassembler and assembler (code emitter).
//!
//! This library emulates the common user and supervisor instructions of the M68k ISA.
//! It is configurable to behave like the given CPU type (see below), changing the instruction's execution times and exception handling.
//!
//! This library has been designed to be used in two different contexts:
//!
//! - It can be used to emulate a whole CPU, and the user of this library only have to call the interpreter methods
//! and [M68000::exception] when an interrupt or reset occurs. This is usually the case in an emulator.
//! - It can also be used as a M68k user-land interpreter to run an M68k program, but without the requirement of having an
//! operating system compiled to binary M68k. In this case, the application runs the program until an exception occurs (TRAP for
//! syscalls, zero divide, etc.) and treat the exception in Rust code (or any other language using the C interface), so the
//! application can implement the surrounding environment required by the M68k program in a high level language and not in M68k assembly.
//!
//! # Supported CPUs
//!
//! The CPU type is specified with a generic parameter on the main structure.
//! The trait [CpuDetails] contains all the details of the emulated CPU:
//! - Instruction execution times
//! - Exception processing times
//! - Exception stack format
//!
//! m68000 provides CPU details for the following CPUs:
//! * MC68000 (as described in the M68000 8-/16-/32-Bit Microprocessors User's Manual, Ninth Edition)
//! * SCC68070 microcontroller
//!
//! # How to use
//!
//! m68000 requires a nightly compiler as it uses the `btree_extract_if` and `bigint_helper_methods` features of the std.
//!
//! First, since the memory map is application-dependant, it is the user's responsibility to define it by implementing
//! the `MemoryAccess` trait on their memory structure, and passing it to the core on each instruction execution.
//!
//! Second, choose the CPU behavior by specifying the instance that implements the `CpuDetails` trait,
//! whether it is your own or one the provided ones.
//!
//! The file `src/bin/scc68070.rs` is a usage example that implements the SCC68070 microcontroller.
//!
//! ## Basic usage:
//!
//! ```ignore
//! const MEM_SIZE: u32 = 65536;
//! struct Memory([u8; MEM_SIZE as usize]); // Define your memory management system.
//!
//! impl MemoryAccess for Memory { // Implement the MemoryAccess trait.
//!     fn get_byte(&mut self, addr: u32) -> Option<u8> {
//!         if addr < MEM_SIZE {
//!             Some(self.0[addr as usize])
//!         } else {
//!             None
//!         }
//!     }
//!
//!     // And so on...
//! }
//!
//! fn main() {
//!     let mut memory = Memory([0; MEM_SIZE as usize]);
//!     // Load the program in memory here.
//!     let mut cpu: M68000<m68000::cpu_details::Mc68000> = M68000::new();
//!
//!     // Execute instructions
//!     cpu.interpreter(&mut memory);
//! }
//! ```
//!
//! ## FFI and C interface
//!
//! By enabling the `ffi` feature, the following structs and enums are made `repr(C)`:
//! - [AddressingMode](addressing_modes::AddressingMode)
//! - [BriefExtensionWord](addressing_modes::BriefExtensionWord)
//! - [Direction](instruction::Direction)
//! - [Instruction](instruction::Instruction)
//! - [Operands](instruction::Operands)
//! - [Registers]
//! - [Size](instruction::Size)
//! - [StatusRegister]
//! - [Vector]
//!
//! The crate `m68000-ffi` in the repo is a collection of structures and functions that allows using m68000's
//! interpreter and disassembler in other languages through a C interface.
//!
//! See the `CINTERFACE.md` file in the repo for more information.
//!
//! ## Potential issues
//! - DIVS/DIVU may not always procuce the correct CCR flags when an overflow occured.
//! - DIVS/DIVU always execute using their maximum execution time.
//! - Long exception stack frame is not implemented.
//!
//! ## TODO
//! - Let memory access return extra read or write cycles for accuracy.

#![feature(bigint_helper_methods)]
#![feature(btree_extract_if)]

pub mod addressing_modes;
pub mod assembler;
pub mod decoder;
pub mod disassembler;
pub mod exception;
pub mod cpu_details;
pub mod instruction;
mod interpreter;
mod interpreter_disassembler;
mod interpreter_fast;
pub mod isa;
pub mod memory_access;
pub mod status_register;
pub mod utils;

use exception::{Exception, Vector};
pub use cpu_details::{CpuDetails, StackFormat};
pub use memory_access::MemoryAccess;
use status_register::StatusRegister;

use std::collections::BTreeSet;
use std::num::Wrapping;

/// M68000 registers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "ffi", repr(C))]
pub struct Registers {
    /// Data registers.
    pub d: [Wrapping<u32>; 8],
    /// Address registers.
    pub a: [Wrapping<u32>; 7],
    /// User Stack Pointer.
    pub usp: Wrapping<u32>,
    /// System Stack Pointer.
    pub ssp: Wrapping<u32>,
    /// Status Register.
    pub sr: StatusRegister,
    /// Program Counter.
    pub pc: Wrapping<u32>,
}

impl Registers {
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
            self.a[reg as usize].0
        } else {
            self.sp()
        }
    }

    /// Returns a mutable reference to an address register.
    pub fn a_mut(&mut self, reg: u8) -> &mut Wrapping<u32> {
        if reg < 7 {
            &mut self.a[reg as usize]
        } else {
            self.sp_mut()
        }
    }

    /// Returns the stack pointer, SSP if in supervisor mode, USP if in user mode.
    pub const fn sp(&self) -> u32 {
        if self.sr.s {
            self.ssp.0
        } else {
            self.usp.0
        }
    }

    /// Returns a mutable reference to the stack pointer, SSP if in supervisor mode, USP if in user mode.
    pub fn sp_mut(&mut self) -> &mut Wrapping<u32> {
        if self.sr.s {
            &mut self.ssp
        } else {
            &mut self.usp
        }
    }
}

/// A M68000 core.
#[derive(Clone, Debug)]
pub struct M68000<CPU: CpuDetails> {
    /// The registers of the CPU.
    pub regs: Registers,

    /// The opcode of the instruction currently executing. Stored because it is an information of the long exception stack frame.
    current_opcode: u16,
    /// True if the CPU is stopped (after a STOP instruction), false to switch back to normal instruction execution.
    pub stop: bool,
    /// The pending exceptions. Low priority are popped first (MC68000UM 6.2.3 Multiple Exceptions).
    exceptions: BTreeSet<exception::Exception>,
    /// The details of the emulated CPU.
    _cpu: CPU,
}

impl<CPU: CpuDetails> M68000<CPU> {
    /// Creates a new M68000 core.
    ///
    /// The created core has a [Reset vector](crate::exception::Vector::ResetSspPc) pushed, so that the first call to an
    /// interpreter method will first fetch the reset vectors, then will execute the first instruction.
    ///
    /// The returned core already has a Status Register set to 0x2700 (supervisor bit set, interrupt level 7).
    pub fn new() -> Self {
        let mut cpu = Self::new_no_reset();

        cpu.exception(Exception::from(Vector::ResetSspPc));

        cpu
    }

    /// [Self::new] but without the initial reset, so you can initialize the core as you want.
    ///
    /// The returned core already has a Status Register set to 0x2700 (supervisor bit set, interrupt level 7).
    pub fn new_no_reset() -> Self {
        Self {
            regs: Registers::default(),

            current_opcode: 0xFFFF,
            stop: false,
            exceptions: BTreeSet::new(),
            _cpu: CPU::default(),
        }
    }
}
