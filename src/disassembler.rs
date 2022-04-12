//! Disassembler module.

use crate::instruction::{Direction, Instruction};
use crate::status_register::disassemble_conditional_test;
use crate::utils::bits;

pub fn disassemble_unknown_instruction(inst: &mut Instruction) -> String {
    format!("Unknown instruction {:04X} at {:#X}", inst.opcode, inst.pc)
}

pub fn disassemble_abcd(inst: &mut Instruction) -> String {
    let (rx, _, mode, ry) = inst.operands.register_size_mode_register();
    if mode == Direction::MemoryToMemory {
        format!("ABCD -(A{}), -(A{})", ry, rx)
    } else {
        format!("ABCD D{}, D{}", ry, rx)
    }
}

pub fn disassemble_add(inst: &mut Instruction) -> String {
    let (r, d, s, ea) = inst.operands.register_direction_size_effective_address();
    if d == Direction::DstEa {
        format!("ADD.{} D{}, {}", s, r, ea.mode)
    } else {
        format!("ADD.{} {}, D{}", s, ea.mode, r)
    }
}

pub fn disassemble_adda(inst: &mut Instruction) -> String {
    let (r, s, ea) = inst.operands.register_size_effective_address();
    format!("ADDA.{} {}, A{}", s, ea.mode, r)
}

pub fn disassemble_addi(inst: &mut Instruction) -> String {
    let (s, ea, imm) = inst.operands.size_effective_address_immediate();
    format!("ADDI.{} #{}, {}", s, imm, ea.mode)
}

pub fn disassemble_addq(inst: &mut Instruction) -> String {
    let (d, s, ea) = inst.operands.data_size_effective_address();
    format!("ADDQ.{} #{}, {}", s, d, ea.mode)
}

pub fn disassemble_addx(inst: &mut Instruction) -> String {
    let (rx, s, mode, ry) = inst.operands.register_size_mode_register();
    if mode == Direction::MemoryToMemory {
        format!("ADDX.{} -(A{}), -(A{})", s, ry, rx)
    } else {
        format!("ADDX.{} D{}, D{}", s, ry, rx)
    }
}

pub fn disassemble_and(inst: &mut Instruction) -> String {
    let (r, d, s, ea) = inst.operands.register_direction_size_effective_address();
    if d == Direction::DstEa {
        format!("AND.{} D{}, {}", s, r, ea.mode)
    } else {
        format!("AND.{} {}, D{}", s, ea.mode, r)
    }
}

pub fn disassemble_andi(inst: &mut Instruction) -> String {
    let (s, ea, imm) = inst.operands.size_effective_address_immediate();
    format!("ANDI.{} #{}, {}", s, imm, ea.mode)
}

pub fn disassemble_andiccr(inst: &mut Instruction) -> String {
    let imm = inst.operands.immediate();
    format!("ANDI {:#X}, CCR", imm)
}

pub fn disassemble_andisr(inst: &mut Instruction) -> String {
    let imm = inst.operands.immediate();
    format!("ANDI {:#X}, SR", imm)
}

pub fn disassemble_asm(inst: &mut Instruction) -> String {
    let (d, ea) = inst.operands.direction_effective_address();
    format!("AS{} {}", d, ea.mode)
}

pub fn disassemble_asr(inst: &mut Instruction) -> String {
    let (rot, d, s, mode, reg) = inst.operands.rotation_direction_size_mode_register();
    if mode == 1 {
        format!("AS{}.{} D{}, D{}", d, s, rot, reg)
    } else {
        format!("AS{}.{} #{}, D{}", d, s, rot, reg)
    }
}

pub fn disassemble_bcc(inst: &mut Instruction) -> String {
    let (cc, disp) = inst.operands.condition_displacement();
    format!("B{} {} <{:#X}>", disassemble_conditional_test(cc), disp, inst.pc + 2 + disp as i32 as u32)
}

