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
    pub decode: fn(u16, &mut MemoryIter) -> (Operands, usize),
    /// Function used to diassemble the instruction.
    pub disassemble: fn(&Instruction) -> String,
}

impl IsaEntry {
    /// The array that maps instructions to their [IsaEntry] entry. Index it using the [Isa] enum.
    pub const ISA_ENTRY: [IsaEntry; Isa::_Size as usize] = [
        IsaEntry { /* isa: Isa::Unknown,*/ decode: no_operands,                                   disassemble: disassemble_unknown_instruction, },
        IsaEntry { /* isa: Isa::Abcd,*/    decode: register_size_mode_register,                   disassemble: disassemble_abcd, },
        IsaEntry { /* isa: Isa::Add,*/     decode: register_direction_size_effective_address,     disassemble: disassemble_add, },
        IsaEntry { /* isa: Isa::Adda,*/    decode: register_size_effective_address,               disassemble: disassemble_adda, },
        IsaEntry { /* isa: Isa::Addi,*/    decode: size_effective_address_immediate,              disassemble: disassemble_addi, },
        IsaEntry { /* isa: Isa::Addq,*/    decode: data_size_effective_address,                   disassemble: disassemble_addq, },
        IsaEntry { /* isa: Isa::Addx,*/    decode: register_size_mode_register,                   disassemble: disassemble_addx, },
        IsaEntry { /* isa: Isa::And,*/     decode: register_direction_size_effective_address,     disassemble: disassemble_and, },
        IsaEntry { /* isa: Isa::Andi,*/    decode: size_effective_address_immediate,              disassemble: disassemble_andi, },
        IsaEntry { /* isa: Isa::Andiccr,*/ decode: immediate,                                     disassemble: disassemble_andiccr },
        IsaEntry { /* isa: Isa::Andisr,*/  decode: immediate,                                     disassemble: disassemble_andisr, },
        IsaEntry { /* isa: Isa::Asm,*/     decode: direction_effective_address,                   disassemble: disassemble_asm, },
        IsaEntry { /* isa: Isa::Asr,*/     decode: rotation_direction_size_mode_register,         disassemble: disassemble_asr, },
        IsaEntry { /* isa: Isa::Bcc,*/     decode: condition_displacement,                        disassemble: disassemble_bcc, },
        IsaEntry { /* isa: Isa::Bchg,*/    decode: effective_address_count,                       disassemble: disassemble_bchg, },
        IsaEntry { /* isa: Isa::Bclr,*/    decode: effective_address_count,                       disassemble: disassemble_bclr, },
        IsaEntry { /* isa: Isa::Bra,*/     decode: displacement,                                  disassemble: disassemble_bra, },
        IsaEntry { /* isa: Isa::Bset,*/    decode: effective_address_count,                       disassemble: disassemble_bset, },
        IsaEntry { /* isa: Isa::Bsr,*/     decode: displacement,                                  disassemble: disassemble_bsr, },
        IsaEntry { /* isa: Isa::Btst,*/    decode: effective_address_count,                       disassemble: disassemble_btst, },
        IsaEntry { /* isa: Isa::Chk,*/     decode: register_effective_address,                    disassemble: disassemble_chk, },
        IsaEntry { /* isa: Isa::Clr,*/     decode: size_effective_address,                        disassemble: disassemble_clr, },
        IsaEntry { /* isa: Isa::Cmp,*/     decode: register_direction_size_effective_address,     disassemble: disassemble_cmp, },
        IsaEntry { /* isa: Isa::Cmpa,*/    decode: register_size_effective_address,               disassemble: disassemble_cmpa, },
        IsaEntry { /* isa: Isa::Cmpi,*/    decode: size_effective_address_immediate,              disassemble: disassemble_cmpi, },
        IsaEntry { /* isa: Isa::Cmpm,*/    decode: register_size_register,                        disassemble: disassemble_cmpm, },
        IsaEntry { /* isa: Isa::Dbcc,*/    decode: condition_register_displacement,               disassemble: disassemble_dbcc, },
        IsaEntry { /* isa: Isa::Divs,*/    decode: register_effective_address,                    disassemble: disassemble_divs, },
        IsaEntry { /* isa: Isa::Divu,*/    decode: register_effective_address,                    disassemble: disassemble_divu, },
        IsaEntry { /* isa: Isa::Eor,*/     decode: register_direction_size_effective_address,     disassemble: disassemble_eor, },
        IsaEntry { /* isa: Isa::Eori,*/    decode: size_effective_address_immediate,              disassemble: disassemble_eori, },
        IsaEntry { /* isa: Isa::Eoriccr,*/ decode: immediate,                                     disassemble: disassemble_eoriccr, },
        IsaEntry { /* isa: Isa::Eorisr,*/  decode: immediate,                                     disassemble: disassemble_eorisr, },
        IsaEntry { /* isa: Isa::Exg,*/     decode: register_opmode_register,                      disassemble: disassemble_exg, },
        IsaEntry { /* isa: Isa::Ext,*/     decode: opmode_register,                               disassemble: disassemble_ext, },
        IsaEntry { /* isa: Isa::Illegal,*/ decode: no_operands,                                   disassemble: disassemble_illegal, },
        IsaEntry { /* isa: Isa::Jmp,*/     decode: effective_address,                             disassemble: disassemble_jmp, },
        IsaEntry { /* isa: Isa::Jsr,*/     decode: effective_address,                             disassemble: disassemble_jsr, },
        IsaEntry { /* isa: Isa::Lea,*/     decode: register_effective_address,                    disassemble: disassemble_lea, },
        IsaEntry { /* isa: Isa::Link,*/    decode: register_displacement,                         disassemble: disassemble_link, },
        IsaEntry { /* isa: Isa::Lsm,*/     decode: direction_effective_address,                   disassemble: disassemble_lsm, },
        IsaEntry { /* isa: Isa::Lsr,*/     decode: rotation_direction_size_mode_register,         disassemble: disassemble_lsr, },
        IsaEntry { /* isa: Isa::Move,*/    decode: size_effective_address_effective_address,      disassemble: disassemble_move, },
        IsaEntry { /* isa: Isa::Movea,*/   decode: size_register_effective_address,               disassemble: disassemble_movea, },
        IsaEntry { /* isa: Isa::Moveccr,*/ decode: effective_address,                             disassemble: disassemble_moveccr, },
        IsaEntry { /* isa: Isa::Movefsr,*/ decode: effective_address,                             disassemble: disassemble_movefsr, },
        IsaEntry { /* isa: Isa::Movesr,*/  decode: effective_address,                             disassemble: disassemble_movesr, },
        IsaEntry { /* isa: Isa::Moveusp,*/ decode: direction_register,                            disassemble: disassemble_moveusp, },
        IsaEntry { /* isa: Isa::Movem,*/   decode: direction_size_effective_address_list,         disassemble: disassemble_movem, },
        IsaEntry { /* isa: Isa::Movep,*/   decode: register_direction_size_register_displacement, disassemble: disassemble_movep, },
        IsaEntry { /* isa: Isa::Moveq,*/   decode: register_data,                                 disassemble: disassemble_moveq, },
        IsaEntry { /* isa: Isa::Muls,*/    decode: register_effective_address,                    disassemble: disassemble_muls, },
        IsaEntry { /* isa: Isa::Mulu,*/    decode: register_effective_address,                    disassemble: disassemble_mulu, },
        IsaEntry { /* isa: Isa::Nbcd,*/    decode: effective_address,                             disassemble: disassemble_nbcd, },
        IsaEntry { /* isa: Isa::Neg,*/     decode: size_effective_address,                        disassemble: disassemble_neg, },
        IsaEntry { /* isa: Isa::Negx,*/    decode: size_effective_address,                        disassemble: disassemble_negx, },
        IsaEntry { /* isa: Isa::Nop,*/     decode: no_operands,                                   disassemble: disassemble_nop, },
        IsaEntry { /* isa: Isa::Not,*/     decode: size_effective_address,                        disassemble: disassemble_not, },
        IsaEntry { /* isa: Isa::Or,*/      decode: register_direction_size_effective_address,     disassemble: disassemble_or, },
        IsaEntry { /* isa: Isa::Ori,*/     decode: size_effective_address_immediate,              disassemble: disassemble_ori, },
        IsaEntry { /* isa: Isa::Oriccr,*/  decode: immediate,                                     disassemble: disassemble_oriccr, },
        IsaEntry { /* isa: Isa::Orisr,*/   decode: immediate,                                     disassemble: disassemble_orisr, },
        IsaEntry { /* isa: Isa::Pea,*/     decode: effective_address,                             disassemble: disassemble_pea, },
        IsaEntry { /* isa: Isa::Reset,*/   decode: no_operands,                                   disassemble: disassemble_reset, },
        IsaEntry { /* isa: Isa::Rom,*/     decode: direction_effective_address,                   disassemble: disassemble_rom, },
        IsaEntry { /* isa: Isa::Ror,*/     decode: rotation_direction_size_mode_register,         disassemble: disassemble_ror, },
        IsaEntry { /* isa: Isa::Roxm,*/    decode: direction_effective_address,                   disassemble: disassemble_roxm, },
        IsaEntry { /* isa: Isa::Roxr,*/    decode: rotation_direction_size_mode_register,         disassemble: disassemble_roxr, },
        IsaEntry { /* isa: Isa::Rte,*/     decode: no_operands,                                   disassemble: disassemble_rte, },
        IsaEntry { /* isa: Isa::Rtr,*/     decode: no_operands,                                   disassemble: disassemble_rtr, },
        IsaEntry { /* isa: Isa::Rts,*/     decode: no_operands,                                   disassemble: disassemble_rts, },
        IsaEntry { /* isa: Isa::Sbcd,*/    decode: register_size_mode_register,                   disassemble: disassemble_sbcd, },
        IsaEntry { /* isa: Isa::Scc,*/     decode: condition_effective_address,                   disassemble: disassemble_scc, },
        IsaEntry { /* isa: Isa::Stop,*/    decode: immediate,                                     disassemble: disassemble_stop, },
        IsaEntry { /* isa: Isa::Sub,*/     decode: register_direction_size_effective_address,     disassemble: disassemble_sub, },
        IsaEntry { /* isa: Isa::Suba,*/    decode: register_size_effective_address,               disassemble: disassemble_suba, },
        IsaEntry { /* isa: Isa::Subi,*/    decode: size_effective_address_immediate,              disassemble: disassemble_subi, },
        IsaEntry { /* isa: Isa::Subq,*/    decode: data_size_effective_address,                   disassemble: disassemble_subq, },
        IsaEntry { /* isa: Isa::Subx,*/    decode: register_size_mode_register,                   disassemble: disassemble_subx, },
        IsaEntry { /* isa: Isa::Swap,*/    decode: register,                                      disassemble: disassemble_swap, },
        IsaEntry { /* isa: Isa::Tas,*/     decode: effective_address,                             disassemble: disassemble_tas, },
        IsaEntry { /* isa: Isa::Trap,*/    decode: vector,                                        disassemble: disassemble_trap, },
        IsaEntry { /* isa: Isa::Trapv,*/   decode: no_operands,                                   disassemble: disassemble_trapv, },
        IsaEntry { /* isa: Isa::Tst,*/     decode: size_effective_address,                        disassemble: disassemble_tst, },
        IsaEntry { /* isa: Isa::Unlk,*/    decode: register,                                      disassemble: disassemble_unlk, },
    ];
}
