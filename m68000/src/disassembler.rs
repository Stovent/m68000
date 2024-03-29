// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Disassembler module.

use crate::instruction::{Direction, Instruction};
use crate::isa::Isa;
use crate::status_register::disassemble_conditional_test;
use crate::utils::bits;

pub fn disassemble_unknown_instruction(inst: &Instruction) -> String {
    format!("Unknown instruction {:04X} at {:#X}", inst.opcode, inst.pc)
}

pub fn disassemble_abcd(inst: &Instruction) -> String {
    let (rx, _, mode, ry) = inst.operands.register_size_mode_register();
    if mode == Direction::MemoryToMemory {
        format!("ABCD -(A{}), -(A{})", ry, rx)
    } else {
        format!("ABCD D{}, D{}", ry, rx)
    }
}

pub fn disassemble_add(inst: &Instruction) -> String {
    let (r, d, s, am) = inst.operands.register_direction_size_effective_address();
    if d == Direction::DstEa {
        format!("ADD.{} D{}, {}", s, r, am)
    } else {
        format!("ADD.{} {}, D{}", s, am, r)
    }
}

pub fn disassemble_adda(inst: &Instruction) -> String {
    let (r, s, am) = inst.operands.register_size_effective_address();
    format!("ADDA.{} {}, A{}", s, am, r)
}

pub fn disassemble_addi(inst: &Instruction) -> String {
    let (s, am, imm) = inst.operands.size_effective_address_immediate();
    format!("ADDI.{} #{}, {}", s, imm, am)
}

pub fn disassemble_addq(inst: &Instruction) -> String {
    let (d, s, am) = inst.operands.data_size_effective_address();
    let d = if d == 0 { 8 } else { d };
    format!("ADDQ.{} #{}, {}", s, d, am)
}

pub fn disassemble_addx(inst: &Instruction) -> String {
    let (rx, s, mode, ry) = inst.operands.register_size_mode_register();
    if mode == Direction::MemoryToMemory {
        format!("ADDX.{} -(A{}), -(A{})", s, ry, rx)
    } else {
        format!("ADDX.{} D{}, D{}", s, ry, rx)
    }
}

pub fn disassemble_and(inst: &Instruction) -> String {
    let (r, d, s, am) = inst.operands.register_direction_size_effective_address();
    if d == Direction::DstEa {
        format!("AND.{} D{}, {}", s, r, am)
    } else {
        format!("AND.{} {}, D{}", s, am, r)
    }
}

pub fn disassemble_andi(inst: &Instruction) -> String {
    let (s, am, imm) = inst.operands.size_effective_address_immediate();
    format!("ANDI.{} #{}, {}", s, imm, am)
}

pub fn disassemble_andiccr(inst: &Instruction) -> String {
    let imm = inst.operands.immediate();
    format!("ANDI {:#X}, CCR", imm)
}

pub fn disassemble_andisr(inst: &Instruction) -> String {
    let imm = inst.operands.immediate();
    format!("ANDI {:#X}, SR", imm)
}

pub fn disassemble_asm(inst: &Instruction) -> String {
    let (d, am) = inst.operands.direction_effective_address();
    format!("AS{} {}", d, am)
}

pub fn disassemble_asr(inst: &Instruction) -> String {
    let (rot, d, s, ir, reg) = inst.operands.rotation_direction_size_mode_register();
    if ir {
        format!("AS{}.{} D{}, D{}", d, s, rot, reg)
    } else {
        let rot = if rot == 0 { 8 } else { rot };
        format!("AS{}.{} #{}, D{}", d, s, rot, reg)
    }
}

pub fn disassemble_bcc(inst: &Instruction) -> String {
    let (cc, disp) = inst.operands.condition_displacement();
    format!("B{} {} <{:#X}>", disassemble_conditional_test(cc), disp, inst.pc.wrapping_add(2).wrapping_add(disp as u32))
}

