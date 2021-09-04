use crate::instruction::Instruction;

use super::{M68000, MemoryAccess};
use super::isa::ISA::Size_;
use super::utils::Bits;

fn disassemble_conditional_test(test: u16) -> &'static str {
    match test {
        0  => "T",
        1  => "F",
        2  => "HI",
        3  => "LS",
        4  => "CC",
        5  => "CS",
        6  => "NE",
        7  => "EQ",
        8  => "VC",
        9  => "VS",
        10 => "PL",
        11 => "MI",
        12 => "GE",
        13 => "LT",
        14 => "GT",
        15 => "LE",
        _ => "Unknown",
    }
}

pub(super) fn disassemble_unknown_instruction(inst: &Instruction) -> String {
    format!("Unknown instruction {:04X} as {:#X}", inst.opcode, inst.pc)
}

pub(super) fn disassemble_abcd(inst: &Instruction) -> String {
    let rx = inst.opcode.bits::<9, 11>();
    let ry = inst.opcode.bits::<0, 2>();
    format!("ABCD {}, {}", ry, rx)
}

pub(super) fn disassemble_add(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_adda(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_addi(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_addq(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_addx(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_and(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_andi(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_andiccr(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_andisr(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_asm(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_asr(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_bcc(inst: &Instruction) -> String {
    format!("B{}", disassemble_conditional_test(inst.opcode.bits::<8, 11>()))
}

pub(super) fn disassemble_bchg(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_bclr(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_bra(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_bset(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_bsr(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_btst(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_chk(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_clr(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_cmp(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_cmpa(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_cmpi(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_cmpm(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_dbcc(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_divs(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_divu(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_eor(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_eori(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_eoriccr(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_eorisr(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_exg(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_ext(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_illegal(inst: &Instruction) -> String {
    format!("ILLEGAL")
}

pub(super) fn disassemble_jmp(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_jsr(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_lea(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_link(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_lsm(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_lsr(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_move(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_movea(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_moveccr(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_movefsr(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_movesr(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_moveusp(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_movem(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_movep(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_moveq(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_muls(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_mulu(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_nbcd(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_neg(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_negx(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_nop(inst: &Instruction) -> String {
    format!("NOP")
}

pub(super) fn disassemble_not(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_or(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_ori(inst: &Instruction) -> String {
    format!("ORI")
}

pub(super) fn disassemble_oriccr(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_orisr(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_pea(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_reset(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_rom(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_ror(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_roxm(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_roxr(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_rte(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_rtr(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_rts(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_sbcd(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_scc(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_stop(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_sub(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_suba(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_subi(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_subq(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_subx(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_swap(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_tas(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_trap(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_trapv(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_tst(inst: &Instruction) -> String {
    format!("")
}

pub(super) fn disassemble_unlk(inst: &Instruction) -> String {
    format!("")
}

pub(super) const DISASSEMBLE: [fn(&Instruction) -> String; Size_ as usize] = [
    disassemble_unknown_instruction,
    disassemble_abcd,
    disassemble_add,
    disassemble_adda,
    disassemble_addi,
    disassemble_addq,
    disassemble_addx,
    disassemble_and,
    disassemble_andi,
    disassemble_andiccr,
    disassemble_andisr,
    disassemble_asm,
    disassemble_asr,
    disassemble_bcc,
    disassemble_bchg,
    disassemble_bclr,
    disassemble_bra,
    disassemble_bset,
    disassemble_bsr,
    disassemble_btst,
    disassemble_chk,
    disassemble_clr,
    disassemble_cmp,
    disassemble_cmpa,
    disassemble_cmpi,
    disassemble_cmpm,
    disassemble_dbcc,
    disassemble_divs,
    disassemble_divu,
    disassemble_eor,
    disassemble_eori,
    disassemble_eoriccr,
    disassemble_eorisr,
    disassemble_exg,
    disassemble_ext,
    disassemble_illegal,
    disassemble_jmp,
    disassemble_jsr,
    disassemble_lea,
    disassemble_link,
    disassemble_lsm,
    disassemble_lsr,
    disassemble_move,
    disassemble_movea,
    disassemble_moveccr,
    disassemble_movefsr,
    disassemble_movesr,
    disassemble_moveusp,
    disassemble_movem,
    disassemble_movep,
    disassemble_moveq,
    disassemble_muls,
    disassemble_mulu,
    disassemble_nbcd,
    disassemble_neg,
    disassemble_negx,
    disassemble_nop,
    disassemble_not,
    disassemble_or,
    disassemble_ori,
    disassemble_oriccr,
    disassemble_orisr,
    disassemble_pea,
    disassemble_reset,
    disassemble_rom,
    disassemble_ror,
    disassemble_roxm,
    disassemble_roxr,
    disassemble_rte,
    disassemble_rtr,
    disassemble_rts,
    disassemble_sbcd,
    disassemble_scc,
    disassemble_stop,
    disassemble_sub,
    disassemble_suba,
    disassemble_subi,
    disassemble_subq,
    disassemble_subx,
    disassemble_swap,
    disassemble_tas,
    disassemble_trap,
    disassemble_trapv,
    disassemble_tst,
    disassemble_unlk,
];
