//! This module defines the Instruction structure and the different operands that can be found inside an opcode.
//! Its responsibility is only to retrive the operands and format them approprately.
//! It is the interpreter's role to interpret the operand.

use super::{M68000, MemoryAccess};
// use super::decoder::DECODER;
use super::isa::Isa;
use super::operands::Operands;

pub(super) struct Instruction {
    // TODO: IsaEntry ?
    pub isa: Isa,
    /// The opcode itself.
    pub opcode: u16,
    /// The address of the instruction.
    pub pc: u32,
    /// The operands.
    pub operands: Operands,
}

impl<M: MemoryAccess> M68000<M> {
    // pub(super) fn get_next_instruction(&mut self) -> Instruction {
    //     let pc = self.pc;
    //     let opcode = self.get_next_word();
    //     let isa = DECODER[opcode as usize];
    //     let entry = &Self::ISA_ENTRY[isa as usize];

    //     let (operands, _) = (entry.decode)(isa, self.memory.iter(pc));

    //     Instruction {
    //         isa,
    //         opcode,
    //         pc,
    //         operands,
    //     }
    // }
}