pub fn disassemble_bchg(inst: &Instruction) -> String {
    let (am, count) = inst.operands.effective_address_count();
    if bits(inst.opcode, 8, 8) != 0 {
        format!("BCHG D{}, {}", count, am)
    } else {
        format!("BCHG #{}, {}", count, am)
    }
}

pub fn disassemble_bclr(inst: &Instruction) -> String {
    let (am, count) = inst.operands.effective_address_count();
    if bits(inst.opcode, 8, 8) != 0 {
        format!("BCLR D{}, {}", count, am)
    } else {
        format!("BCLR #{}, {}", count, am)
    }
}

pub fn disassemble_bra(inst: &Instruction) -> String {
    let disp = inst.operands.displacement();
    format!("BRA {} <{:#X}>", disp, inst.pc.wrapping_add(2).wrapping_add(disp as u32))
}

pub fn disassemble_bset(inst: &Instruction) -> String {
    let (am, count) = inst.operands.effective_address_count();
    if bits(inst.opcode, 8, 8) != 0 {
        format!("BSET D{}, {}", count, am)
    } else {
        format!("BSET #{}, {}", count, am)
    }
}

pub fn disassemble_bsr(inst: &Instruction) -> String {
    let disp = inst.operands.displacement();
    format!("BSR {} <{:#X}>", disp, inst.pc.wrapping_add(2).wrapping_add(disp as u32))
}

pub fn disassemble_btst(inst: &Instruction) -> String {
    let (am, count) = inst.operands.effective_address_count();
    if bits(inst.opcode, 8, 8) != 0 {
        format!("BTST D{}, {}", count, am)
    } else {
        format!("BTST #{}, {}", count, am)
    }
}

pub fn disassemble_chk(inst: &Instruction) -> String {
    let (r, am) = inst.operands.register_effective_address();
    format!("CHK.W {}, D{}", am, r)
}

pub fn disassemble_clr(inst: &Instruction) -> String {
    let (s, am) = inst.operands.size_effective_address();
    format!("CLR.{} {}", s, am)
}

pub fn disassemble_cmp(inst: &Instruction) -> String {
    let (r, _, s, am) = inst.operands.register_direction_size_effective_address();
    format!("CMP.{} {}, D{}", s, am, r)
}

pub fn disassemble_cmpa(inst: &Instruction) -> String {
    let (r, s, am) = inst.operands.register_size_effective_address();
    format!("CMPA.{} {}, A{}", s, am, r)
}

pub fn disassemble_cmpi(inst: &Instruction) -> String {
    let (s, am, imm) = inst.operands.size_effective_address_immediate();
    format!("CMPI.{} #{}, {}", s, imm, am)
}

pub fn disassemble_cmpm(inst: &Instruction) -> String {
    let (rx, s, ry) = inst.operands.register_size_register();
    format!("CMPM.{} (A{})+, (A{})+", s, ry, rx)
}

pub fn disassemble_dbcc(inst: &Instruction) -> String {
    let (cc, r, disp) = inst.operands.condition_register_displacement();
    format!("DB{} D{}, {} <{:#X}>", disassemble_conditional_test(cc), r, disp, inst.pc.wrapping_add(2).wrapping_add(disp as u32))
}

pub fn disassemble_divs(inst: &Instruction) -> String {
    let (r, am) = inst.operands.register_effective_address();
    format!("DIVS.W {}, D{}", am, r)
}

pub fn disassemble_divu(inst: &Instruction) -> String {
    let (r, am) = inst.operands.register_effective_address();
    format!("DIVU.W {}, D{}", am, r)
}

pub fn disassemble_eor(inst: &Instruction) -> String {
    let (r, _, s, am) = inst.operands.register_direction_size_effective_address();
    format!("EOR.{} D{}, {}", s, r, am)
}

pub fn disassemble_eori(inst: &Instruction) -> String {
    let (s, am, imm) = inst.operands.size_effective_address_immediate();
    format!("EORI.{} #{}, {}", s, imm, am)
}

