// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! ISA definition and helper structs to decode, disassemble and interpret (internal only) the instructions.

use crate::decoder::DECODER;
use crate::disassembler::*;
use crate::instruction::Instruction;
use crate::memory_access::MemoryIter;
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

impl From<u16> for Isa {
    /// Returns the instruction represented by the given opcode.
    fn from(opcode: u16) -> Self {
        DECODER[opcode as usize]
    }
}

/// Struct used to store the decode and disassemble functions of an instruction.
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
///
/// // Disassemble
/// let inst = (IsaEntry::ISA_ENTRY[isa as usize].disassemble)(&instruction);
/// println!("{}", inst);
/// ```
#[derive(Clone, Copy)]
pub struct IsaEntry {
    // /// The ISA value.
    // pub isa: Isa,
    /// Function used to decode the instruction. See the [instruction](crate::instruction) module.
    pub decode: fn(u16, &mut MemoryIter) -> Operands,
    /// Function used to diassemble the instruction.
    pub disassemble: fn(&Instruction) -> String,
}

impl IsaEntry {
    /// The array that maps instructions to their [IsaEntry] entry. Index it using the [Isa] enum.
    pub const ISA_ENTRY: [IsaEntry; Isa::_Size as usize] = [
        IsaEntry { /* isa: Isa::Unknown,*/ decode: Operands::new_no_operands,                                   disassemble: disassemble_unknown_instruction, },
        IsaEntry { /* isa: Isa::Abcd,*/    decode: Operands::new_register_size_mode_register,                   disassemble: disassemble_abcd, },
        IsaEntry { /* isa: Isa::Add,*/     decode: Operands::new_register_direction_size_effective_address,     disassemble: disassemble_add, },
        IsaEntry { /* isa: Isa::Adda,*/    decode: Operands::new_register_size_effective_address,               disassemble: disassemble_adda, },
        IsaEntry { /* isa: Isa::Addi,*/    decode: Operands::new_size_effective_address_immediate,              disassemble: disassemble_addi, },
        IsaEntry { /* isa: Isa::Addq,*/    decode: Operands::new_data_size_effective_address,                   disassemble: disassemble_addq, },
        IsaEntry { /* isa: Isa::Addx,*/    decode: Operands::new_register_size_mode_register,                   disassemble: disassemble_addx, },
        IsaEntry { /* isa: Isa::And,*/     decode: Operands::new_register_direction_size_effective_address,     disassemble: disassemble_and, },
        IsaEntry { /* isa: Isa::Andi,*/    decode: Operands::new_size_effective_address_immediate,              disassemble: disassemble_andi, },
        IsaEntry { /* isa: Isa::Andiccr,*/ decode: Operands::new_immediate,                                     disassemble: disassemble_andiccr },
        IsaEntry { /* isa: Isa::Andisr,*/  decode: Operands::new_immediate,                                     disassemble: disassemble_andisr, },
        IsaEntry { /* isa: Isa::Asm,*/     decode: Operands::new_direction_effective_address,                   disassemble: disassemble_asm, },
        IsaEntry { /* isa: Isa::Asr,*/     decode: Operands::new_rotation_direction_size_mode_register,         disassemble: disassemble_asr, },
        IsaEntry { /* isa: Isa::Bcc,*/     decode: Operands::new_condition_displacement,                        disassemble: disassemble_bcc, },
        IsaEntry { /* isa: Isa::Bchg,*/    decode: Operands::new_effective_address_count,                       disassemble: disassemble_bchg, },
        IsaEntry { /* isa: Isa::Bclr,*/    decode: Operands::new_effective_address_count,                       disassemble: disassemble_bclr, },
        IsaEntry { /* isa: Isa::Bra,*/     decode: Operands::new_displacement,                                  disassemble: disassemble_bra, },
        IsaEntry { /* isa: Isa::Bset,*/    decode: Operands::new_effective_address_count,                       disassemble: disassemble_bset, },
        IsaEntry { /* isa: Isa::Bsr,*/     decode: Operands::new_displacement,                                  disassemble: disassemble_bsr, },
        IsaEntry { /* isa: Isa::Btst,*/    decode: Operands::new_effective_address_count,                       disassemble: disassemble_btst, },
        IsaEntry { /* isa: Isa::Chk,*/     decode: Operands::new_register_effective_address,                    disassemble: disassemble_chk, },
        IsaEntry { /* isa: Isa::Clr,*/     decode: Operands::new_size_effective_address,                        disassemble: disassemble_clr, },
        IsaEntry { /* isa: Isa::Cmp,*/     decode: Operands::new_register_direction_size_effective_address,     disassemble: disassemble_cmp, },
        IsaEntry { /* isa: Isa::Cmpa,*/    decode: Operands::new_register_size_effective_address,               disassemble: disassemble_cmpa, },
        IsaEntry { /* isa: Isa::Cmpi,*/    decode: Operands::new_size_effective_address_immediate,              disassemble: disassemble_cmpi, },
        IsaEntry { /* isa: Isa::Cmpm,*/    decode: Operands::new_register_size_register,                        disassemble: disassemble_cmpm, },
        IsaEntry { /* isa: Isa::Dbcc,*/    decode: Operands::new_condition_register_displacement,               disassemble: disassemble_dbcc, },
        IsaEntry { /* isa: Isa::Divs,*/    decode: Operands::new_register_effective_address,                    disassemble: disassemble_divs, },
        IsaEntry { /* isa: Isa::Divu,*/    decode: Operands::new_register_effective_address,                    disassemble: disassemble_divu, },
        IsaEntry { /* isa: Isa::Eor,*/     decode: Operands::new_register_direction_size_effective_address,     disassemble: disassemble_eor, },
        IsaEntry { /* isa: Isa::Eori,*/    decode: Operands::new_size_effective_address_immediate,              disassemble: disassemble_eori, },
        IsaEntry { /* isa: Isa::Eoriccr,*/ decode: Operands::new_immediate,                                     disassemble: disassemble_eoriccr, },
        IsaEntry { /* isa: Isa::Eorisr,*/  decode: Operands::new_immediate,                                     disassemble: disassemble_eorisr, },
        IsaEntry { /* isa: Isa::Exg,*/     decode: Operands::new_register_opmode_register,                      disassemble: disassemble_exg, },
        IsaEntry { /* isa: Isa::Ext,*/     decode: Operands::new_opmode_register,                               disassemble: disassemble_ext, },
        IsaEntry { /* isa: Isa::Illegal,*/ decode: Operands::new_no_operands,                                   disassemble: disassemble_illegal, },
        IsaEntry { /* isa: Isa::Jmp,*/     decode: Operands::new_effective_address,                             disassemble: disassemble_jmp, },
        IsaEntry { /* isa: Isa::Jsr,*/     decode: Operands::new_effective_address,                             disassemble: disassemble_jsr, },
        IsaEntry { /* isa: Isa::Lea,*/     decode: Operands::new_register_effective_address,                    disassemble: disassemble_lea, },
        IsaEntry { /* isa: Isa::Link,*/    decode: Operands::new_register_displacement,                         disassemble: disassemble_link, },
        IsaEntry { /* isa: Isa::Lsm,*/     decode: Operands::new_direction_effective_address,                   disassemble: disassemble_lsm, },
        IsaEntry { /* isa: Isa::Lsr,*/     decode: Operands::new_rotation_direction_size_mode_register,         disassemble: disassemble_lsr, },
        IsaEntry { /* isa: Isa::Move,*/    decode: Operands::new_size_effective_address_effective_address,      disassemble: disassemble_move, },
        IsaEntry { /* isa: Isa::Movea,*/   decode: Operands::new_size_register_effective_address,               disassemble: disassemble_movea, },
        IsaEntry { /* isa: Isa::Moveccr,*/ decode: Operands::new_effective_address,                             disassemble: disassemble_moveccr, },
        IsaEntry { /* isa: Isa::Movefsr,*/ decode: Operands::new_effective_address,                             disassemble: disassemble_movefsr, },
        IsaEntry { /* isa: Isa::Movesr,*/  decode: Operands::new_effective_address,                             disassemble: disassemble_movesr, },
        IsaEntry { /* isa: Isa::Moveusp,*/ decode: Operands::new_direction_register,                            disassemble: disassemble_moveusp, },
        IsaEntry { /* isa: Isa::Movem,*/   decode: Operands::new_direction_size_effective_address_list,         disassemble: disassemble_movem, },
        IsaEntry { /* isa: Isa::Movep,*/   decode: Operands::new_register_direction_size_register_displacement, disassemble: disassemble_movep, },
        IsaEntry { /* isa: Isa::Moveq,*/   decode: Operands::new_register_data,                                 disassemble: disassemble_moveq, },
        IsaEntry { /* isa: Isa::Muls,*/    decode: Operands::new_register_effective_address,                    disassemble: disassemble_muls, },
        IsaEntry { /* isa: Isa::Mulu,*/    decode: Operands::new_register_effective_address,                    disassemble: disassemble_mulu, },
        IsaEntry { /* isa: Isa::Nbcd,*/    decode: Operands::new_effective_address,                             disassemble: disassemble_nbcd, },
        IsaEntry { /* isa: Isa::Neg,*/     decode: Operands::new_size_effective_address,                        disassemble: disassemble_neg, },
        IsaEntry { /* isa: Isa::Negx,*/    decode: Operands::new_size_effective_address,                        disassemble: disassemble_negx, },
        IsaEntry { /* isa: Isa::Nop,*/     decode: Operands::new_no_operands,                                   disassemble: disassemble_nop, },
        IsaEntry { /* isa: Isa::Not,*/     decode: Operands::new_size_effective_address,                        disassemble: disassemble_not, },
        IsaEntry { /* isa: Isa::Or,*/      decode: Operands::new_register_direction_size_effective_address,     disassemble: disassemble_or, },
        IsaEntry { /* isa: Isa::Ori,*/     decode: Operands::new_size_effective_address_immediate,              disassemble: disassemble_ori, },
        IsaEntry { /* isa: Isa::Oriccr,*/  decode: Operands::new_immediate,                                     disassemble: disassemble_oriccr, },
        IsaEntry { /* isa: Isa::Orisr,*/   decode: Operands::new_immediate,                                     disassemble: disassemble_orisr, },
        IsaEntry { /* isa: Isa::Pea,*/     decode: Operands::new_effective_address,                             disassemble: disassemble_pea, },
        IsaEntry { /* isa: Isa::Reset,*/   decode: Operands::new_no_operands,                                   disassemble: disassemble_reset, },
        IsaEntry { /* isa: Isa::Rom,*/     decode: Operands::new_direction_effective_address,                   disassemble: disassemble_rom, },
        IsaEntry { /* isa: Isa::Ror,*/     decode: Operands::new_rotation_direction_size_mode_register,         disassemble: disassemble_ror, },
        IsaEntry { /* isa: Isa::Roxm,*/    decode: Operands::new_direction_effective_address,                   disassemble: disassemble_roxm, },
        IsaEntry { /* isa: Isa::Roxr,*/    decode: Operands::new_rotation_direction_size_mode_register,         disassemble: disassemble_roxr, },
        IsaEntry { /* isa: Isa::Rte,*/     decode: Operands::new_no_operands,                                   disassemble: disassemble_rte, },
        IsaEntry { /* isa: Isa::Rtr,*/     decode: Operands::new_no_operands,                                   disassemble: disassemble_rtr, },
        IsaEntry { /* isa: Isa::Rts,*/     decode: Operands::new_no_operands,                                   disassemble: disassemble_rts, },
        IsaEntry { /* isa: Isa::Sbcd,*/    decode: Operands::new_register_size_mode_register,                   disassemble: disassemble_sbcd, },
        IsaEntry { /* isa: Isa::Scc,*/     decode: Operands::new_condition_effective_address,                   disassemble: disassemble_scc, },
        IsaEntry { /* isa: Isa::Stop,*/    decode: Operands::new_immediate,                                     disassemble: disassemble_stop, },
        IsaEntry { /* isa: Isa::Sub,*/     decode: Operands::new_register_direction_size_effective_address,     disassemble: disassemble_sub, },
        IsaEntry { /* isa: Isa::Suba,*/    decode: Operands::new_register_size_effective_address,               disassemble: disassemble_suba, },
        IsaEntry { /* isa: Isa::Subi,*/    decode: Operands::new_size_effective_address_immediate,              disassemble: disassemble_subi, },
        IsaEntry { /* isa: Isa::Subq,*/    decode: Operands::new_data_size_effective_address,                   disassemble: disassemble_subq, },
        IsaEntry { /* isa: Isa::Subx,*/    decode: Operands::new_register_size_mode_register,                   disassemble: disassemble_subx, },
        IsaEntry { /* isa: Isa::Swap,*/    decode: Operands::new_register,                                      disassemble: disassemble_swap, },
        IsaEntry { /* isa: Isa::Tas,*/     decode: Operands::new_effective_address,                             disassemble: disassemble_tas, },
        IsaEntry { /* isa: Isa::Trap,*/    decode: Operands::new_vector,                                        disassemble: disassemble_trap, },
        IsaEntry { /* isa: Isa::Trapv,*/   decode: Operands::new_no_operands,                                   disassemble: disassemble_trapv, },
        IsaEntry { /* isa: Isa::Tst,*/     decode: Operands::new_size_effective_address,                        disassemble: disassemble_tst, },
        IsaEntry { /* isa: Isa::Unlk,*/    decode: Operands::new_register,                                      disassemble: disassemble_unlk, },
    ];
}
