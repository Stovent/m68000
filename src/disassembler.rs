use super::instruction::Instruction;
use super::operands::Direction;
use super::status_register::disassemble_conditional_test;
use super::utils::bits;

pub(super) fn disassemble_unknown_instruction(inst: &mut Instruction) -> String {
    format!("Unknown instruction {:04X} at {:#X}", inst.opcode, inst.pc)
}

pub(super) fn disassemble_abcd(inst: &mut Instruction) -> String {
    let (rx, _, mode, ry) = inst.operands.register_size_mode_register();
    if mode == Direction::MemoryToMemory {
        format!("ABCD -(A{}), -(A{})", ry, rx)
    } else {
        format!("ABCD D{}, D{}", ry, rx)
    }
}

pub(super) fn disassemble_add(inst: &mut Instruction) -> String {
    let (r, d, s, ea) = inst.operands.register_direction_size_effective_address();
    if d == Direction::DstEa {
        format!("ADD.{} D{}, {}", s, r, ea)
    } else {
        format!("ADD.{} {}, D{}", s, ea, r)
    }
}

pub(super) fn disassemble_adda(inst: &mut Instruction) -> String {
    let (r, s, ea) = inst.operands.register_size_effective_address();
    format!("ADDA.{} {}, A{}", s, ea, r)
}

pub(super) fn disassemble_addi(inst: &mut Instruction) -> String {
    let (s, ea, imm) = inst.operands.size_effective_address_immediate();
    format!("ADDI.{} #{}, {}", s, imm, ea)
}

pub(super) fn disassemble_addq(inst: &mut Instruction) -> String {
    let (d, s, ea) = inst.operands.data_size_effective_address();
    format!("ADDQ.{} #{}, {}", s, d, ea)
}

pub(super) fn disassemble_addx(inst: &mut Instruction) -> String {
    let (rx, s, mode, ry) = inst.operands.register_size_mode_register();
    if mode == Direction::MemoryToMemory {
        format!("ADDX.{} -(A{}), -(A{})", s, ry, rx)
    } else {
        format!("ADDX.{} D{}, D{}", s, ry, rx)
    }
}

pub(super) fn disassemble_and(inst: &mut Instruction) -> String {
    let (r, d, s, ea) = inst.operands.register_direction_size_effective_address();
    if d == Direction::DstEa {
        format!("AND.{} D{}, {}", s, r, ea)
    } else {
        format!("AND.{} {}, D{}", s, ea, r)
    }
}

pub(super) fn disassemble_andi(inst: &mut Instruction) -> String {
    let (s, ea, imm) = inst.operands.size_effective_address_immediate();
    format!("ANDI.{} #{}, {}", s, imm, ea)
}

pub(super) fn disassemble_andiccr(inst: &mut Instruction) -> String {
    let imm = inst.operands.immediate();
    format!("ANDI {:#X}, CCR", imm)
}

pub(super) fn disassemble_andisr(inst: &mut Instruction) -> String {
    let imm = inst.operands.immediate();
    format!("ANDI {:#X}, SR", imm)
}

pub(super) fn disassemble_asm(inst: &mut Instruction) -> String {
    let (d, ea) = inst.operands.direction_effective_address();
    format!("AS{} {}", d, ea)
}

pub(super) fn disassemble_asr(inst: &mut Instruction) -> String {
    let (rot, d, s, mode, reg) = inst.operands.rotation_direction_size_mode_register();
    if mode == 1 {
        format!("AS{}.{} D{}, D{}", d, s, rot, reg)
    } else {
        format!("AS{}.{} #{}, D{}", d, s, rot, reg)
    }
}

pub(super) fn disassemble_bcc(inst: &mut Instruction) -> String {
    let (cc, disp) = inst.operands.condition_displacement();
    format!("B{} {} <{:#X}>", disassemble_conditional_test(cc), disp, inst.pc + 2 + disp as i32 as u32)
}

