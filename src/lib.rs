//! Motorola 68000 emulator.
//!
//! # TODO:
//! - Calculation times.
//! - Exceptions.
//! - Documentation.

mod addressing_modes;
mod decoder;
mod disassembler;
mod instruction;
mod interpreter;
pub mod isa;
pub mod memory_access;
mod operand;
mod status_register;
mod utils;

use memory_access::MemoryAccess;
use status_register::StatusRegister;

#[derive(Debug)]
pub struct M68000<M: MemoryAccess> {
    d: [u32; 8],
    a_: [u32; 7],
    usp: u32,
    ssp: u32,
    sr: StatusRegister,
    pub pc: u32,

    memory: M,
    current_opcode: u16,
    current_pc: u32,
}

impl<M: MemoryAccess> M68000<M> {
    pub fn new(memory: M) -> Self {
        Self {
            d: [0; 8],
            a_: [0; 7],
            usp: 0,
            ssp: 0,
            sr: StatusRegister::default(),
            pc: 0,

            memory,
            current_opcode: 0xFFFF,
            current_pc: 0,
        }
    }

    fn a(&self, reg: usize) -> u32 {
        if reg < 7 {
            self.a_[reg]
        } else {
            if self.sr.s {
                self.ssp
            } else {
                self.usp
            }
        }
    }

    fn a_mut(&mut self, reg: usize) -> &mut u32 {
        if reg < 7 {
            &mut self.a_[reg]
        } else {
            if self.sr.s {
                &mut self.ssp
            } else {
                &mut self.usp
            }
        }
    }
}