pub fn disassemble_bchg(inst: &mut Instruction) -> String {
    let (ea, count) = inst.operands.effective_address_count();
    if bits(inst.opcode, 8, 8) != 0 {
        format!("BCHG D{}, {}", count, ea.mode)
    } else {
        format!("BCHG #{}, {}", count, ea.mode)
    }
}

pub fn disassemble_bclr(inst: &mut Instruction) -> String {
    let (ea, count) = inst.operands.effective_address_count();
    if bits(inst.opcode, 8, 8) != 0 {
        format!("BCLR D{}, {}", count, ea.mode)
    } else {
        format!("BCLR #{}, {}", count, ea.mode)
    }
}

pub fn disassemble_bra(inst: &mut Instruction) -> String {
    let disp = inst.operands.displacement();
    format!("BRA {} <{:#X}>", disp, inst.pc + 2 + disp as i32 as u32)
}

pub fn disassemble_bset(inst: &mut Instruction) -> String {
    let (ea, count) = inst.operands.effective_address_count();
    if bits(inst.opcode, 8, 8) != 0 {
        format!("BSET D{}, {}", count, ea.mode)
    } else {
        format!("BSET #{}, {}", count, ea.mode)
    }
}

pub fn disassemble_bsr(inst: &mut Instruction) -> String {
    let disp = inst.operands.displacement();
    format!("BSR {} <{:#X}>", disp, inst.pc + 2 + disp as i32 as u32)
}

pub fn disassemble_btst(inst: &mut Instruction) -> String {
    let (ea, count) = inst.operands.effective_address_count();
    if bits(inst.opcode, 8, 8) != 0 {
        format!("BTST D{}, {}", count, ea.mode)
    } else {
        format!("BTST #{}, {}", count, ea.mode)
    }
}

pub fn disassemble_chk(inst: &mut Instruction) -> String {
    let (r, ea) = inst.operands.register_effective_address();
    format!("CHK.W {}, D{}", ea.mode, r)
}

pub fn disassemble_clr(inst: &mut Instruction) -> String {
    let (s, ea) = inst.operands.size_effective_address();
    format!("CLR.{} {}", s, ea.mode)
}

pub fn disassemble_cmp(inst: &mut Instruction) -> String {
    let (r, _, s, ea) = inst.operands.register_direction_size_effective_address();
    format!("CMP.{} {}, D{}", s, ea.mode, r)
}

pub fn disassemble_cmpa(inst: &mut Instruction) -> String {
    let (r, s, ea) = inst.operands.register_size_effective_address();
    format!("CMPA.{} {}, A{}", s, ea.mode, r)
}

pub fn disassemble_cmpi(inst: &mut Instruction) -> String {
    let (s, ea, imm) = inst.operands.size_effective_address_immediate();
    format!("CMPI.{} #{}, {}", s, imm, ea.mode)
}

pub fn disassemble_cmpm(inst: &mut Instruction) -> String {
    let (rx, s, ry) = inst.operands.register_size_register();
    format!("CMPM.{} (A{})+, (A{})+", s, ry, rx)
}

pub fn disassemble_dbcc(inst: &mut Instruction) -> String {
    let (cc, r, disp) = inst.operands.condition_register_displacement();
    format!("DB{} D{}, {} <{:#X}>", disassemble_conditional_test(cc), r, disp, inst.pc + 2 + disp as i32 as u32)
}

pub fn disassemble_divs(inst: &mut Instruction) -> String {
    let (r, ea) = inst.operands.register_effective_address();
    format!("DIVS.W {}, D{}", ea.mode, r)
}

pub fn disassemble_divu(inst: &mut Instruction) -> String {
    let (r, ea) = inst.operands.register_effective_address();
    format!("DIVU.W {}, D{}", ea.mode, r)
}

pub fn disassemble_eor(inst: &mut Instruction) -> String {
    let (r, _, s, ea) = inst.operands.register_direction_size_effective_address();
    format!("EOR.{} D{}, {}", s, r, ea.mode)
}