pub fn disassemble_eoriccr(inst: &Instruction) -> String {
    let imm = inst.operands.immediate();
    format!("EORI {:#X}, CCR", imm)
}

pub fn disassemble_eorisr(inst: &Instruction) -> String {
    let imm = inst.operands.immediate();
    format!("EORI {:#X}, SR", imm)
}

pub fn disassemble_exg(inst: &Instruction) -> String {
    let (rx, mode, ry) = inst.operands.register_opmode_register();
    if mode == Direction::ExchangeData {
        format!("EXG D{}, D{}", rx, ry)
    } else if mode == Direction::ExchangeAddress {
        format!("EXG A{}, A{}", rx, ry)
    } else {
        format!("EXG D{}, A{}", rx, ry)
    }
}

pub fn disassemble_ext(inst: &Instruction) -> String {
    let (mode, r) = inst.operands.opmode_register();
    if mode == 0b010 {
        format!("EXT.W D{}", r)
    } else {
        format!("EXT.L D{}", r)
    }
}

pub fn disassemble_illegal(_: &Instruction) -> String {
    "ILLEGAL".to_string()
}

pub fn disassemble_jmp(inst: &Instruction) -> String {
    let am = inst.operands.effective_address();
    format!("JMP {}", am)
}

pub fn disassemble_jsr(inst: &Instruction) -> String {
    let am = inst.operands.effective_address();
    format!("JSR {}", am)
}

pub fn disassemble_lea(inst: &Instruction) -> String {
    let (r, am) = inst.operands.register_effective_address();
    format!("LEA {}, A{}", am, r)
}

pub fn disassemble_link(inst: &Instruction) -> String {
    let (r, disp) = inst.operands.register_displacement();
    format!("LINK.W A{}, #{}", r, disp)
}

pub fn disassemble_lsm(inst: &Instruction) -> String {
    let (d, am) = inst.operands.direction_effective_address();
    format!("LS{} {}", d, am)
}

pub fn disassemble_lsr(inst: &Instruction) -> String {
    let (rot, d, s, ir, reg) = inst.operands.rotation_direction_size_mode_register();
    if ir {
        format!("LS{}.{} D{}, D{}", d, s, rot, reg)
    } else {
        let rot = if rot == 0 { 8 } else { rot };
        format!("LS{}.{} #{}, D{}", d, s, rot, reg)
    }
}

pub fn disassemble_move(inst: &Instruction) -> String {
    let (s, dst, src) = inst.operands.size_effective_address_effective_address();
    format!("MOVE.{} {}, {}", s, src, dst)
}

pub fn disassemble_movea(inst: &Instruction) -> String {
    let (s, r, am) = inst.operands.size_register_effective_address();
    format!("MOVEA.{} {:#X}, A{}", s, am, r)
}

pub fn disassemble_moveccr(inst: &Instruction) -> String {
    let am = inst.operands.effective_address();
    format!("MOVE {:#X}, CCR", am)
}

pub fn disassemble_movefsr(inst: &Instruction) -> String {
    let am = inst.operands.effective_address();
    format!("MOVE SR, {:#X}", am)
}

pub fn disassemble_movesr(inst: &Instruction) -> String {
    let am = inst.operands.effective_address();
    format!("MOVE {:#X}, SR", am)
}

pub fn disassemble_moveusp(inst: &Instruction) -> String {
    let (dir, reg) = inst.operands.direction_register();
    if dir == Direction::UspToRegister {
        format!("MOVE USP, A{}", reg)
    } else {
        format!("MOVE A{}, USP", reg)
    }
}

pub fn disassemble_movem(inst: &Instruction) -> String {
    // TODO: disassemble register list.
    let (d, s, am, list) = inst.operands.direction_size_effective_address_list();
    if d == Direction::MemoryToRegister {
        format!("MOVEM.{} {}, {:#X}", s, am, list)
    } else {
        format!("MOVEM.{} {:#X}, {}", s, list, am)
    }
}

