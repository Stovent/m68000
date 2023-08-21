// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! ISA definition and helper structs to decode, disassemble and interpret (internal only) the instructions.

use crate::decoder::DECODER;
use crate::memory_access::{MemoryAccess, MemoryIter};
use crate::instruction::*;

/// ISA of the M68000.
///
/// Converts a raw opcode to this enum using the [from](Self::from) method.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Isa {
    Unknown,
    Abcd,
    Add,
    Adda,
    Addi,
    Addq,
    Addx,
    And,
    Andi,
    Andiccr,
    Andisr,
    Asm,
    Asr,
    Bcc,
    Bchg,
    Bclr,
    Bra,
    Bset,
    Bsr,
    Btst,
    Chk,
    Clr,
    Cmp,
    Cmpa,
    Cmpi,
    Cmpm,
    Dbcc,
    Divs,
    Divu,
    Eor,
    Eori,
    Eoriccr,
    Eorisr,
    Exg,
    Ext,
    Illegal,
    Jmp,
    Jsr,
    Lea,
    Link,
    Lsm,
    Lsr,
    Move,
    Movea,
    Moveccr,
    Movefsr,
    Movesr,
    Moveusp,
    Movem,
    Movep,
    Moveq,
    Muls,
    Mulu,
    Nbcd,
    Neg,
    Negx,
    Nop,
    Not,
    Or,
    Ori,
    Oriccr,
    Orisr,
    Pea,
    Reset,
    Rom,
    Ror,
    Roxm,
    Roxr,
    Rte,
    Rtr,
    Rts,
    Sbcd,
    Scc,
    Stop,
    Sub,
    Suba,
    Subi,
    Subq,
    Subx,
    Swap,
    Tas,
    Trap,
    Trapv,
    Tst,
    Unlk,
    _Size,
}

impl Isa {
    /// Returns whether the instruction is privileged or not.
    ///
    /// Privileged instructions are not traced (MC68000UM 6.3.8 Tracing).
    pub const fn is_privileged(self) -> bool {
        match self {
            Self::Andisr => true,
            Self::Eorisr => true,
            Self::Movesr => true,
            Self::Moveusp => true,
            Self::Orisr => true,
            Self::Reset => true,
            Self::Rte => true,
            Self::Stop => true,
            _ => false,
        }
    }
}

impl From<u16> for Isa {
    /// Returns the instruction represented by the given opcode.
    fn from(opcode: u16) -> Self {
        DECODER[opcode as usize]
    }
}

/// Struct used to store the decode functions of an instruction.
///
/// # Usage:
///
/// ```ignore
/// use m68000::isa::{Isa, IsaEntry};
///
/// let opcode = 0; // Read raw opcode here.
/// let isa = Isa::from(opcode);
///
/// // Decode
/// let decode = IsaEntry::ISA_ENTRY[isa as usize].decode;
/// let (operands, len) = decode(opcode, memory); // Give here the memory structure.
///
/// let instruction = Instruction {
///     opcode,
///     pc,
///     operands,
/// };
/// ```
#[derive(Clone, Copy)]
pub struct IsaEntry<M: MemoryAccess + ?Sized> {
    // /// The ISA value.
    // pub isa: Isa,
    /// Function used to decode the instruction. See the [instruction](crate::instruction) module.
    pub decode: fn(u16, &mut MemoryIter<M>) -> Operands,
}