pub(super) fn disassemble_bchg(inst: &mut Instruction) -> String {
    let (ea, count) = inst.operands.effective_address_count();
    if bits(inst.opcode, 8, 8) != 0 {
        format!("BCHG D{}, {}", count, ea)
    } else {
        format!("BCHG #{}, {}", count, ea)
    }
}

pub(super) fn disassemble_bclr(inst: &mut Instruction) -> String {
    let (ea, count) = inst.operands.effective_address_count();
    if bits(inst.opcode, 8, 8) != 0 {
        format!("BCLR D{}, {}", count, ea)
    } else {
        format!("BCLR #{}, {}", count, ea)
    }
}

pub(super) fn disassemble_bra(inst: &mut Instruction) -> String {
    let disp = inst.operands.displacement();
    format!("BRA {} <{:#X}>", disp, inst.pc + 2 + disp as i32 as u32)
}

pub(super) fn disassemble_bset(inst: &mut Instruction) -> String {
    let (ea, count) = inst.operands.effective_address_count();
    if bits(inst.opcode, 8, 8) != 0 {
        format!("BSET D{}, {}", count, ea)
    } else {
        format!("BSET #{}, {}", count, ea)
    }
}

pub(super) fn disassemble_bsr(inst: &mut Instruction) -> String {
    let disp = inst.operands.displacement();
    format!("BSR {} <{:#X}>", disp, inst.pc + 2 + disp as i32 as u32)
}

pub(super) fn disassemble_btst(inst: &mut Instruction) -> String {
    let (ea, count) = inst.operands.effective_address_count();
    if bits(inst.opcode, 8, 8) != 0 {
        format!("BTST D{}, {}", count, ea)
    } else {
        format!("BTST #{}, {}", count, ea)
    }
}

pub(super) fn disassemble_chk(inst: &mut Instruction) -> String {
    let (r, ea) = inst.operands.register_effective_address();
    format!("CHK.W {}, D{}", ea, r)
}

pub(super) fn disassemble_clr(inst: &mut Instruction) -> String {
    let (s, ea) = inst.operands.size_effective_address();
    format!("CLR.{} {}", s, ea)
}

pub(super) fn disassemble_cmp(inst: &mut Instruction) -> String {
    let (r, _, s, ea) = inst.operands.register_direction_size_effective_address();
    format!("CMP.{} {}, D{}", s, ea, r)
}

pub(super) fn disassemble_cmpa(inst: &mut Instruction) -> String {
    let (r, s, ea) = inst.operands.register_size_effective_address();
    format!("CMPA.{} {}, A{}", s, ea, r)
}

pub(super) fn disassemble_cmpi(inst: &mut Instruction) -> String {
    let (s, ea, imm) = inst.operands.size_effective_address_immediate();
    format!("CMPI.{} #{}, {}", s, imm, ea)
}

pub(super) fn disassemble_cmpm(inst: &mut Instruction) -> String {
    let (rx, s, ry) = inst.operands.register_size_register();
    format!("CMPM.{} (A{})+, (A{})+", s, ry, rx)
}

pub(super) fn disassemble_dbcc(inst: &mut Instruction) -> String {
    let (cc, r, disp) = inst.operands.condition_register_disp();
    format!("DB{} D{}, {} <{:#X}>", disassemble_conditional_test(cc), r, disp, inst.pc + 2 + disp as i32 as u32)
}

pub(super) fn disassemble_divs(inst: &mut Instruction) -> String {
    let (r, ea) = inst.operands.register_effective_address();
    format!("DIVS.W {}, D{}", ea, r)
}

pub(super) fn disassemble_divu(inst: &mut Instruction) -> String {
    let (r, ea) = inst.operands.register_effective_address();
    format!("DIVU.W {}, D{}", ea, r)
}

pub(super) fn disassemble_eor(inst: &mut Instruction) -> String {
    let (r, _, s, ea) = inst.operands.register_direction_size_effective_address();
    format!("EOR.{} D{}, {}", s, r, ea)
}