pub fn disassemble_eori(inst: &mut Instruction) -> String {
    let (s, ea, imm) = inst.operands.size_effective_address_immediate();
    format!("EORI.{} #{}, {}", s, imm, ea.mode)
}

pub fn disassemble_eoriccr(inst: &mut Instruction) -> String {
    let imm = inst.operands.immediate();
    format!("EORI {:#X}, CCR", imm)
}

pub fn disassemble_eorisr(inst: &mut Instruction) -> String {
    let imm = inst.operands.immediate();
    format!("EORI {:#X}, SR", imm)
}

pub fn disassemble_exg(inst: &mut Instruction) -> String {
    let (rx, mode, ry) = inst.operands.register_opmode_register();
    if mode == 0b01000 {
        format!("EXG D{}, D{}", rx, ry)
    } else if mode == 0b01001 {
        format!("EXG A{}, A{}", rx, ry)
    } else {
        format!("EXG D{}, A{}", rx, ry)
    }
}

pub fn disassemble_ext(inst: &mut Instruction) -> String {
    let (mode, r) = inst.operands.opmode_register();
    if mode == 0b010 {
        format!("EXT.W D{}", r)
    } else {
        format!("EXT.L D{}", r)
    }
}

pub fn disassemble_illegal(_: &mut Instruction) -> String {
    format!("ILLEGAL")
}

pub fn disassemble_jmp(inst: &mut Instruction) -> String {
    let ea = inst.operands.effective_address();
    format!("JMP {}", ea.mode)
}

pub fn disassemble_jsr(inst: &mut Instruction) -> String {
    let ea = inst.operands.effective_address();
    format!("JSR {}", ea.mode)
}

pub fn disassemble_lea(inst: &mut Instruction) -> String {
    let (r, ea) = inst.operands.register_effective_address();
    format!("LEA {}, A{}", ea.mode, r)
}

pub fn disassemble_link(inst: &mut Instruction) -> String {
    let (r, disp) = inst.operands.register_displacement();
    format!("LINK.W A{}, #{}", r, disp)
}

pub fn disassemble_lsm(inst: &mut Instruction) -> String {
    let (d, ea) = inst.operands.direction_effective_address();
    format!("LS{} {}", d, ea.mode)
}

pub fn disassemble_lsr(inst: &mut Instruction) -> String {
    let (rot, d, s, mode, reg) = inst.operands.rotation_direction_size_mode_register();
    if mode == 1 {
        format!("LS{}.{} D{}, D{}", d, s, rot, reg)
    } else {
        format!("LS{}.{} #{}, D{}", d, s, rot, reg)
    }
}

pub fn disassemble_move(inst: &mut Instruction) -> String {
    let (s, dst, src) = inst.operands.size_effective_address_effective_address();
    format!("MOVE.{} {}, {}", s, src.mode, dst.mode)
}

pub fn disassemble_movea(inst: &mut Instruction) -> String {
    let (s, r, ea) = inst.operands.size_register_effective_address();
    format!("MOVEA.{} {:#X}, A{}", s, ea.mode, r)
}

pub fn disassemble_moveccr(inst: &mut Instruction) -> String {
    let ea = inst.operands.effective_address();
    format!("MOVE {:#X}, CCR", ea.mode)
}

pub fn disassemble_movefsr(inst: &mut Instruction) -> String {
    let ea = inst.operands.effective_address();
    format!("MOVE SR, {:#X}", ea.mode)
}

pub fn disassemble_movesr(inst: &mut Instruction) -> String {
    let ea = inst.operands.effective_address();
    format!("MOVE {:#X}, SR", ea.mode)
}

pub fn disassemble_moveusp(inst: &mut Instruction) -> String {
    let (dir, reg) = inst.operands.direction_register();
    if dir == Direction::UspToRegister {
        format!("MOVE USP, A{}", reg)
    } else {
        format!("MOVE A{}, USP", reg)
    }
}