impl<M: MemoryAccess + ?Sized> IsaEntry<M> {
    /// The array that maps instructions to their [IsaEntry] entry. Index it using the [Isa] enum.
    pub const ISA_ENTRY: [IsaEntry<M>; Isa::_Size as usize] = [
        IsaEntry { /* isa: Isa::Unknown,*/ decode: Operands::new_no_operands, },
        IsaEntry { /* isa: Isa::Abcd,*/    decode: Operands::new_register_size_mode_register, },
        IsaEntry { /* isa: Isa::Add,*/     decode: Operands::new_register_direction_size_effective_address, },
        IsaEntry { /* isa: Isa::Adda,*/    decode: Operands::new_register_size_effective_address, },
        IsaEntry { /* isa: Isa::Addi,*/    decode: Operands::new_size_effective_address_immediate, },
        IsaEntry { /* isa: Isa::Addq,*/    decode: Operands::new_data_size_effective_address, },
        IsaEntry { /* isa: Isa::Addx,*/    decode: Operands::new_register_size_mode_register, },
        IsaEntry { /* isa: Isa::And,*/     decode: Operands::new_register_direction_size_effective_address, },
        IsaEntry { /* isa: Isa::Andi,*/    decode: Operands::new_size_effective_address_immediate, },
        IsaEntry { /* isa: Isa::Andiccr,*/ decode: Operands::new_immediate, },
        IsaEntry { /* isa: Isa::Andisr,*/  decode: Operands::new_immediate, },
        IsaEntry { /* isa: Isa::Asm,*/     decode: Operands::new_direction_effective_address, },
        IsaEntry { /* isa: Isa::Asr,*/     decode: Operands::new_rotation_direction_size_mode_register, },
        IsaEntry { /* isa: Isa::Bcc,*/     decode: Operands::new_condition_displacement, },
        IsaEntry { /* isa: Isa::Bchg,*/    decode: Operands::new_effective_address_count, },
        IsaEntry { /* isa: Isa::Bclr,*/    decode: Operands::new_effective_address_count, },
        IsaEntry { /* isa: Isa::Bra,*/     decode: Operands::new_displacement, },
        IsaEntry { /* isa: Isa::Bset,*/    decode: Operands::new_effective_address_count, },
        IsaEntry { /* isa: Isa::Bsr,*/     decode: Operands::new_displacement, },
        IsaEntry { /* isa: Isa::Btst,*/    decode: Operands::new_effective_address_count, },
        IsaEntry { /* isa: Isa::Chk,*/     decode: Operands::new_register_effective_address, },
        IsaEntry { /* isa: Isa::Clr,*/     decode: Operands::new_size_effective_address, },
        IsaEntry { /* isa: Isa::Cmp,*/     decode: Operands::new_register_direction_size_effective_address, },
        IsaEntry { /* isa: Isa::Cmpa,*/    decode: Operands::new_register_size_effective_address, },
        IsaEntry { /* isa: Isa::Cmpi,*/    decode: Operands::new_size_effective_address_immediate, },
        IsaEntry { /* isa: Isa::Cmpm,*/    decode: Operands::new_register_size_register, },
        IsaEntry { /* isa: Isa::Dbcc,*/    decode: Operands::new_condition_register_displacement, },
        IsaEntry { /* isa: Isa::Divs,*/    decode: Operands::new_register_effective_address, },
        IsaEntry { /* isa: Isa::Divu,*/    decode: Operands::new_register_effective_address, },
        IsaEntry { /* isa: Isa::Eor,*/     decode: Operands::new_register_direction_size_effective_address, },
        IsaEntry { /* isa: Isa::Eori,*/    decode: Operands::new_size_effective_address_immediate, },
        IsaEntry { /* isa: Isa::Eoriccr,*/ decode: Operands::new_immediate, },
        IsaEntry { /* isa: Isa::Eorisr,*/  decode: Operands::new_immediate, },
        IsaEntry { /* isa: Isa::Exg,*/     decode: Operands::new_register_opmode_register, },
        IsaEntry { /* isa: Isa::Ext,*/     decode: Operands::new_opmode_register, },
        IsaEntry { /* isa: Isa::Illegal,*/ decode: Operands::new_no_operands, },
        IsaEntry { /* isa: Isa::Jmp,*/     decode: Operands::new_effective_address, },
        IsaEntry { /* isa: Isa::Jsr,*/     decode: Operands::new_effective_address, },
        IsaEntry { /* isa: Isa::Lea,*/     decode: Operands::new_register_effective_address, },
        IsaEntry { /* isa: Isa::Link,*/    decode: Operands::new_register_displacement, },
        IsaEntry { /* isa: Isa::Lsm,*/     decode: Operands::new_direction_effective_address, },
        IsaEntry { /* isa: Isa::Lsr,*/     decode: Operands::new_rotation_direction_size_mode_register, },
        IsaEntry { /* isa: Isa::Move,*/    decode: Operands::new_size_effective_address_effective_address, },
        IsaEntry { /* isa: Isa::Movea,*/   decode: Operands::new_size_register_effective_address, },
        IsaEntry { /* isa: Isa::Moveccr,*/ decode: Operands::new_effective_address, },
        IsaEntry { /* isa: Isa::Movefsr,*/ decode: Operands::new_effective_address, },
        IsaEntry { /* isa: Isa::Movesr,*/  decode: Operands::new_effective_address, },
        IsaEntry { /* isa: Isa::Moveusp,*/ decode: Operands::new_direction_register, },
        IsaEntry { /* isa: Isa::Movem,*/   decode: Operands::new_direction_size_effective_address_list, },
        IsaEntry { /* isa: Isa::Movep,*/   decode: Operands::new_register_direction_size_register_displacement, },
        IsaEntry { /* isa: Isa::Moveq,*/   decode: Operands::new_register_data, },
        IsaEntry { /* isa: Isa::Muls,*/    decode: Operands::new_register_effective_address, },
        IsaEntry { /* isa: Isa::Mulu,*/    decode: Operands::new_register_effective_address, },
        IsaEntry { /* isa: Isa::Nbcd,*/    decode: Operands::new_effective_address, },
        IsaEntry { /* isa: Isa::Neg,*/     decode: Operands::new_size_effective_address, },
        IsaEntry { /* isa: Isa::Negx,*/    decode: Operands::new_size_effective_address, },
        IsaEntry { /* isa: Isa::Nop,*/     decode: Operands::new_no_operands, },
        IsaEntry { /* isa: Isa::Not,*/     decode: Operands::new_size_effective_address, },
        IsaEntry { /* isa: Isa::Or,*/      decode: Operands::new_register_direction_size_effective_address, },
        IsaEntry { /* isa: Isa::Ori,*/     decode: Operands::new_size_effective_address_immediate, },
        IsaEntry { /* isa: Isa::Oriccr,*/  decode: Operands::new_immediate, },
        IsaEntry { /* isa: Isa::Orisr,*/   decode: Operands::new_immediate, },
        IsaEntry { /* isa: Isa::Pea,*/     decode: Operands::new_effective_address, },
        IsaEntry { /* isa: Isa::Reset,*/   decode: Operands::new_no_operands, },
        IsaEntry { /* isa: Isa::Rom,*/     decode: Operands::new_direction_effective_address, },
        IsaEntry { /* isa: Isa::Ror,*/     decode: Operands::new_rotation_direction_size_mode_register, },
        IsaEntry { /* isa: Isa::Roxm,*/    decode: Operands::new_direction_effective_address, },
        IsaEntry { /* isa: Isa::Roxr,*/    decode: Operands::new_rotation_direction_size_mode_register, },
        IsaEntry { /* isa: Isa::Rte,*/     decode: Operands::new_no_operands, },
        IsaEntry { /* isa: Isa::Rtr,*/     decode: Operands::new_no_operands, },
        IsaEntry { /* isa: Isa::Rts,*/     decode: Operands::new_no_operands, },
        IsaEntry { /* isa: Isa::Sbcd,*/    decode: Operands::new_register_size_mode_register, },
        IsaEntry { /* isa: Isa::Scc,*/     decode: Operands::new_condition_effective_address, },
        IsaEntry { /* isa: Isa::Stop,*/    decode: Operands::new_immediate, },
        IsaEntry { /* isa: Isa::Sub,*/     decode: Operands::new_register_direction_size_effective_address, },
        IsaEntry { /* isa: Isa::Suba,*/    decode: Operands::new_register_size_effective_address, },
        IsaEntry { /* isa: Isa::Subi,*/    decode: Operands::new_size_effective_address_immediate, },
        IsaEntry { /* isa: Isa::Subq,*/    decode: Operands::new_data_size_effective_address, },
        IsaEntry { /* isa: Isa::Subx,*/    decode: Operands::new_register_size_mode_register, },
        IsaEntry { /* isa: Isa::Swap,*/    decode: Operands::new_register, },
        IsaEntry { /* isa: Isa::Tas,*/     decode: Operands::new_effective_address, },
        IsaEntry { /* isa: Isa::Trap,*/    decode: Operands::new_vector, },
        IsaEntry { /* isa: Isa::Trapv,*/   decode: Operands::new_no_operands, },
        IsaEntry { /* isa: Isa::Tst,*/     decode: Operands::new_size_effective_address, },
        IsaEntry { /* isa: Isa::Unlk,*/    decode: Operands::new_register, },
    ];
}