pub(super) fn disassemble_eori(inst: &mut Instruction) -> String {
    let (s, ea, imm) = inst.operands.size_effective_address_immediate();
    format!("EORI.{} #{}, {}", s, imm, ea)
}

pub(super) fn disassemble_eoriccr(inst: &mut Instruction) -> String {
    let imm = inst.operands.immediate();
    format!("EORI {:#X}, CCR", imm)
}

pub(super) fn disassemble_eorisr(inst: &mut Instruction) -> String {
    let imm = inst.operands.immediate();
    format!("EORI {:#X}, SR", imm)
}

pub(super) fn disassemble_exg(inst: &mut Instruction) -> String {
    let (rx, mode, ry) = inst.operands.register_opmode_register();
    if mode == 0b01000 {
        format!("EXG D{}, D{}", rx, ry)
    } else if mode == 0b01001 {
        format!("EXG A{}, A{}", rx, ry)
    } else {
        format!("EXG D{}, A{}", rx, ry)
    }
}

pub(super) fn disassemble_ext(inst: &mut Instruction) -> String {
    let (mode, r) = inst.operands.opmode_register();
    if mode == 0b010 {
        format!("EXT.W D{}", r)
    } else {
        format!("EXT.L D{}", r)
    }
}

pub(super) fn disassemble_illegal(_: &mut Instruction) -> String {
    format!("ILLEGAL")
}

pub(super) fn disassemble_jmp(inst: &mut Instruction) -> String {
    let ea = inst.operands.effective_address();
    format!("JMP {}", ea)
}

pub(super) fn disassemble_jsr(inst: &mut Instruction) -> String {
    let ea = inst.operands.effective_address();
    format!("JSR {}", ea)
}

pub(super) fn disassemble_lea(inst: &mut Instruction) -> String {
    let (r, ea) = inst.operands.register_effective_address();
    format!("LEA {}, A{}", ea, r)
}

pub(super) fn disassemble_link(inst: &mut Instruction) -> String {
    let (r, disp) = inst.operands.register_disp();
    format!("LINK.W A{}, #{}", r, disp)
}

pub(super) fn disassemble_lsm(inst: &mut Instruction) -> String {
    let (d, ea) = inst.operands.direction_effective_address();
    format!("LS{} {}", d, ea)
}

pub(super) fn disassemble_lsr(inst: &mut Instruction) -> String {
    let (rot, d, s, mode, reg) = inst.operands.rotation_direction_size_mode_register();
    if mode == 1 {
        format!("LS{}.{} D{}, D{}", d, s, rot, reg)
    } else {
        format!("LS{}.{} #{}, D{}", d, s, rot, reg)
    }
}

pub(super) fn disassemble_move(inst: &mut Instruction) -> String {
    let (s, dst, src) = inst.operands.size_effective_address_effective_address();
    format!("MOVE.{} {}, {}", s, src, dst)
}

pub(super) fn disassemble_movea(inst: &mut Instruction) -> String {
    let (s, r, ea) = inst.operands.size_register_effective_address();
    format!("MOVEA.{} {:#X}, A{}", s, ea, r)
}

pub(super) fn disassemble_moveccr(inst: &mut Instruction) -> String {
    let ea = inst.operands.effective_address();
    format!("MOVE {:#X}, CCR", ea)
}

pub(super) fn disassemble_movefsr(inst: &mut Instruction) -> String {
    let ea = inst.operands.effective_address();
    format!("MOVE SR, {:#X}", ea)
}

pub(super) fn disassemble_movesr(inst: &mut Instruction) -> String {
    let ea = inst.operands.effective_address();
    format!("MOVE {:#X}, SR", ea)
}

pub(super) fn disassemble_moveusp(inst: &mut Instruction) -> String {
    let (d, ea) = inst.operands.direction_register();
    if d == Direction::UspToRegister {
        format!("MOVE USP, {:#X}", ea)
    } else {
        format!("MOVE {:#X}, USP", ea)
    }
}