pub fn disassemble_movep(inst: &Instruction) -> String {
    let (dreg, d, s, areg, disp) = inst.operands.register_direction_size_register_displacement();
    if d == Direction::RegisterToMemory {
        format!("MOVEP.{} D{}, ({}, A{})", s, dreg, disp, areg)
    } else {
        format!("MOVEP.{} ({}, A{}), D{}", s, disp, areg, dreg)
    }
}

pub fn disassemble_moveq(inst: &Instruction) -> String {
    let (r, d) = inst.operands.register_data();
    format!("MOVEQ.L #{}, D{}", d, r)
}

pub fn disassemble_muls(inst: &Instruction) -> String {
    let (r, am) = inst.operands.register_effective_address();
    format!("MULS.W {}, D{}", am, r)
}

pub fn disassemble_mulu(inst: &Instruction) -> String {
    let (r, am) = inst.operands.register_effective_address();
    format!("MULU.W {}, D{}", am, r)
}

pub fn disassemble_nbcd(inst: &Instruction) -> String {
    let am = inst.operands.effective_address();
    format!("NBCD {}", am)
}

pub fn disassemble_neg(inst: &Instruction) -> String {
    let (s, am) = inst.operands.size_effective_address();
    format!("NEG.{} {}", s, am)
}

pub fn disassemble_negx(inst: &Instruction) -> String {
    let (s, am) = inst.operands.size_effective_address();
    format!("NEGX.{} {}", s, am)
}

pub fn disassemble_nop(_: &Instruction) -> String {
    "NOP".to_string()
}

pub fn disassemble_not(inst: &Instruction) -> String {
    let (s, am) = inst.operands.size_effective_address();
    format!("NOT.{} {}", s, am)
}

pub fn disassemble_or(inst: &Instruction) -> String {
    let (r, d, s, am) = inst.operands.register_direction_size_effective_address();
    if d == Direction::DstEa {
        format!("OR.{} D{}, {}", s, r, am)
    } else {
        format!("OR.{} {}, D{}", s, am, r)
    }
}

pub fn disassemble_ori(inst: &Instruction) -> String {
    let (s, am, imm) = inst.operands.size_effective_address_immediate();
    format!("ORI.{} #{}, {}", s, imm, am)
}

pub fn disassemble_oriccr(inst: &Instruction) -> String {
    let imm = inst.operands.immediate();
    format!("ORI {:#X}, CCR", imm)
}

pub fn disassemble_orisr(inst: &Instruction) -> String {
    let imm = inst.operands.immediate();
    format!("ORI {:#X}, SR", imm)
}

pub fn disassemble_pea(inst: &Instruction) -> String {
    let am = inst.operands.effective_address();
    format!("PEA {}", am)
}

pub fn disassemble_reset(_: &Instruction) -> String {
    "RESET".to_owned()
}

pub fn disassemble_rom(inst: &Instruction) -> String {
    let (d, am) = inst.operands.direction_effective_address();
    format!("RO{} {}", d, am)
}

pub fn disassemble_ror(inst: &Instruction) -> String {
    let (rot, d, s, ir, reg) = inst.operands.rotation_direction_size_mode_register();
    if ir {
        format!("RO{}.{} D{}, D{}", d, s, rot, reg)
    } else {
        let rot = if rot == 0 { 8 } else { rot };
        format!("RO{}.{} #{}, D{}", d, s, rot, reg)
    }
}

pub fn disassemble_roxm(inst: &Instruction) -> String {
    let (d, am) = inst.operands.direction_effective_address();
    format!("ROX{} {}", d, am)
}

pub fn disassemble_roxr(inst: &Instruction) -> String {
    let (rot, d, s, ir, reg) = inst.operands.rotation_direction_size_mode_register();
    if ir {
        format!("ROX{}.{} D{}, D{}", d, s, rot, reg)
    } else {
        let rot = if rot == 0 { 8 } else { rot };
        format!("ROX{}.{} #{}, D{}", d, s, rot, reg)
    }
}