pub fn disassemble_movem(inst: &mut Instruction) -> String {
    // TODO: disassemble register list.
    let (d, s, ea, list) = inst.operands.direction_size_effective_address_list();
    if d == Direction::MemoryToRegister {
        format!("MOVEM.{} {}, {:#X}", s, ea.mode, list)
    } else {
        format!("MOVEM.{} {:#X}, {}", s, list, ea.mode)
    }
}

pub fn disassemble_movep(inst: &mut Instruction) -> String {
    let (dreg, d, s, areg, disp) = inst.operands.register_direction_size_register_displacement();
    if d == Direction::RegisterToMemory {
        format!("MOVEP.{} D{}, ({}, A{})", s, dreg, disp, areg)
    } else {
        format!("MOVEP.{} ({}, A{}), D{}", s, disp, areg, dreg)
    }
}

pub fn disassemble_moveq(inst: &mut Instruction) -> String {
    let (r, d) = inst.operands.register_data();
    format!("MOVEQ.L #{}, D{}", d, r)
}

pub fn disassemble_muls(inst: &mut Instruction) -> String {
    let (r, ea) = inst.operands.register_effective_address();
    format!("MULS.W {}, D{}", ea.mode, r)
}

pub fn disassemble_mulu(inst: &mut Instruction) -> String {
    let (r, ea) = inst.operands.register_effective_address();
    format!("MULU.W {}, D{}", ea.mode, r)
}

pub fn disassemble_nbcd(inst: &mut Instruction) -> String {
    let ea = inst.operands.effective_address();
    format!("NBCD {}", ea.mode)
}

pub fn disassemble_neg(inst: &mut Instruction) -> String {
    let (s, ea) = inst.operands.size_effective_address();
    format!("NEG.{} {}", s, ea.mode)
}

pub fn disassemble_negx(inst: &mut Instruction) -> String {
    let (s, ea) = inst.operands.size_effective_address();
    format!("NEGX.{} {}", s, ea.mode)
}

pub fn disassemble_nop(_: &mut Instruction) -> String {
    format!("NOP")
}

pub fn disassemble_not(inst: &mut Instruction) -> String {
    let (s, ea) = inst.operands.size_effective_address();
    format!("NOT.{} {}", s, ea.mode)
}

pub fn disassemble_or(inst: &mut Instruction) -> String {
    let (r, d, s, ea) = inst.operands.register_direction_size_effective_address();
    if d == Direction::DstEa {
        format!("OR.{} D{}, {}", s, r, ea.mode)
    } else {
        format!("OR.{} {}, D{}", s, ea.mode, r)
    }
}

pub fn disassemble_ori(inst: &mut Instruction) -> String {
    let (s, ea, imm) = inst.operands.size_effective_address_immediate();
    format!("ORI.{} #{}, {}", s, imm, ea.mode)
}

pub fn disassemble_oriccr(inst: &mut Instruction) -> String {
    let imm = inst.operands.immediate();
    format!("ORI {:#X}, CCR", imm)
}

pub fn disassemble_orisr(inst: &mut Instruction) -> String {
    let imm = inst.operands.immediate();
    format!("ORI {:#X}, SR", imm)
}

pub fn disassemble_pea(inst: &mut Instruction) -> String {
    let ea = inst.operands.effective_address();
    format!("PEA {}", ea.mode)
}

pub fn disassemble_reset(_: &mut Instruction) -> String {
    format!("RESET")
}

pub fn disassemble_rom(inst: &mut Instruction) -> String {
    let (d, ea) = inst.operands.direction_effective_address();
    format!("RO{} {}", d, ea.mode)
}

pub fn disassemble_ror(inst: &mut Instruction) -> String {
    let (rot, d, s, mode, reg) = inst.operands.rotation_direction_size_mode_register();
    if mode == 1 {
        format!("RO{}.{} D{}, D{}", d, s, rot, reg)
    } else {
        format!("RO{}.{} #{}, D{}", d, s, rot, reg)
    }
}

