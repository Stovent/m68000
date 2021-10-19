//! This module only defines the Instruction structure.

use super::operands::Operands;

pub(super) struct Instruction {
    /// The opcode itself.
    pub opcode: u16,
    /// The address of the instruction.
    pub pc: u32,
    /// The operands.
    pub operands: Operands,
}
