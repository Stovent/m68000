use super::{M68000, MemoryAccess};
#[cfg(debug_assertions)]
use super::disassembler::*;
use super::instruction::Instruction;
use super::memory_access::U16Iter;
use super::instruction::*;

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
    Size_,
}

#[derive(Clone, Copy)]
pub struct IsaEntry<M: MemoryAccess> {
    // /// The ISA value.
    // pub isa: Isa,
    /// Function used to decode the instruction.
    pub decode: fn(u16, &mut dyn U16Iter) -> (Operands, usize),
    /// Function used to executing the instruction.
    pub execute: fn(&mut M68000, &mut M, &mut Instruction) -> usize,
    /// Function used to diassemble the instruction.
    #[cfg(debug_assertions)]
    pub disassemble: fn(&mut Instruction) -> String,
}

impl<M: MemoryAccess> IsaEntry<M> {
    pub const ISA_ENTRY: [IsaEntry<M>; Isa::Size_ as usize] = [
        IsaEntry { /* isa: Isa::Unknown,*/ decode: no_operands,                                   execute: M68000::unknown_instruction, #[cfg(debug_assertions)] disassemble: disassemble_unknown_instruction, },
        IsaEntry { /* isa: Isa::Abcd,*/    decode: register_size_mode_register,                   execute: M68000::abcd,                #[cfg(debug_assertions)] disassemble: disassemble_abcd, },
        IsaEntry { /* isa: Isa::Add,*/     decode: register_direction_size_effective_address,     execute: M68000::add,                 #[cfg(debug_assertions)] disassemble: disassemble_add, },
        IsaEntry { /* isa: Isa::Adda,*/    decode: register_size_effective_address,               execute: M68000::adda,                #[cfg(debug_assertions)] disassemble: disassemble_adda, },
        IsaEntry { /* isa: Isa::Addi,*/    decode: size_effective_address_immediate,              execute: M68000::addi,                #[cfg(debug_assertions)] disassemble: disassemble_addi, },
        IsaEntry { /* isa: Isa::Addq,*/    decode: data_size_effective_address,                   execute: M68000::addq,                #[cfg(debug_assertions)] disassemble: disassemble_addq, },
        IsaEntry { /* isa: Isa::Addx,*/    decode: register_size_mode_register,                   execute: M68000::addx,                #[cfg(debug_assertions)] disassemble: disassemble_addx, },
        IsaEntry { /* isa: Isa::And,*/     decode: register_direction_size_effective_address,     execute: M68000::and,                 #[cfg(debug_assertions)] disassemble: disassemble_and, },
        IsaEntry { /* isa: Isa::Andi,*/    decode: size_effective_address_immediate,              execute: M68000::andi,                #[cfg(debug_assertions)] disassemble: disassemble_andi, },
        IsaEntry { /* isa: Isa::Andiccr,*/ decode: immediate,                                     execute: M68000::andiccr,             #[cfg(debug_assertions)] disassemble: disassemble_andiccr },
        IsaEntry { /* isa: Isa::Andisr,*/  decode: immediate,                                     execute: M68000::andisr,              #[cfg(debug_assertions)] disassemble: disassemble_andisr, },
        IsaEntry { /* isa: Isa::Asm,*/     decode: direction_effective_address,                   execute: M68000::asm,                 #[cfg(debug_assertions)] disassemble: disassemble_asm, },
        IsaEntry { /* isa: Isa::Asr,*/     decode: rotation_direction_size_mode_register,         execute: M68000::asr,                 #[cfg(debug_assertions)] disassemble: disassemble_asr, },
        IsaEntry { /* isa: Isa::Bcc,*/     decode: condition_displacement,                        execute: M68000::bcc,                 #[cfg(debug_assertions)] disassemble: disassemble_bcc, },
        IsaEntry { /* isa: Isa::Bchg,*/    decode: effective_address_count,                       execute: M68000::bchg,                #[cfg(debug_assertions)] disassemble: disassemble_bchg, },
        IsaEntry { /* isa: Isa::Bclr,*/    decode: effective_address_count,                       execute: M68000::bclr,                #[cfg(debug_assertions)] disassemble: disassemble_bclr, },
        IsaEntry { /* isa: Isa::Bra,*/     decode: displacement,                                  execute: M68000::bra,                 #[cfg(debug_assertions)] disassemble: disassemble_bra, },
        IsaEntry { /* isa: Isa::Bset,*/    decode: effective_address_count,                       execute: M68000::bset,                #[cfg(debug_assertions)] disassemble: disassemble_bset, },
        IsaEntry { /* isa: Isa::Bsr,*/     decode: displacement,                                  execute: M68000::bsr,                 #[cfg(debug_assertions)] disassemble: disassemble_bsr, },
        IsaEntry { /* isa: Isa::Btst,*/    decode: effective_address_count,                       execute: M68000::btst,                #[cfg(debug_assertions)] disassemble: disassemble_btst, },
        IsaEntry { /* isa: Isa::Chk,*/     decode: register_effective_address,                    execute: M68000::chk,                 #[cfg(debug_assertions)] disassemble: disassemble_chk, },
        IsaEntry { /* isa: Isa::Clr,*/     decode: size_effective_address,                        execute: M68000::clr,                 #[cfg(debug_assertions)] disassemble: disassemble_clr, },
        IsaEntry { /* isa: Isa::Cmp,*/     decode: register_direction_size_effective_address,     execute: M68000::cmp,                 #[cfg(debug_assertions)] disassemble: disassemble_cmp, },
        IsaEntry { /* isa: Isa::Cmpa,*/    decode: register_size_effective_address,               execute: M68000::cmpa,                #[cfg(debug_assertions)] disassemble: disassemble_cmpa, },
        IsaEntry { /* isa: Isa::Cmpi,*/    decode: size_effective_address_immediate,              execute: M68000::cmpi,                #[cfg(debug_assertions)] disassemble: disassemble_cmpi, },
        IsaEntry { /* isa: Isa::Cmpm,*/    decode: register_size_register,                        execute: M68000::cmpm,                #[cfg(debug_assertions)] disassemble: disassemble_cmpm, },
        IsaEntry { /* isa: Isa::Dbcc,*/    decode: condition_register_displacement,               execute: M68000::dbcc,                #[cfg(debug_assertions)] disassemble: disassemble_dbcc, },
        IsaEntry { /* isa: Isa::Divs,*/    decode: register_effective_address,                    execute: M68000::divs,                #[cfg(debug_assertions)] disassemble: disassemble_divs, },
        IsaEntry { /* isa: Isa::Divu,*/    decode: register_effective_address,                    execute: M68000::divu,                #[cfg(debug_assertions)] disassemble: disassemble_divu, },
        IsaEntry { /* isa: Isa::Eor,*/     decode: register_direction_size_effective_address,     execute: M68000::eor,                 #[cfg(debug_assertions)] disassemble: disassemble_eor, },
        IsaEntry { /* isa: Isa::Eori,*/    decode: size_effective_address_immediate,              execute: M68000::eori,                #[cfg(debug_assertions)] disassemble: disassemble_eori, },
        IsaEntry { /* isa: Isa::Eoriccr,*/ decode: immediate,                                     execute: M68000::eoriccr,             #[cfg(debug_assertions)] disassemble: disassemble_eoriccr, },
        IsaEntry { /* isa: Isa::Eorisr,*/  decode: immediate,                                     execute: M68000::eorisr,              #[cfg(debug_assertions)] disassemble: disassemble_eorisr, },
        IsaEntry { /* isa: Isa::Exg,*/     decode: register_opmode_register,                      execute: M68000::exg,                 #[cfg(debug_assertions)] disassemble: disassemble_exg, },
        IsaEntry { /* isa: Isa::Ext,*/     decode: opmode_register,                               execute: M68000::ext,                 #[cfg(debug_assertions)] disassemble: disassemble_ext, },
        IsaEntry { /* isa: Isa::Illegal,*/ decode: no_operands,                                   execute: M68000::illegal,             #[cfg(debug_assertions)] disassemble: disassemble_illegal, },
        IsaEntry { /* isa: Isa::Jmp,*/     decode: effective_address,                             execute: M68000::jmp,                 #[cfg(debug_assertions)] disassemble: disassemble_jmp, },
        IsaEntry { /* isa: Isa::Jsr,*/     decode: effective_address,                             execute: M68000::jsr,                 #[cfg(debug_assertions)] disassemble: disassemble_jsr, },
        IsaEntry { /* isa: Isa::Lea,*/     decode: register_effective_address,                    execute: M68000::lea,                 #[cfg(debug_assertions)] disassemble: disassemble_lea, },
        IsaEntry { /* isa: Isa::Link,*/    decode: register_displacement,                         execute: M68000::link,                #[cfg(debug_assertions)] disassemble: disassemble_link, },
        IsaEntry { /* isa: Isa::Lsm,*/     decode: direction_effective_address,                   execute: M68000::lsm,                 #[cfg(debug_assertions)] disassemble: disassemble_lsm, },
        IsaEntry { /* isa: Isa::Lsr,*/     decode: rotation_direction_size_mode_register,         execute: M68000::lsr,                 #[cfg(debug_assertions)] disassemble: disassemble_lsr, },
        IsaEntry { /* isa: Isa::Move,*/    decode: size_effective_address_effective_address,      execute: M68000::r#move,              #[cfg(debug_assertions)] disassemble: disassemble_move, },
        IsaEntry { /* isa: Isa::Movea,*/   decode: size_register_effective_address,               execute: M68000::movea,               #[cfg(debug_assertions)] disassemble: disassemble_movea, },
        IsaEntry { /* isa: Isa::Moveccr,*/ decode: effective_address,                             execute: M68000::moveccr,             #[cfg(debug_assertions)] disassemble: disassemble_moveccr, },
        IsaEntry { /* isa: Isa::Movefsr,*/ decode: effective_address,                             execute: M68000::movefsr,             #[cfg(debug_assertions)] disassemble: disassemble_movefsr, },
        IsaEntry { /* isa: Isa::Movesr,*/  decode: effective_address,                             execute: M68000::movesr,              #[cfg(debug_assertions)] disassemble: disassemble_movesr, },
        IsaEntry { /* isa: Isa::Moveusp,*/ decode: direction_register,                            execute: M68000::moveusp,             #[cfg(debug_assertions)] disassemble: disassemble_moveusp, },
        IsaEntry { /* isa: Isa::Movem,*/   decode: direction_size_effective_address_list,         execute: M68000::movem,               #[cfg(debug_assertions)] disassemble: disassemble_movem, },
        IsaEntry { /* isa: Isa::Movep,*/   decode: register_direction_size_register_displacement, execute: M68000::movep,               #[cfg(debug_assertions)] disassemble: disassemble_movep, },
        IsaEntry { /* isa: Isa::Moveq,*/   decode: register_data,                                 execute: M68000::moveq,               #[cfg(debug_assertions)] disassemble: disassemble_moveq, },
        IsaEntry { /* isa: Isa::Muls,*/    decode: register_effective_address,                    execute: M68000::muls,                #[cfg(debug_assertions)] disassemble: disassemble_muls, },
        IsaEntry { /* isa: Isa::Mulu,*/    decode: register_effective_address,                    execute: M68000::mulu,                #[cfg(debug_assertions)] disassemble: disassemble_mulu, },
        IsaEntry { /* isa: Isa::Nbcd,*/    decode: effective_address,                             execute: M68000::nbcd,                #[cfg(debug_assertions)] disassemble: disassemble_nbcd, },
        IsaEntry { /* isa: Isa::Neg,*/     decode: size_effective_address,                        execute: M68000::neg,                 #[cfg(debug_assertions)] disassemble: disassemble_neg, },
        IsaEntry { /* isa: Isa::Negx,*/    decode: size_effective_address,                        execute: M68000::negx,                #[cfg(debug_assertions)] disassemble: disassemble_negx, },
        IsaEntry { /* isa: Isa::Nop,*/     decode: no_operands,                                   execute: M68000::nop,                 #[cfg(debug_assertions)] disassemble: disassemble_nop, },
        IsaEntry { /* isa: Isa::Not,*/     decode: size_effective_address,                        execute: M68000::not,                 #[cfg(debug_assertions)] disassemble: disassemble_not, },
        IsaEntry { /* isa: Isa::Or,*/      decode: register_direction_size_effective_address,     execute: M68000::or,                  #[cfg(debug_assertions)] disassemble: disassemble_or, },
        IsaEntry { /* isa: Isa::Ori,*/     decode: size_effective_address_immediate,              execute: M68000::ori,                 #[cfg(debug_assertions)] disassemble: disassemble_ori, },
        IsaEntry { /* isa: Isa::Oriccr,*/  decode: immediate,                                     execute: M68000::oriccr,              #[cfg(debug_assertions)] disassemble: disassemble_oriccr, },
        IsaEntry { /* isa: Isa::Orisr,*/   decode: immediate,                                     execute: M68000::orisr,               #[cfg(debug_assertions)] disassemble: disassemble_orisr, },
        IsaEntry { /* isa: Isa::Pea,*/     decode: effective_address,                             execute: M68000::pea,                 #[cfg(debug_assertions)] disassemble: disassemble_pea, },
        IsaEntry { /* isa: Isa::Reset,*/   decode: no_operands,                                   execute: M68000::reset,               #[cfg(debug_assertions)] disassemble: disassemble_reset, },
        IsaEntry { /* isa: Isa::Rom,*/     decode: direction_effective_address,                   execute: M68000::rom,                 #[cfg(debug_assertions)] disassemble: disassemble_rom, },
        IsaEntry { /* isa: Isa::Ror,*/     decode: rotation_direction_size_mode_register,         execute: M68000::ror,                 #[cfg(debug_assertions)] disassemble: disassemble_ror, },
        IsaEntry { /* isa: Isa::Roxm,*/    decode: direction_effective_address,                   execute: M68000::roxm,                #[cfg(debug_assertions)] disassemble: disassemble_roxm, },
        IsaEntry { /* isa: Isa::Roxr,*/    decode: rotation_direction_size_mode_register,         execute: M68000::roxr,                #[cfg(debug_assertions)] disassemble: disassemble_roxr, },
        IsaEntry { /* isa: Isa::Rte,*/     decode: no_operands,                                   execute: M68000::rte,                 #[cfg(debug_assertions)] disassemble: disassemble_rte, },
        IsaEntry { /* isa: Isa::Rtr,*/     decode: no_operands,                                   execute: M68000::rtr,                 #[cfg(debug_assertions)] disassemble: disassemble_rtr, },
        IsaEntry { /* isa: Isa::Rts,*/     decode: no_operands,                                   execute: M68000::rts,                 #[cfg(debug_assertions)] disassemble: disassemble_rts, },
        IsaEntry { /* isa: Isa::Sbcd,*/    decode: register_size_mode_register,                   execute: M68000::sbcd,                #[cfg(debug_assertions)] disassemble: disassemble_sbcd, },
        IsaEntry { /* isa: Isa::Scc,*/     decode: condition_effective_address,                   execute: M68000::scc,                 #[cfg(debug_assertions)] disassemble: disassemble_scc, },
        IsaEntry { /* isa: Isa::Stop,*/    decode: immediate,                                     execute: M68000::stop,                #[cfg(debug_assertions)] disassemble: disassemble_stop, },
        IsaEntry { /* isa: Isa::Sub,*/     decode: register_direction_size_effective_address,     execute: M68000::sub,                 #[cfg(debug_assertions)] disassemble: disassemble_sub, },
        IsaEntry { /* isa: Isa::Suba,*/    decode: register_size_effective_address,               execute: M68000::suba,                #[cfg(debug_assertions)] disassemble: disassemble_suba, },
        IsaEntry { /* isa: Isa::Subi,*/    decode: size_effective_address_immediate,              execute: M68000::subi,                #[cfg(debug_assertions)] disassemble: disassemble_subi, },
        IsaEntry { /* isa: Isa::Subq,*/    decode: data_size_effective_address,                   execute: M68000::subq,                #[cfg(debug_assertions)] disassemble: disassemble_subq, },
        IsaEntry { /* isa: Isa::Subx,*/    decode: register_size_mode_register,                   execute: M68000::subx,                #[cfg(debug_assertions)] disassemble: disassemble_subx, },
        IsaEntry { /* isa: Isa::Swap,*/    decode: register,                                      execute: M68000::swap,                #[cfg(debug_assertions)] disassemble: disassemble_swap, },
        IsaEntry { /* isa: Isa::Tas,*/     decode: effective_address,                             execute: M68000::tas,                 #[cfg(debug_assertions)] disassemble: disassemble_tas, },
        IsaEntry { /* isa: Isa::Trap,*/    decode: vector,                                        execute: M68000::trap,                #[cfg(debug_assertions)] disassemble: disassemble_trap, },
        IsaEntry { /* isa: Isa::Trapv,*/   decode: no_operands,                                   execute: M68000::trapv,               #[cfg(debug_assertions)] disassemble: disassemble_trapv, },
        IsaEntry { /* isa: Isa::Tst,*/     decode: size_effective_address,                        execute: M68000::tst,                 #[cfg(debug_assertions)] disassemble: disassemble_tst, },
        IsaEntry { /* isa: Isa::Unlk,*/    decode: register,                                      execute: M68000::unlk,                #[cfg(debug_assertions)] disassemble: disassemble_unlk, },
    ];
}