pub fn disassemble_rte(_: &Instruction) -> String {
    "RTE".to_string()
}

pub fn disassemble_rtr(_: &Instruction) -> String {
    "RTR".to_string()
}

pub fn disassemble_rts(_: &Instruction) -> String {
    "RTS".to_string()
}

pub fn disassemble_sbcd(inst: &Instruction) -> String {
    let (ry, _, mode, rx) = inst.operands.register_size_mode_register();
    if mode == Direction::MemoryToMemory {
        format!("SBCD -(A{}), -(A{})", rx, ry)
    } else {
        format!("SBCD D{}, D{}", rx, ry)
    }
}

pub fn disassemble_scc(inst: &Instruction) -> String {
    let (cc, am) = inst.operands.condition_effective_address();
    format!("S{} {}", disassemble_conditional_test(cc), am)
}

pub fn disassemble_stop(inst: &Instruction) -> String {
    let imm = inst.operands.immediate();
    format!("STOP #{:#X}", imm)
}

pub fn disassemble_sub(inst: &Instruction) -> String {
    let (r, d, s, am) = inst.operands.register_direction_size_effective_address();
    if d == Direction::DstEa {
        format!("SUB.{} D{}, {}", s, r, am)
    } else {
        format!("SUB.{} {}, D{}", s, am, r)
    }
}

pub fn disassemble_suba(inst: &Instruction) -> String {
    let (r, s, am) = inst.operands.register_size_effective_address();
    format!("SUBA.{} {}, A{}", s, am, r)
}

pub fn disassemble_subi(inst: &Instruction) -> String {
    let (s, am, imm) = inst.operands.size_effective_address_immediate();
    format!("SUBI.{} #{}, {}", s, imm, am)
}

pub fn disassemble_subq(inst: &Instruction) -> String {
    let (d, s, am) = inst.operands.data_size_effective_address();
    let d = if d == 0 { 8 } else { d };
    format!("SUBQ.{} #{}, {}", s, d, am)
}

pub fn disassemble_subx(inst: &Instruction) -> String {
    let (ry, s, mode, rx) = inst.operands.register_size_mode_register();
    if mode == Direction::MemoryToMemory {
        format!("SUBX.{} -(A{}), -(A{})", s, rx, ry)
    } else {
        format!("SUBX.{} D{}, D{}", s, rx, ry)
    }
}

pub fn disassemble_swap(inst: &Instruction) -> String {
    let r = inst.operands.register();
    format!("SWAP D{}", r)
}

pub fn disassemble_tas(inst: &Instruction) -> String {
    let am = inst.operands.effective_address();
    format!("TAS {}", am)
}

pub fn disassemble_trap(inst: &Instruction) -> String {
    let v = inst.operands.vector();
    format!("TRAP #{}", v)
}

pub fn disassemble_trapv(_: &Instruction) -> String {
    "TRAPV".to_string()
}

pub fn disassemble_tst(inst: &Instruction) -> String {
    let (s, am) = inst.operands.size_effective_address();
    format!("TST.{} {}", s, am)
}

pub fn disassemble_unlk(inst: &Instruction) -> String {
    let r = inst.operands.register();
    format!("UNLK A{}", r)
}

/// Disassembler function Look-Up Table.
///
/// # Usage
///
/// ```
/// use m68000::decoder::DECODER;
/// use m68000::disassembler::DLUT;
/// use m68000::instruction::Instruction;
/// use m68000::memory_access::MemoryAccess;
///
/// let mut data: Vec<u8> = Vec::new();
/// data.resize(4, 0); // Load the binary in data.
/// let mut iter = data.iter_u16(0);
/// let inst = Instruction::from_memory(&mut iter).unwrap();
/// let disassemble = DLUT[DECODER[inst.opcode as usize] as usize];
/// println!("{:#X} {}", inst.pc, disassemble(&inst));
/// ```
pub const DLUT: [fn(&Instruction) -> String; Isa::_Size as usize] = [
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