pub fn disassemble_roxm(inst: &mut Instruction) -> String {
    let (d, ea) = inst.operands.direction_effective_address();
    format!("ROX{} {}", d, ea.mode)
}

pub fn disassemble_roxr(inst: &mut Instruction) -> String {
    let (rot, d, s, mode, reg) = inst.operands.rotation_direction_size_mode_register();
    if mode == 1 {
        format!("ROX{}.{} D{}, D{}", d, s, rot, reg)
    } else {
        format!("ROX{}.{} #{}, D{}", d, s, rot, reg)
    }
}

pub fn disassemble_rte(_: &mut Instruction) -> String {
    format!("RTE")
}

pub fn disassemble_rtr(_: &mut Instruction) -> String {
    format!("RTR")
}

pub fn disassemble_rts(_: &mut Instruction) -> String {
    format!("RTS")
}

pub fn disassemble_sbcd(inst: &mut Instruction) -> String {
    let (ry, _, mode, rx) = inst.operands.register_size_mode_register();
    if mode == Direction::MemoryToMemory {
        format!("SBCD -(A{}), -(A{})", rx, ry)
    } else {
        format!("SBCD D{}, D{}", rx, ry)
    }
}

pub fn disassemble_scc(inst: &mut Instruction) -> String {
    let (cc, ea) = inst.operands.condition_effective_address();
    format!("S{} {}", disassemble_conditional_test(cc), ea.mode)
}

pub fn disassemble_stop(inst: &mut Instruction) -> String {
    let imm = inst.operands.immediate();
    format!("STOP #{:#X}", imm)
}

pub fn disassemble_sub(inst: &mut Instruction) -> String {
    let (r, d, s, ea) = inst.operands.register_direction_size_effective_address();
    if d == Direction::DstEa {
        format!("SUB.{} D{}, {}", s, r, ea.mode)
    } else {
        format!("SUB.{} {}, D{}", s, ea.mode, r)
    }
}

pub fn disassemble_suba(inst: &mut Instruction) -> String {
    let (r, s, ea) = inst.operands.register_size_effective_address();
    format!("SUBA.{} {}, A{}", s, ea.mode, r)
}

pub fn disassemble_subi(inst: &mut Instruction) -> String {
    let (s, ea, imm) = inst.operands.size_effective_address_immediate();
    format!("SUBI.{} #{}, {}", s, imm, ea.mode)
}

pub fn disassemble_subq(inst: &mut Instruction) -> String {
    let (d, s, ea) = inst.operands.data_size_effective_address();
    format!("SUBQ.{} #{}, {}", s, d, ea.mode)
}

pub fn disassemble_subx(inst: &mut Instruction) -> String {
    let (ry, s, mode, rx) = inst.operands.register_size_mode_register();
    if mode == Direction::MemoryToMemory {
        format!("SUBX.{} -(A{}), -(A{})", s, rx, ry)
    } else {
        format!("SUBX.{} D{}, D{}", s, rx, ry)
    }
}

pub fn disassemble_swap(inst: &mut Instruction) -> String {
    let r = inst.operands.register();
    format!("SWAP D{}", r)
}

pub fn disassemble_tas(inst: &mut Instruction) -> String {
    let ea = inst.operands.effective_address();
    format!("TAS {}", ea.mode)
}

pub fn disassemble_trap(inst: &mut Instruction) -> String {
    let v = inst.operands.vector();
    format!("TRAP #{}", v)
}

pub fn disassemble_trapv(_: &mut Instruction) -> String {
    format!("TRAPV")
}

pub fn disassemble_tst(inst: &mut Instruction) -> String {
    let (s, ea) = inst.operands.size_effective_address();
    format!("TST.{} {}", s, ea.mode)
}

pub fn disassemble_unlk(inst: &mut Instruction) -> String {
    let r = inst.operands.register();
    format!("UNLK A{}", r)
}