pub(super) fn disassemble_movem(inst: &mut Instruction) -> String {
    // TODO: disassemble register list.
    let (d, s, ea, list) = inst.operands.direction_size_effective_address_list();
    if d == Direction::MemoryToRegister {
        format!("MOVEM.{} {}, {:#X}", s, ea, list)
    } else {
        format!("MOVEM.{} {:#X}, {}", s, list, ea)
    }
}

pub(super) fn disassemble_movep(inst: &mut Instruction) -> String {
    let (dreg, d, s, areg, disp) = inst.operands.register_direction_size_register_disp();
    if d == Direction::RegisterToMemory {
        format!("MOVEP.{} D{}, ({}, A{})", s, dreg, disp, areg)
    } else {
        format!("MOVEP.{} ({}, A{}), D{}", s, disp, areg, dreg)
    }
}

pub(super) fn disassemble_moveq(inst: &mut Instruction) -> String {
    let (r, d) = inst.operands.register_data();
    format!("MOVEQ.L #{}, D{}", d, r)
}

pub(super) fn disassemble_muls(inst: &mut Instruction) -> String {
    let (r, ea) = inst.operands.register_effective_address();
    format!("MULS.W {}, D{}", ea, r)
}

pub(super) fn disassemble_mulu(inst: &mut Instruction) -> String {
    let (r, ea) = inst.operands.register_effective_address();
    format!("MULU.W {}, D{}", ea, r)
}

pub(super) fn disassemble_nbcd(inst: &mut Instruction) -> String {
    let ea = inst.operands.effective_address();
    format!("NBCD {}", ea)
}

pub(super) fn disassemble_neg(inst: &mut Instruction) -> String {
    let (s, ea) = inst.operands.size_effective_address();
    format!("NEG.{} {}", s, ea)
}

pub(super) fn disassemble_negx(inst: &mut Instruction) -> String {
    let (s, ea) = inst.operands.size_effective_address();
    format!("NEGX.{} {}", s, ea)
}

pub(super) fn disassemble_nop(_: &mut Instruction) -> String {
    format!("NOP")
}

pub(super) fn disassemble_not(inst: &mut Instruction) -> String {
    let (s, ea) = inst.operands.size_effective_address();
    format!("NOT.{} {}", s, ea)
}

pub(super) fn disassemble_or(inst: &mut Instruction) -> String {
    let (r, d, s, ea) = inst.operands.register_direction_size_effective_address();
    if d == Direction::DstEa {
        format!("OR.{} D{}, {}", s, r, ea)
    } else {
        format!("OR.{} {}, D{}", s, ea, r)
    }
}

pub(super) fn disassemble_ori(inst: &mut Instruction) -> String {
    let (s, ea, imm) = inst.operands.size_effective_address_immediate();
    format!("ORI.{} #{}, {}", s, imm, ea)
}

pub(super) fn disassemble_oriccr(inst: &mut Instruction) -> String {
    let imm = inst.operands.immediate();
    format!("ORI {:#X}, CCR", imm)
}

pub(super) fn disassemble_orisr(inst: &mut Instruction) -> String {
    let imm = inst.operands.immediate();
    format!("ORI {:#X}, SR", imm)
}

pub(super) fn disassemble_pea(inst: &mut Instruction) -> String {
    let ea = inst.operands.effective_address();
    format!("PEA {}", ea)
}

pub(super) fn disassemble_reset(_: &mut Instruction) -> String {
    format!("RESET")
}

pub(super) fn disassemble_rom(inst: &mut Instruction) -> String {
    let (d, ea) = inst.operands.direction_effective_address();
    format!("RO{} {}", d, ea)
}

pub(super) fn disassemble_ror(inst: &mut Instruction) -> String {
    let (rot, d, s, mode, reg) = inst.operands.rotation_direction_size_mode_register();
    if mode == 1 {
        format!("RO{}.{} D{}, D{}", d, s, rot, reg)
    } else {
        format!("RO{}.{} #{}, D{}", d, s, rot, reg)
    }
}

pub(super) fn disassemble_roxm(inst: &mut Instruction) -> String {
    let (d, ea) = inst.operands.direction_effective_address();
    format!("ROX{} {}", d, ea)
}

pub(super) fn disassemble_roxr(inst: &mut Instruction) -> String {
    let (rot, d, s, mode, reg) = inst.operands.rotation_direction_size_mode_register();
    if mode == 1 {
        format!("ROX{}.{} D{}, D{}", d, s, rot, reg)
    } else {
        format!("ROX{}.{} #{}, D{}", d, s, rot, reg)
    }
}

pub(super) fn disassemble_rte(_: &mut Instruction) -> String {
    format!("RTE")
}

pub(super) fn disassemble_rtr(_: &mut Instruction) -> String {
    format!("RTR")
}

pub(super) fn disassemble_rts(_: &mut Instruction) -> String {
    format!("RTS")
}

pub(super) fn disassemble_sbcd(inst: &mut Instruction) -> String {
    let (ry, _, mode, rx) = inst.operands.register_size_mode_register();
    if mode == Direction::MemoryToMemory {
        format!("SBCD -(A{}), -(A{})", rx, ry)
    } else {
        format!("SBCD D{}, D{}", rx, ry)
    }
}

pub(super) fn disassemble_scc(inst: &mut Instruction) -> String {
    let (cc, ea) = inst.operands.condition_effective_address();
    format!("S{} {}", disassemble_conditional_test(cc), ea)
}

pub(super) fn disassemble_stop(inst: &mut Instruction) -> String {
    let imm = inst.operands.immediate();
    format!("STOP #{:#X}", imm)
}

pub(super) fn disassemble_sub(inst: &mut Instruction) -> String {
    let (r, d, s, ea) = inst.operands.register_direction_size_effective_address();
    if d == Direction::DstEa {
        format!("SUB.{} D{}, {}", s, r, ea)
    } else {
        format!("SUB.{} {}, D{}", s, ea, r)
    }
}

pub(super) fn disassemble_suba(inst: &mut Instruction) -> String {
    let (r, s, ea) = inst.operands.register_size_effective_address();
    format!("SUBA.{} {}, A{}", s, ea, r)
}

pub(super) fn disassemble_subi(inst: &mut Instruction) -> String {
    let (s, ea, imm) = inst.operands.size_effective_address_immediate();
    format!("SUBI.{} #{}, {}", s, imm, ea)
}

pub(super) fn disassemble_subq(inst: &mut Instruction) -> String {
    let (d, s, ea) = inst.operands.data_size_effective_address();
    format!("SUBQ.{} #{}, {}", s, d, ea)
}

pub(super) fn disassemble_subx(inst: &mut Instruction) -> String {
    let (ry, s, mode, rx) = inst.operands.register_size_mode_register();
    if mode == Direction::MemoryToMemory {
        format!("SUBX.{} -(A{}), -(A{})", s, rx, ry)
    } else {
        format!("SUBX.{} D{}, D{}", s, rx, ry)
    }
}

pub(super) fn disassemble_swap(inst: &mut Instruction) -> String {
    let r = inst.operands.register();
    format!("SWAP D{}", r)
}

pub(super) fn disassemble_tas(inst: &mut Instruction) -> String {
    let ea = inst.operands.effective_address();
    format!("TAS {}", ea)
}

pub(super) fn disassemble_trap(inst: &mut Instruction) -> String {
    let v = inst.operands.vector();
    format!("TRAP #{}", v)
}

pub(super) fn disassemble_trapv(_: &mut Instruction) -> String {
    format!("TRAPV")
}

pub(super) fn disassemble_tst(inst: &mut Instruction) -> String {
    let (s, ea) = inst.operands.size_effective_address();
    format!("TST.{} {}", s, ea)
}

pub(super) fn disassemble_unlk(inst: &mut Instruction) -> String {
    let r = inst.operands.register();
    format!("UNLK A{}", r)
}
