// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! This file is used to generate a test ROM, that will be later tested by the interpreter.

use m68000::addressing_modes::{AddressingMode as AM, BriefExtensionWord as BEW};
use m68000::assembler as asm;
use m68000::assembler::Condition as CC;
use m68000::instruction::{
    Direction,
    Direction::*,
    Size,
    Size::*,
};

use std::panic::catch_unwind;

const ALL_DIRECTIONS: [Direction; 13] = [
    RegisterToMemory,
    MemoryToRegister,
    DstReg,
    DstEa,
    Left,
    Right,
    RegisterToUsp,
    UspToRegister,
    RegisterToRegister,
    MemoryToMemory,
    ExchangeData,
    ExchangeAddress,
    ExchangeDataAddress,
];

fn get_condition(cc: u16) -> CC {
    match cc {
        0x0 => CC::T,
        0x1 => CC::F,
        0x2 => CC::HI,
        0x3 => CC::LS,
        0x4 => CC::CC,
        0x5 => CC::CS,
        0x6 => CC::NE,
        0x7 => CC::EQ,
        0x8 => CC::VC,
        0x9 => CC::VS,
        0xA => CC::PL,
        0xB => CC::MI,
        0xC => CC::GE,
        0xD => CC::LT,
        0xE => CC::GT,
        0xF => CC::LE,
        _ => panic!("Wrong cc."),
    }
}

/// Assembles the given addressing mode.
fn assemble_am(opcode: u16, am: AM) -> Vec<u16> {
    let (op, ext) = am.assemble(false);
    let opcode = opcode | op;
    let mut asm = vec![opcode];
    asm.extend_from_slice(&ext);
    asm
}

fn assemble_move_am(opcode: u16, dst: AM, src: AM) -> Vec<u16> {
    let (dop, dext) = dst.assemble_move_dst();
    let (sop, sext) = src.assemble(false);
    let opcode = opcode | dop | sop;
    let mut asm = vec![opcode];
    asm.extend_from_slice(&sext);
    asm.extend_from_slice(&dext);
    asm
}

/// Assembles the given addressing mode for use with Immediate operand instruction + Bit manip + MOVEM.
fn immediate_assemble_am(opcode: u16, imm: u16, am: AM) -> Vec<u16> {
    let (op, ext) = am.assemble(false);
    let opcode = opcode | op;
    let mut asm = vec![opcode, imm];
    asm.extend_from_slice(&ext);
    asm
}

/// Assembles the given addressing mode for use with long-sized Immediate operand instruction.
fn immediate_long_assemble_am(opcode: u16, imm: u32, am: AM) -> Vec<u16> {
    let (op, ext) = am.assemble(false);
    let opcode = opcode | op;
    let mut asm = vec![opcode, (imm >> 16) as u16, imm as u16];
    asm.extend_from_slice(&ext);
    asm
}

/// Assembles the given Immediate addressing mode.
fn assemble_am_immediate(opcode: u16, data: u32, long: bool) -> Vec<u16> {
    let (op, ext) = AM::Immediate(data).assemble(long);
    let opcode = opcode | op;
    let mut asm = vec![opcode];
    asm.extend_from_slice(&ext);
    asm
}

/// Assembles the given Immediate addressing mode.
fn assemble_move_am_immediate(opcode: u16, dst: AM, data: u32, long: bool) -> Vec<u16> {
    let (dop, dext) = dst.assemble_move_dst();
    let (sop, sext) = AM::Immediate(data).assemble(long);
    let opcode = opcode | dop | sop;
    let mut asm = vec![opcode];
    asm.extend_from_slice(&sext);
    asm.extend_from_slice(&dext);
    asm
}

/// ADDI, ANDI, CMPI, EORI, ORI, SUBI
fn size_effective_address_immediate(asm: fn(Size, AM, u32) -> Vec<u16>, opcode: u16) {
    for size in [Byte, Word, Long] {
        for imm in [1, 0x106, 0x1_0005, 0x100_0007] {
            let bew = BEW::new(true, 0, true, 0);
            let opcode = opcode | Into::<u16>::into(size) << 6;

            if size.is_long() {
                assert_eq!(&asm(size, AM::Drd(imm as u8), imm), &immediate_long_assemble_am(opcode, imm, AM::Drd(imm as u8)));
                assert_eq!(&asm(size, AM::Ari(imm as u8), imm), &immediate_long_assemble_am(opcode, imm, AM::Ari(imm as u8)));
                assert_eq!(&asm(size, AM::Ariwpo(imm as u8), imm), &immediate_long_assemble_am(opcode, imm, AM::Ariwpo(imm as u8)));
                assert_eq!(&asm(size, AM::Ariwpr(imm as u8), imm), &immediate_long_assemble_am(opcode, imm, AM::Ariwpr(imm as u8)));
                assert_eq!(&asm(size, AM::Ariwd(imm as u8, 0), imm), &immediate_long_assemble_am(opcode, imm, AM::Ariwd(imm as u8, 0)));
                assert_eq!(&asm(size, AM::Ariwi8(imm as u8, bew), imm), &immediate_long_assemble_am(opcode, imm, AM::Ariwi8(imm as u8, bew)));
                assert_eq!(&asm(size, AM::AbsShort(0xFF00), imm), &immediate_long_assemble_am(opcode, imm, AM::AbsShort(0xFF00)));
                assert_eq!(&asm(size, AM::AbsLong(0xFF_0000), imm), &immediate_long_assemble_am(opcode, imm, AM::AbsLong(0xFF_0000)));
            } else {
                let imm = if size.is_byte() { imm & 0x00FF } else { imm };
                assert_eq!(&asm(size, AM::Drd(imm as u8), imm), &immediate_assemble_am(opcode, imm as u16, AM::Drd(imm as u8)));
                assert_eq!(&asm(size, AM::Ari(imm as u8), imm), &immediate_assemble_am(opcode, imm as u16, AM::Ari(imm as u8)));
                assert_eq!(&asm(size, AM::Ariwpo(imm as u8), imm), &immediate_assemble_am(opcode, imm as u16, AM::Ariwpo(imm as u8)));
                assert_eq!(&asm(size, AM::Ariwpr(imm as u8), imm), &immediate_assemble_am(opcode, imm as u16, AM::Ariwpr(imm as u8)));
                assert_eq!(&asm(size, AM::Ariwd(imm as u8, 0), imm), &immediate_assemble_am(opcode, imm as u16, AM::Ariwd(imm as u8, 0)));
                assert_eq!(&asm(size, AM::Ariwi8(imm as u8, bew), imm), &immediate_assemble_am(opcode, imm as u16, AM::Ariwi8(imm as u8, bew)));
                assert_eq!(&asm(size, AM::AbsShort(0xFF00), imm), &immediate_assemble_am(opcode, imm as u16, AM::AbsShort(0xFF00)));
                assert_eq!(&asm(size, AM::AbsLong(0xFF_0000), imm), &immediate_assemble_am(opcode, imm as u16, AM::AbsLong(0xFF_0000)));
            }

            catch_unwind(|| asm(size, AM::Ard(imm as u8), imm)).unwrap_err();
            catch_unwind(|| asm(size, AM::Pciwd(0, 0), imm)).unwrap_err();
            catch_unwind(|| asm(size, AM::Pciwi8(0, bew), imm)).unwrap_err();
            catch_unwind(|| asm(size, AM::Immediate(0), imm)).unwrap_err();
        }
    }
}

/// BCHG, BCLR, BSET, BTST
fn effective_address_count(asm_dynamic: fn(u8, AM) -> Vec<u16>, asm_static: fn(AM, u8) -> Vec<u16>, opcode: u16) {
    for reg in 0..10 {
        let bew = BEW::new(true, 0, true, 0);
        let opcode_dynamic = opcode | 0x0100 | (reg as u16) << 9;
        let opcode_static = opcode | 0x0800;

        if reg <= 7 {
            assert_eq!(&asm_dynamic(reg, AM::Drd(reg)), &assemble_am(opcode_dynamic, AM::Drd(reg)));
            assert_eq!(&asm_static(AM::Drd(reg), reg), &immediate_assemble_am(opcode_static, reg as u16, AM::Drd(reg)));
            catch_unwind(|| asm_dynamic(reg, AM::Ard(reg))).unwrap_err();
            catch_unwind(|| asm_static(AM::Ard(reg), reg)).unwrap_err();
            assert_eq!(&asm_dynamic(reg, AM::Ari(reg)), &assemble_am(opcode_dynamic, AM::Ari(reg)));
            assert_eq!(&asm_static(AM::Ari(reg), reg), &immediate_assemble_am(opcode_static, reg as u16, AM::Ari(reg)));
            assert_eq!(&asm_dynamic(reg, AM::Ariwpo(reg)), &assemble_am(opcode_dynamic, AM::Ariwpo(reg)));
            assert_eq!(&asm_static(AM::Ariwpo(reg), reg), &immediate_assemble_am(opcode_static, reg as u16, AM::Ariwpo(reg)));
            assert_eq!(&asm_dynamic(reg, AM::Ariwpr(reg)), &assemble_am(opcode_dynamic, AM::Ariwpr(reg)));
            assert_eq!(&asm_static(AM::Ariwpr(reg), reg), &immediate_assemble_am(opcode_static, reg as u16, AM::Ariwpr(reg)));
            assert_eq!(&asm_dynamic(reg, AM::Ariwd(reg, 0)), &assemble_am(opcode_dynamic, AM::Ariwd(reg, 0)));
            assert_eq!(&asm_static(AM::Ariwd(reg, 0), reg), &immediate_assemble_am(opcode_static, reg as u16, AM::Ariwd(reg, 0)));
            assert_eq!(&asm_dynamic(reg, AM::Ariwi8(reg, bew)), &assemble_am(opcode_dynamic, AM::Ariwi8(reg, bew)));
            assert_eq!(&asm_static(AM::Ariwi8(reg, bew), reg), &immediate_assemble_am(opcode_static, reg as u16, AM::Ariwi8(reg, bew)));
            assert_eq!(&asm_dynamic(reg, AM::AbsShort(0xFF00)), &assemble_am(opcode_dynamic, AM::AbsShort(0xFF00)));
            assert_eq!(&asm_static(AM::AbsShort(0xFF00), reg), &immediate_assemble_am(opcode_static, reg as u16, AM::AbsShort(0xFF00)));
            assert_eq!(&asm_dynamic(reg, AM::AbsLong(0xFF_0000)), &assemble_am(opcode_dynamic, AM::AbsLong(0xFF_0000)));
            assert_eq!(&asm_static(AM::AbsLong(0xFF_0000), reg), &immediate_assemble_am(opcode_static, reg as u16, AM::AbsLong(0xFF_0000)));

            if asm_dynamic == asm::btst_dynamic && asm_static == asm::btst_static {
                assert_eq!(&asm_dynamic(reg, AM::Pciwd(0, 0)), &assemble_am(opcode_dynamic, AM::Pciwd(0, 0)));
                assert_eq!(&asm_static(AM::Pciwd(0, 0), reg), &immediate_assemble_am(opcode_static, reg as u16, AM::Pciwd(0, 0)));
                assert_eq!(&asm_dynamic(reg, AM::Pciwi8(0, bew)), &assemble_am(opcode_dynamic, AM::Pciwi8(0, bew)));
                assert_eq!(&asm_static(AM::Pciwi8(0, bew), reg), &immediate_assemble_am(opcode_static, reg as u16, AM::Pciwi8(0, bew)));
                assert_eq!(&asm_dynamic(reg, AM::Immediate(0xFF00)), &assemble_am(opcode_dynamic, AM::Immediate(0xFF00)));
                assert_eq!(&asm_static(AM::Immediate(0xFF00), reg), &immediate_assemble_am(opcode_static, reg as u16, AM::Immediate(0xFF00)));
            } else {
                catch_unwind(|| asm_dynamic(reg, AM::Pciwd(0, 0))).unwrap_err();
                catch_unwind(|| asm_static(AM::Pciwd(0, 0), reg)).unwrap_err();
                catch_unwind(|| asm_dynamic(reg, AM::Pciwi8(0, bew))).unwrap_err();
                catch_unwind(|| asm_static(AM::Pciwi8(0, bew), reg)).unwrap_err();
                catch_unwind(|| asm_dynamic(reg, AM::Immediate(0))).unwrap_err();
                catch_unwind(|| asm_static(AM::Immediate(0), reg)).unwrap_err();
            }
        } else {
            catch_unwind(|| asm_dynamic(reg, AM::Drd(reg))).unwrap_err();
            catch_unwind(|| asm_static(AM::Drd(reg), reg)).unwrap_err();
            catch_unwind(|| asm_dynamic(reg, AM::Ari(reg))).unwrap_err();
            catch_unwind(|| asm_static(AM::Ari(reg), reg)).unwrap_err();
            catch_unwind(|| asm_dynamic(reg, AM::Ariwpo(reg))).unwrap_err();
            catch_unwind(|| asm_static(AM::Ariwpo(reg), reg)).unwrap_err();
            catch_unwind(|| asm_dynamic(reg, AM::Ariwpr(reg))).unwrap_err();
            catch_unwind(|| asm_static(AM::Ariwpr(reg), reg)).unwrap_err();
            catch_unwind(|| asm_dynamic(reg, AM::Ariwd(reg, 0))).unwrap_err();
            catch_unwind(|| asm_static(AM::Ariwd(reg, 0), reg)).unwrap_err();
            catch_unwind(|| asm_dynamic(reg, AM::Ariwi8(reg, bew))).unwrap_err();
            catch_unwind(|| asm_static(AM::Ariwi8(reg, bew), reg)).unwrap_err();
        }
    }
}

/// JMP, JSR, MOVE (f) SR CCR, NBCD, PEA, TAS
fn effective_address(asm: fn(AM) -> Vec<u16>, opcode: u16) {
    for reg in 0..10 {
        let bew = BEW::new(true, 0, true, 0);
        if reg <= 7 {
            assert_eq!(&asm(AM::Ari(reg)), &assemble_am(opcode, AM::Ari(reg)));
            if asm != asm::jmp && asm != asm::jsr && asm != asm::pea {
                assert_eq!(&asm(AM::Drd(reg)), &assemble_am(opcode, AM::Drd(reg)));
                assert_eq!(&asm(AM::Ariwpo(reg)), &assemble_am(opcode, AM::Ariwpo(reg)));
                assert_eq!(&asm(AM::Ariwpr(reg)), &assemble_am(opcode, AM::Ariwpr(reg)));
            } else {
                catch_unwind(|| asm(AM::Drd(reg))).unwrap_err();
                catch_unwind(|| asm(AM::Ariwpo(reg))).unwrap_err();
                catch_unwind(|| asm(AM::Ariwpr(reg))).unwrap_err();
            }
            assert_eq!(&asm(AM::Ariwd(reg, 0)), &assemble_am(opcode, AM::Ariwd(reg, 0)));
            assert_eq!(&asm(AM::Ariwi8(reg, bew)), &assemble_am(opcode, AM::Ariwi8(reg, bew)));
            assert_eq!(&asm(AM::AbsShort(0xFF00)), &assemble_am(opcode, AM::AbsShort(0xFF00)));
            assert_eq!(&asm(AM::AbsLong(0xFF_0000)), &assemble_am(opcode, AM::AbsLong(0xFF_0000)));

            catch_unwind(|| asm(AM::Ard(reg))).unwrap_err();
            if asm == asm::jmp || asm == asm::jsr || asm == asm::moveccr || asm == asm::movesr || asm == asm::pea {
                assert_eq!(&asm(AM::Pciwd(0, 0)), &assemble_am(opcode, AM::Pciwd(0, 0)));
                assert_eq!(&asm(AM::Pciwi8(0, bew)), &assemble_am(opcode, AM::Pciwi8(0, bew)));
                if asm != asm::jmp && asm != asm::jsr && asm != asm::pea {
                    assert_eq!(&asm(AM::Immediate(0xFF00)), &assemble_am(opcode, AM::Immediate(0xFF00)));
                }
            } else {
                catch_unwind(|| asm(AM::Pciwd(0, 0))).unwrap_err();
                catch_unwind(|| asm(AM::Pciwi8(0, bew))).unwrap_err();
                catch_unwind(|| asm(AM::Immediate(0))).unwrap_err();
            }
        } else {
            catch_unwind(|| asm(AM::Drd(reg))).unwrap_err();
            catch_unwind(|| asm(AM::Ari(reg))).unwrap_err();
            catch_unwind(|| asm(AM::Ariwpo(reg))).unwrap_err();
            catch_unwind(|| asm(AM::Ariwpr(reg))).unwrap_err();
            catch_unwind(|| asm(AM::Ariwd(reg, 0))).unwrap_err();
            catch_unwind(|| asm(AM::Ariwi8(reg, bew))).unwrap_err();
            if asm != asm::moveccr && asm != asm::movesr {
                catch_unwind(|| asm(AM::Immediate(0))).unwrap_err();
            }
        }
    }
}

/// CLR, NEG, NEGX, NOT, TST
fn size_effective_address(asm: fn(Size, AM) -> Vec<u16>, opcode: u16) {
    for size in [Byte, Word, Long] {
        for reg in 0..10 {
            let bew = BEW::new(true, 0, true, 0);
            let opcode = opcode | Into::<u16>::into(size) << 6;

            if reg <= 7 {
                assert_eq!(&asm(size, AM::Drd(reg)), &assemble_am(opcode, AM::Drd(reg)));
                assert_eq!(&asm(size, AM::Ari(reg)), &assemble_am(opcode, AM::Ari(reg)));
                assert_eq!(&asm(size, AM::Ariwpo(reg)), &assemble_am(opcode, AM::Ariwpo(reg)));
                assert_eq!(&asm(size, AM::Ariwpr(reg)), &assemble_am(opcode, AM::Ariwpr(reg)));
                assert_eq!(&asm(size, AM::Ariwd(reg, 0)), &assemble_am(opcode, AM::Ariwd(reg, 0)));
                assert_eq!(&asm(size, AM::Ariwi8(reg, bew)), &assemble_am(opcode, AM::Ariwi8(reg, bew)));
                assert_eq!(&asm(size, AM::AbsShort(0xFF00)), &assemble_am(opcode, AM::AbsShort(0xFF00)));
                assert_eq!(&asm(size, AM::AbsLong(0xFF_0000)), &assemble_am(opcode, AM::AbsLong(0xFF_0000)));

                catch_unwind(|| asm(size, AM::Ard(reg))).unwrap_err();
                catch_unwind(|| asm(size, AM::Pciwd(0, 0))).unwrap_err();
                catch_unwind(|| asm(size, AM::Pciwi8(0, bew))).unwrap_err();
                catch_unwind(|| asm(size, AM::Immediate(0))).unwrap_err();
            } else {
                catch_unwind(|| asm(size, AM::Drd(reg))).unwrap_err();
                catch_unwind(|| asm(size, AM::Ari(reg))).unwrap_err();
                catch_unwind(|| asm(size, AM::Ariwpo(reg))).unwrap_err();
                catch_unwind(|| asm(size, AM::Ariwpr(reg))).unwrap_err();
                catch_unwind(|| asm(size, AM::Ariwd(reg, 0))).unwrap_err();
                catch_unwind(|| asm(size, AM::Ariwi8(reg, bew))).unwrap_err();
                catch_unwind(|| asm(size, AM::Pciwd(0, 0))).unwrap_err();
                catch_unwind(|| asm(size, AM::Pciwi8(0, bew))).unwrap_err();
                catch_unwind(|| asm(size, AM::Immediate(0))).unwrap_err();
            }
        }
    }
}

/// CHK, DIVS, DIVU, LEA, MULS, MULU
fn register_effective_address(asm: fn(u8, AM) -> Vec<u16>, opcode: u16) {
    for reg in 0..10 {
        let bew = BEW::new(true, 0, true, 0);
        let opcode = opcode | (reg as u16) << 9;

        if reg <= 7 {
            assert_eq!(&asm(reg, AM::Ari(reg)), &assemble_am(opcode, AM::Ari(reg)));
            if asm != asm::lea {
                assert_eq!(&asm(reg, AM::Drd(reg)), &assemble_am(opcode, AM::Drd(reg)));
                assert_eq!(&asm(reg, AM::Ariwpo(reg)), &assemble_am(opcode, AM::Ariwpo(reg)));
                assert_eq!(&asm(reg, AM::Ariwpr(reg)), &assemble_am(opcode, AM::Ariwpr(reg)));
            } else {
                catch_unwind(|| asm(reg, AM::Drd(reg))).unwrap_err();
                catch_unwind(|| asm(reg, AM::Ariwpo(reg))).unwrap_err();
                catch_unwind(|| asm(reg, AM::Ariwpr(reg))).unwrap_err();
            }
            assert_eq!(&asm(reg, AM::Ariwd(reg, 0)), &assemble_am(opcode, AM::Ariwd(reg, 0)));
            assert_eq!(&asm(reg, AM::Ariwi8(reg, bew)), &assemble_am(opcode, AM::Ariwi8(reg, bew)));
            assert_eq!(&asm(reg, AM::AbsShort(0xFF00)), &assemble_am(opcode, AM::AbsShort(0xFF00)));
            assert_eq!(&asm(reg, AM::AbsLong(0xFF_0000)), &assemble_am(opcode, AM::AbsLong(0xFF_0000)));

            assert_eq!(&asm(reg, AM::Pciwd(0, 0)), &assemble_am(opcode, AM::Pciwd(0, 0)));
            assert_eq!(&asm(reg, AM::Pciwi8(0, bew)), &assemble_am(opcode, AM::Pciwi8(0, bew)));
            if asm != asm::lea {
                assert_eq!(&asm(reg, AM::Immediate(0xFF00)), &assemble_am(opcode, AM::Immediate(0xFF00)));
            } else {
                catch_unwind(|| asm(reg, AM::Immediate(0xFF00))).unwrap_err();
            }
            catch_unwind(|| asm(reg, AM::Ard(reg))).unwrap_err();
        } else {
            catch_unwind(|| asm(reg, AM::Drd(reg))).unwrap_err();
            catch_unwind(|| asm(reg, AM::Ari(reg))).unwrap_err();
            catch_unwind(|| asm(reg, AM::Ariwpo(reg))).unwrap_err();
            catch_unwind(|| asm(reg, AM::Ariwpr(reg))).unwrap_err();
            catch_unwind(|| asm(reg, AM::Ariwd(reg, 0))).unwrap_err();
            catch_unwind(|| asm(reg, AM::Ariwi8(reg, bew))).unwrap_err();
            catch_unwind(|| asm(reg, AM::Pciwd(0, 0))).unwrap_err();
            catch_unwind(|| asm(reg, AM::Pciwi8(0, bew))).unwrap_err();
            catch_unwind(|| asm(reg, AM::Immediate(0))).unwrap_err();
        }
    }
}

/// ADDQ, SUBQ
fn data_size_effective_address(asm: fn(u8, Size, AM) -> Vec<u16>, opcode: u16) {
    for data in 0..10 {
        for size in [Byte, Word, Long] {
            let bew = BEW::new(true, 0, true, 0);
            if data >= 1 && data <= 8 {
                let reg = if data != 8 { data } else { 0 };
                let opcode = opcode |
                    (reg as u16) << 9 |
                    Into::<u16>::into(size) << 6;

                assert_eq!(&asm(data, size, AM::Drd(reg)), &assemble_am(opcode, AM::Drd(reg)));
                if !size.is_byte() {
                    assert_eq!(&asm(data, size, AM::Ard(reg)), &assemble_am(opcode, AM::Ard(reg)));
                }
                assert_eq!(&asm(data, size, AM::Ari(reg)), &assemble_am(opcode, AM::Ari(reg)));
                assert_eq!(&asm(data, size, AM::Ariwpo(reg)), &assemble_am(opcode, AM::Ariwpo(reg)));
                assert_eq!(&asm(data, size, AM::Ariwpr(reg)), &assemble_am(opcode, AM::Ariwpr(reg)));
                assert_eq!(&asm(data, size, AM::Ariwd(reg, 0)), &assemble_am(opcode, AM::Ariwd(reg, 0)));
                assert_eq!(&asm(data, size, AM::Ariwi8(reg, bew)), &assemble_am(opcode, AM::Ariwi8(reg, bew)));
                assert_eq!(&asm(data, size, AM::AbsShort(0xFF00)), &assemble_am(opcode, AM::AbsShort(0xFF00)));
                assert_eq!(&asm(data, size, AM::AbsLong(0xFF_0000)), &assemble_am(opcode, AM::AbsLong(0xFF_0000)));
            } else {
                catch_unwind(|| asm(data, size, AM::Drd(0))).unwrap_err();
                catch_unwind(|| asm(data, Byte, AM::Ard(0))).unwrap_err();
                catch_unwind(|| asm(data, size, AM::Ari(0))).unwrap_err();
                catch_unwind(|| asm(data, size, AM::Ariwpo(0))).unwrap_err();
                catch_unwind(|| asm(data, size, AM::Ariwpr(0))).unwrap_err();
                catch_unwind(|| asm(data, size, AM::Ariwd(0, 0))).unwrap_err();
                catch_unwind(|| asm(data, size, AM::Ariwi8(0, bew))).unwrap_err();
                catch_unwind(|| asm(data, size, AM::AbsShort(0))).unwrap_err();
                catch_unwind(|| asm(data, size, AM::AbsLong(0))).unwrap_err();
                catch_unwind(|| asm(data, size, AM::Pciwd(0, 0))).unwrap_err();
                catch_unwind(|| asm(data, size, AM::Pciwi8(0, bew))).unwrap_err();
                catch_unwind(|| asm(data, size, AM::Immediate(0))).unwrap_err();
            }
        }
    }
}

/// ADD, AND, CMP, EOR, OR, SUB
fn register_direction_size_effective_address(asm: fn(u8, Direction, Size, AM) -> Vec<u16>, opcode: u16, dirs: &[Direction]) {
    for reg in 0..10 {
        for dir in ALL_DIRECTIONS {
            for size in [Byte, Word, Long] {
                let bew = BEW::new(true, 7 - reg, true, -16);
                if reg <= 7 &&
                   dirs.contains(&dir) {
                    let opcode = opcode |
                        if dir == DstEa { 1 << 8 } else { 0 } |
                        Into::<u16>::into(size) << 6 |
                        (reg as u16) << 9;

                    if dir == DstReg {
                        assert_eq!(asm(reg, dir, size, AM::Drd(reg)).as_slice(), &[opcode | reg as u16]);
                        if size != Byte &&
                           opcode & 0xF000 != 0xC000 && // AND has not this addressing mode.
                           opcode & 0xF000 != 0x8000 { // OR has not this addressing mode.
                            assert_eq!(asm(reg, dir, size, AM::Ard(reg)).as_slice(), &[opcode | 0x8 | reg as u16]);
                        }
                    }
                    assert_eq!(asm(reg, dir, size, AM::Ari(reg)).as_slice(), &assemble_am(opcode, AM::Ari(reg)));
                    assert_eq!(asm(reg, dir, size, AM::Ariwpo(reg)).as_slice(), &assemble_am(opcode, AM::Ariwpo(reg)));
                    assert_eq!(asm(reg, dir, size, AM::Ariwpr(reg)).as_slice(), &assemble_am(opcode, AM::Ariwpr(reg)));
                    assert_eq!(asm(reg, dir, size, AM::Ariwd(reg, 12)).as_slice(), &assemble_am(opcode, AM::Ariwd(reg, 12)));
                    assert_eq!(asm(reg, dir, size, AM::Ariwi8(reg, bew)).as_slice(), &assemble_am(opcode, AM::Ariwi8(reg, bew)));
                    assert_eq!(asm(reg, dir, size, AM::AbsShort(0xFF00)).as_slice(), &assemble_am(opcode, AM::AbsShort(0xFF00)));
                    assert_eq!(asm(reg, dir, size, AM::AbsLong(0xFF_0000)).as_slice(), &assemble_am(opcode, AM::AbsLong(0xFF_0000)));
                    if dir == DstReg {
                        assert_eq!(asm(reg, dir, size, AM::Pciwd(0, 12)).as_slice(), &assemble_am(opcode, AM::Pciwd(0, 12)));
                        assert_eq!(asm(reg, dir, size, AM::Pciwi8(0, bew)).as_slice(), &assemble_am(opcode, AM::Pciwi8(0, bew)));
                        assert_eq!(asm(reg, dir, size, AM::Immediate(16)).as_slice(), &assemble_am_immediate(opcode, 16, size.is_long()));
                    }
                } else {
                    if dirs.len() == 2 {
                        catch_unwind(|| asm(reg, dir, size, AM::Ari(reg))).unwrap_err();
                        catch_unwind(|| asm(reg, dir, size, AM::Ariwpo(reg))).unwrap_err();
                        catch_unwind(|| asm(reg, dir, size, AM::Ariwpr(reg))).unwrap_err();
                        catch_unwind(|| asm(reg, dir, size, AM::Ariwd(reg, -13))).unwrap_err();
                        catch_unwind(|| asm(reg, dir, size, AM::Ariwi8(reg, bew))).unwrap_err();
                        catch_unwind(|| asm(reg, dir, size, AM::AbsShort(14))).unwrap_err();
                        catch_unwind(|| asm(reg, dir, size, AM::AbsLong(14))).unwrap_err();
                        catch_unwind(|| asm(reg, dir, size, AM::Pciwd(0, -14))).unwrap_err();
                        catch_unwind(|| asm(reg, dir, size, AM::Pciwi8(0, bew))).unwrap_err();
                        catch_unwind(|| asm(reg, dir, size, AM::Immediate(12))).unwrap_err();
                    }
                }
            }
        }
    }
}

/// ADDA, CMPA, SUBA
fn register_size_effective_address(asm: fn(u8, Size, AM) -> Vec<u16>, opcode: u16) {
    for reg in 0..10 {
        for size in [Byte, Word, Long] {
            let bew = BEW::new(true, 7 - reg, true, -16);
            if reg <= 7 &&
               (size == Word || size == Long) {
                let opcode = opcode | 0x00C0 |
                    if size == Long { 1 << 8 } else { 0 } |
                    (reg as u16) << 9;

                assert_eq!(asm(reg, size, AM::Ari(reg)).as_slice(), &assemble_am(opcode, AM::Ari(reg)));
                assert_eq!(asm(reg, size, AM::Ariwpo(reg)).as_slice(), &assemble_am(opcode, AM::Ariwpo(reg)));
                assert_eq!(asm(reg, size, AM::Ariwpr(reg)).as_slice(), &assemble_am(opcode, AM::Ariwpr(reg)));
                assert_eq!(asm(reg, size, AM::Ariwd(reg, 12)).as_slice(), &assemble_am(opcode, AM::Ariwd(reg, 12)));
                assert_eq!(asm(reg, size, AM::Ariwi8(reg, bew)).as_slice(), &assemble_am(opcode, AM::Ariwi8(reg, bew)));
                assert_eq!(asm(reg, size, AM::AbsShort(0xFF00)).as_slice(), &assemble_am(opcode, AM::AbsShort(0xFF00)));
                assert_eq!(asm(reg, size, AM::AbsLong(0xFF_0000)).as_slice(), &assemble_am(opcode, AM::AbsLong(0xFF_0000)));
                assert_eq!(asm(reg, size, AM::Pciwd(0, 12)).as_slice(), &assemble_am(opcode, AM::Pciwd(0, 12)));
                assert_eq!(asm(reg, size, AM::Pciwi8(0, bew)).as_slice(), &assemble_am(opcode, AM::Pciwi8(0, bew)));
            } else {
                catch_unwind(|| asm(reg, size, AM::Ari(reg))).unwrap_err();
                catch_unwind(|| asm(reg, size, AM::Ariwpo(reg))).unwrap_err();
                catch_unwind(|| asm(reg, size, AM::Ariwpr(reg))).unwrap_err();
                catch_unwind(|| asm(reg, size, AM::Ariwd(reg, -13))).unwrap_err();
                catch_unwind(|| asm(reg, size, AM::Ariwi8(reg, bew))).unwrap_err();
                catch_unwind(|| asm(reg, size, AM::AbsShort(14))).unwrap_err();
                catch_unwind(|| asm(reg, size, AM::AbsLong(14))).unwrap_err();
                catch_unwind(|| asm(reg, size, AM::Pciwd(0, -14))).unwrap_err();
                catch_unwind(|| asm(reg, size, AM::Pciwi8(0, bew))).unwrap_err();
                catch_unwind(|| asm(reg, size, AM::Immediate(12))).unwrap_err();
            }
        }
    }
}

/// ASm, LSm, ROm, ROXm
fn direction_effective_address(asm: fn(Direction, AM) -> Vec<u16>, opcode: u16) {
    for dir in ALL_DIRECTIONS {
        for reg in 0..10 {
            if reg <= 7 &&
               (dir == Left || dir == Right) {
                let opcode = opcode | if dir == Left { 1 << 8 } else { 0 };
                let bew = BEW::new(true, 7 - reg, true, -16);

                assert_eq!(asm(dir, AM::Ari(reg)).as_slice(), &assemble_am(opcode, AM::Ari(reg)));
                assert_eq!(asm(dir, AM::Ariwpo(reg)).as_slice(), &assemble_am(opcode, AM::Ariwpo(reg)));
                assert_eq!(asm(dir, AM::Ariwpr(reg)).as_slice(), &assemble_am(opcode, AM::Ariwpr(reg)));
                assert_eq!(asm(dir, AM::Ariwd(reg, -13)).as_slice(), &assemble_am(opcode, AM::Ariwd(reg, -13)));
                assert_eq!(asm(dir, AM::Ariwi8(reg, bew)).as_slice(), &assemble_am(opcode, AM::Ariwi8(reg, bew)));
                assert_eq!(asm(dir, AM::AbsShort(0x4321)).as_slice(), &assemble_am(opcode, AM::AbsShort(0x4321)));
                assert_eq!(asm(dir, AM::AbsLong(0x76543210)).as_slice(), &assemble_am(opcode, AM::AbsLong(0x76543210)));
            } else {
                let bew = BEW::new(true, 7 - reg, true, -16);

                catch_unwind(|| asm(dir, AM::Ari(reg))).unwrap_err();
                catch_unwind(|| asm(dir, AM::Ariwpo(reg))).unwrap_err();
                catch_unwind(|| asm(dir, AM::Ariwpr(reg))).unwrap_err();
                catch_unwind(|| asm(dir, AM::Ariwd(reg, -13))).unwrap_err();
                catch_unwind(|| asm(dir, AM::Ariwi8(reg, bew))).unwrap_err();
                if dir != Left && dir != Right {
                    catch_unwind(|| asm(dir, AM::AbsShort(14))).unwrap_err();
                    catch_unwind(|| asm(dir, AM::AbsLong(14))).unwrap_err();
                }
                catch_unwind(|| asm(dir, AM::Pciwd(0, -14))).unwrap_err();
                catch_unwind(|| asm(dir, AM::Pciwi8(0, bew))).unwrap_err();
                catch_unwind(|| asm(dir, AM::Immediate(12))).unwrap_err();
            }
        }
    }
}

/// ASr, LSr, ROr, ROXr
fn rotation_direction_size_mode_register(asm: fn(u16, Direction, Size, bool, u16) -> u16, opcode: u16) {
    for count_reg in 0..10 {
        for dir in ALL_DIRECTIONS {
            for size in [Byte, Word, Long] {
                for ir in [false, true] {
                    for reg in 0..10 {
                        if count_reg <= 7 && reg <= 7 &&
                           (dir == Left || dir == Right) {
                            let opcode = opcode |
                                (count_reg as u16) << 9 |
                                if dir == Left { 1 << 8 } else { 0 } |
                                Into::<u16>::into(size) << 6 |
                                if ir { 1 << 5 } else { 0 } |
                                reg as u16;
                            assert_eq!(asm(count_reg, dir, size, ir, reg), opcode);
                        } else {
                            catch_unwind(|| asm(count_reg, dir, size, ir, reg)).unwrap_err();
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn assembler_abcd() {
    for dstreg in 0..10 {
        for mode in ALL_DIRECTIONS {
            for srcreg in 0..10 {
                if dstreg <= 7 && srcreg <= 7 &&
                   (mode == RegisterToRegister || mode == MemoryToMemory) {
                    let abcd = 0xC100 |
                        (dstreg as u16 & 7) << 9 |
                        if mode == MemoryToMemory { 1 << 3 } else { 0 } |
                        (srcreg as u16 & 7);
                    assert_eq!(asm::abcd(dstreg, mode, srcreg), abcd);
                } else {
                    catch_unwind(|| asm::abcd(dstreg, mode, srcreg)).unwrap_err();
                }
            }
        }
    }
}

#[test]
fn assembler_add() {
    register_direction_size_effective_address(asm::add, 0xD000, &[DstEa, DstReg]);
}

#[test]
fn assembler_adda() {
    register_size_effective_address(asm::adda, 0xD000);
}

#[test]
fn assembler_addi() {
    size_effective_address_immediate(asm::addi, 0x0600);
}

#[test]
fn assembler_addq() {
    data_size_effective_address(asm::addq, 0x5000);
}

#[test]
fn assembler_addx() {
    for dstreg in 0..10 {
        for size in [Byte, Word, Long] {
            for mode in ALL_DIRECTIONS {
                for srcreg in 0..10 {
                    if dstreg <= 7 && srcreg <= 7 &&
                       (mode == RegisterToRegister || mode == MemoryToMemory) {
                        let addx = 0xD100 |
                            (dstreg as u16 & 7) << 9 |
                            Into::<u16>::into(size) << 6 |
                            if mode == MemoryToMemory { 1 << 3 } else { 0 } |
                            (srcreg as u16 & 7);
                        assert_eq!(asm::addx(dstreg, size, mode, srcreg), addx);
                    } else {
                        catch_unwind(|| asm::addx(dstreg, size, mode, srcreg)).unwrap_err();
                    }
                }
            }
        }
    }
}

#[test]
fn assembler_and() {
    register_direction_size_effective_address(asm::and, 0xC000, &[DstEa, DstReg]);
}

#[test]
fn assembler_andi() {
    size_effective_address_immediate(asm::andi, 0x0200);
}

#[test]
fn assembler_andiccr() {
    for imm in 0..=u16::MAX {
        assert_eq!(asm::andiccr(imm), [0x023C, imm & 0x00FF]);
    }
}

#[test]
fn assembler_andisr() {
    for imm in 0..=u16::MAX {
        assert_eq!(asm::andisr(imm), [0x027C, imm]);
    }
}

#[test]
fn assembler_asm() {
    direction_effective_address(asm::asm, 0xE0C0);
}

#[test]
fn assembler_asr() {
    rotation_direction_size_mode_register(asm::asr, 0xE000);
}

#[test]
fn assembler_bcc() {
    for cc in 0..16 {
        if cc >= 2 {
            let bcc = 0x6000 | cc << 8;
            assert_eq!(asm::bcc(get_condition(cc), 0).as_slice(), &[bcc, 0]);
            assert_eq!(asm::bcc(get_condition(cc), -10).as_slice(), &[bcc | -10i8 as u8 as u16]);
            assert_eq!(asm::bcc(get_condition(cc), -30_000).as_slice(), &[bcc, -30_000i16 as u16]);
        } else {
            catch_unwind(|| asm::bcc(get_condition(cc), 0)).unwrap_err();
            catch_unwind(|| asm::bcc(get_condition(cc), -10)).unwrap_err();
            catch_unwind(|| asm::bcc(get_condition(cc), -30_000)).unwrap_err();
        }
    }
}

#[test]
fn assembler_bchg() {
    effective_address_count(asm::bchg_dynamic, asm::bchg_static, 0x0040);
}

#[test]
fn assembler_bclr() {
    effective_address_count(asm::bclr_dynamic, asm::bclr_static, 0x0080);
}

#[test]
fn assembler_bra() {
    assert_eq!(asm::bra(0).as_slice(), &[0x6000, 0]);
    assert_eq!(asm::bra(-10).as_slice(), &[0x6000 | -10i8 as u8 as u16]);
    assert_eq!(asm::bra(-30_000).as_slice(), &[0x6000, -30_000i16 as u16]);
}

#[test]
fn assembler_bset() {
    effective_address_count(asm::bset_dynamic, asm::bset_static, 0x00C0);
}

#[test]
fn assembler_bsr() {
    assert_eq!(asm::bsr(0).as_slice(), &[0x6100, 0]);
    assert_eq!(asm::bsr(-10).as_slice(), &[0x6100 | -10i8 as u8 as u16]);
    assert_eq!(asm::bsr(-30_000).as_slice(), &[0x6100, -30_000i16 as u16]);
}

#[test]
fn assembler_btst() {
    effective_address_count(asm::btst_dynamic, asm::btst_static, 0x0000);
}

#[test]
fn assembler_chk() {
    register_effective_address(asm::chk, 0x4180);
}

#[test]
fn assembler_clr() {
    size_effective_address(asm::clr, 0x4200);
}

#[test]
fn assembler_cmp() {
    let cmp = |reg: u8, _: Direction, size: Size, am: AM| asm::cmp(reg, size, am);
    register_direction_size_effective_address(cmp, 0xB000, &[DstReg]);
}

#[test]
fn assembler_cmpa() {
    register_size_effective_address(asm::cmpa, 0xB000);
}

#[test]
fn assembler_cmpi() {
    size_effective_address_immediate(asm::cmpi, 0x0C00);
}

#[test]
fn assembler_cmpm() {
    for dstreg in 0..10 {
        for size in [Byte, Word, Long] {
            for srcreg in 0..10 {
                if dstreg <= 7 && srcreg <= 7 {
                    let cmpm = 0xB108 |
                        (dstreg as u16) << 9 |
                        Into::<u16>::into(size) << 6 |
                        srcreg as u16;
                    assert_eq!(asm::cmpm(dstreg, size, srcreg), cmpm);
                } else {
                    catch_unwind(|| asm::cmpm(dstreg, size, srcreg)).unwrap_err();
                }
            }
        }
    }
}

#[test]
fn assembler_dbcc() {
    for cc in 0..16 {
        for reg in 0..10 {
            if reg <= 7 {
                let dbcc = 0x50C8 |
                    (cc as u16) << 8 |
                    reg as u16;
                assert_eq!(asm::dbcc(get_condition(cc), reg, -30_000), [dbcc, -30_000i16 as u16]);
            } else {
                catch_unwind(|| asm::dbcc(get_condition(cc), reg, 0)).unwrap_err();
            }
        }
    }
}

#[test]
fn assembler_divs() {
    register_effective_address(asm::divs, 0x81C0);
}

#[test]
fn assembler_divu() {
    register_effective_address(asm::divu, 0x80C0);
}

#[test]
fn assembler_eor() {
    let eor = |reg: u8, _: Direction, size: Size, am: AM| asm::eor(reg, size, am);
    register_direction_size_effective_address(eor, 0xB000, &[DstEa]);
}

#[test]
fn assembler_eori() {
    size_effective_address_immediate(asm::eori, 0x0A00);
}

#[test]
fn assembler_eoriccr() {
    for imm in 0..=u16::MAX {
        assert_eq!(asm::eoriccr(imm), [0x0A3C, imm & 0x00FF]);
    }
}

#[test]
fn assembler_eorisr() {
    for imm in 0..=u16::MAX {
        assert_eq!(asm::eorisr(imm), [0x0A7C, imm]);
    }
}

#[test]
fn assembler_exg() {
    for rx in 0..10 {
        for dir in ALL_DIRECTIONS {
            for ry in 0..10 {
                if rx <= 7 && ry <= 7 &&
                   (dir == ExchangeData || dir == ExchangeAddress || dir == ExchangeDataAddress) {
                    let exg = 0xC100 |
                        (rx as u16) << 9 |
                        if dir == ExchangeData { 0b01000 }
                        else if dir == ExchangeAddress { 0b01001 }
                        else { 0b10001 } << 3 |
                        ry as u16;
                    assert_eq!(asm::exg(rx, dir, ry), exg);
                } else {
                    catch_unwind(|| asm::exg(rx, dir, ry)).unwrap_err();
                }
            }
        }
    }
}

#[test]
fn assembler_ext() {
    for word_to_long in [false, true] {
        for reg in 0..10 {
            if reg <= 7 {
                let ext = 0x4880 |
                    if word_to_long { 1 << 6 } else { 0 } |
                    reg as u16;
                assert_eq!(asm::ext(word_to_long, reg), ext);
            } else {
                catch_unwind(|| asm::ext(word_to_long, reg)).unwrap_err();
            }
        }
    }
}

#[test]
fn assembler_illegal() {
    assert_eq!(asm::illegal(), 0x4AFC);
}

#[test]
fn assembler_jmp() {
    effective_address(asm::jmp, 0x4EC0);
}

#[test]
fn assembler_jsr() {
    effective_address(asm::jsr, 0x4E80);
}

#[test]
fn assembler_lea() {
    register_effective_address(asm::lea, 0x41C0);
}

#[test]
fn assembler_link() {
    for reg in 0..10 {
        if reg <= 7 {
            let link = 0x4E50 | reg as u16;
            assert_eq!(asm::link(reg, -13), [link, -13i16 as u16]);
        } else {
            catch_unwind(|| asm::link(reg, 0)).unwrap_err();
        }
    }
}

#[test]
fn assembler_lsm() {
    direction_effective_address(asm::lsm, 0xE2C0);
}

#[test]
fn assembler_lsr() {
    rotation_direction_size_mode_register(asm::lsr, 0xE008);
}

#[test]
fn assembler_move() {
    for size in [Byte, Word, Long] {
        for reg in 0..10 {
            if reg <= 7 {
                let bew = BEW::new(false, 7 - reg, false, 15);
                let r#move = size.into_move() << 12;

                assert_eq!(asm::r#move(size, AM::Drd(7 - reg), AM::Drd(reg)).as_slice(), &assemble_move_am(r#move, AM::Drd(7 - reg), AM::Drd(reg)));
                if !size.is_byte() {
                    assert_eq!(asm::r#move(size, AM::Drd(7 - reg), AM::Ard(reg)).as_slice(), &assemble_move_am(r#move, AM::Drd(7 - reg), AM::Ard(reg)));
                } else {
                    catch_unwind(|| asm::r#move(size, AM::Drd(7 - reg), AM::Ard(reg))).unwrap_err();
                }
                assert_eq!(asm::r#move(size, AM::Ari(7 - reg), AM::Ari(reg)).as_slice(), &assemble_move_am(r#move, AM::Ari(7 - reg), AM::Ari(reg)));
                assert_eq!(asm::r#move(size, AM::Ariwd(7 - reg, 30), AM::Ariwd(reg, -30)).as_slice(), &assemble_move_am(r#move, AM::Ariwd(7 - reg, 30), AM::Ariwd(reg, -30)));
                assert_eq!(asm::r#move(size, AM::Ariwi8(7 - reg, bew), AM::Ariwi8(reg, bew)).as_slice(), &assemble_move_am(r#move, AM::Ariwi8(7 - reg, bew), AM::Ariwi8(reg, bew)));
                assert_eq!(asm::r#move(size, AM::AbsShort(0xFF00), AM::AbsShort(0xFF00)).as_slice(), &assemble_move_am(r#move, AM::AbsShort(0xFF00), AM::AbsShort(0xFF00)));
                assert_eq!(asm::r#move(size, AM::AbsLong(0xFF_0000), AM::AbsLong(0xFF_0000)).as_slice(), &assemble_move_am(r#move, AM::AbsLong(0xFF_0000), AM::AbsLong(0xFF_0000)));
                assert_eq!(asm::r#move(size, AM::Drd(7 - reg), AM::Pciwd(0, 30)).as_slice(), &assemble_move_am(r#move, AM::Drd(7 - reg), AM::Pciwd(0, 30)));
                assert_eq!(asm::r#move(size, AM::Drd(7 - reg), AM::Pciwi8(0, bew)).as_slice(), &assemble_move_am(r#move, AM::Drd(7 - reg), AM::Pciwi8(0, bew)));
                assert_eq!(asm::r#move(size, AM::Drd(7 - reg), AM::Immediate(43)).as_slice(), &assemble_move_am_immediate(r#move, AM::Drd(7 - reg), 43, size.is_long()));

                catch_unwind(|| asm::r#move(size, AM::Pciwd(0, 0), AM::Drd(0))).unwrap_err();
                catch_unwind(|| asm::r#move(size, AM::Pciwi8(0, bew), AM::Drd(0))).unwrap_err();
                catch_unwind(|| asm::r#move(size, AM::Immediate(0), AM::Drd(0))).unwrap_err();
            } else {
                catch_unwind(|| asm::r#move(size, AM::Drd(7 - reg), AM::Drd(reg))).unwrap_err();
            }
        }
    }
}

#[test]
fn assembler_movea() {
    for size in [Byte, Word, Long] {
        for reg in 0..10 {
            if !size.is_byte() && reg <= 7 {
                let bew = BEW::new(false, 7 - reg, false, 15);
                let movea = 0x0040 |
                    if size.is_word() { 3 } else { 2 } << 12 |
                    (reg as u16) << 9;

                assert_eq!(asm::movea(size, reg, AM::Drd(reg)).as_slice(), &assemble_am(movea, AM::Drd(reg)));
                assert_eq!(asm::movea(size, reg, AM::Ard(reg)).as_slice(), &assemble_am(movea, AM::Ard(reg)));
                assert_eq!(asm::movea(size, reg, AM::Ari(reg)).as_slice(), &assemble_am(movea, AM::Ari(reg)));
                assert_eq!(asm::movea(size, reg, AM::Ariwd(reg, -30)).as_slice(), &assemble_am(movea, AM::Ariwd(reg, -30)));
                assert_eq!(asm::movea(size, reg, AM::Ariwi8(reg, bew)).as_slice(), &assemble_am(movea, AM::Ariwi8(reg, bew)));
                assert_eq!(asm::movea(size, reg, AM::AbsShort(0xFF00)).as_slice(), &assemble_am(movea, AM::AbsShort(0xFF00)));
                assert_eq!(asm::movea(size, reg, AM::AbsLong(0xFF_0000)).as_slice(), &assemble_am(movea, AM::AbsLong(0xFF_0000)));
                assert_eq!(asm::movea(size, reg, AM::Pciwd(0, 30)).as_slice(), &assemble_am(movea, AM::Pciwd(0, 30)));
                assert_eq!(asm::movea(size, reg, AM::Pciwi8(0, bew)).as_slice(), &assemble_am(movea, AM::Pciwi8(0, bew)));
                assert_eq!(asm::movea(size, reg, AM::Immediate(43)).as_slice(), &assemble_am_immediate(movea, 43, size.is_long()));
            } else {
                catch_unwind(|| asm::movea(size, reg, AM::Drd(0))).unwrap_err();
            }
        }
    }
}

#[test]
fn assembler_moveccr() {
    effective_address(asm::moveccr, 0x44C0);
}

#[test]
fn assembler_movefsr() {
    effective_address(asm::movefsr, 0x40C0);
}

#[test]
fn assembler_movesr() {
    effective_address(asm::movesr, 0x46C0);
}

#[test]
fn assembler_moveusp() {
    for reg in 0..10 {
        for dir in ALL_DIRECTIONS {
            if reg <= 7 &&
               (dir == UspToRegister || dir == RegisterToUsp) {
                let moveusp = 0x4E60 |
                    if dir == UspToRegister { 1 << 3 } else { 0 } |
                    reg as u16;
                assert_eq!(asm::moveusp(dir, reg), moveusp);
            } else {
                catch_unwind(|| asm::moveusp(dir, reg)).unwrap_err();
            }
        }
    }
}

#[test]
fn assembler_movem() {
    for dir in ALL_DIRECTIONS {
        for size in [Byte, Word, Long] {
            let bew = BEW::new(true, 0, true, 0);
            if !size.is_byte() &&
               (dir == RegisterToMemory || dir == MemoryToRegister) {
                let movem = 0x4880 |
                    if dir == MemoryToRegister { 1 << 10 } else { 0 } |
                    if size.is_long() { 1 << 6 } else { 0 };

                assert_eq!(&asm::movem(dir, size, AM::Ari(0), 0xFF), &immediate_assemble_am(movem, 0xFF, AM::Ari(0)));
                if dir == MemoryToRegister {
                    assert_eq!(&asm::movem(dir, size, AM::Ariwpo(0), 0xFF), &immediate_assemble_am(movem, 0xFF, AM::Ariwpo(0)));
                    assert_eq!(&asm::movem(dir, size, AM::Pciwd(0, 12), 0xFF), &immediate_assemble_am(movem, 0xFF, AM::Pciwd(0, 12)));
                    assert_eq!(&asm::movem(dir, size, AM::Pciwi8(0, bew), 0xFF), &immediate_assemble_am(movem, 0xFF, AM::Pciwi8(0, bew)));
                } else {
                    assert_eq!(&asm::movem(dir, size, AM::Ariwpr(0), 0xFF), &immediate_assemble_am(movem, 0xFF, AM::Ariwpr(0)));
                }
                assert_eq!(&asm::movem(dir, size, AM::Ariwd(0, 12), 0xFF), &immediate_assemble_am(movem, 0xFF, AM::Ariwd(0, 12)));
                assert_eq!(&asm::movem(dir, size, AM::Ariwi8(0, bew), 0xFF), &immediate_assemble_am(movem, 0xFF, AM::Ariwi8(0, bew)));
                assert_eq!(&asm::movem(dir, size, AM::AbsShort(0xFF00), 0xFF), &immediate_assemble_am(movem, 0xFF, AM::AbsShort(0xFF00)));
                assert_eq!(&asm::movem(dir, size, AM::AbsLong(0xFF_0000), 0xFF), &immediate_assemble_am(movem, 0xFF, AM::AbsLong(0xFF_0000)));
            } else {
                catch_unwind(|| asm::movem(dir, size, AM::Drd(0), 0)).unwrap_err();
                catch_unwind(|| asm::movem(dir, size, AM::Ard(0), 0)).unwrap_err();
                catch_unwind(|| asm::movem(dir, size, AM::Ari(0), 0)).unwrap_err();
                catch_unwind(|| asm::movem(dir, size, AM::Ariwd(0, 0), 0)).unwrap_err();
                catch_unwind(|| asm::movem(dir, size, AM::Ariwi8(0, bew), 0)).unwrap_err();
                catch_unwind(|| asm::movem(dir, size, AM::AbsShort(0), 0)).unwrap_err();
                catch_unwind(|| asm::movem(dir, size, AM::AbsLong(0), 0)).unwrap_err();
                catch_unwind(|| asm::movem(dir, size, AM::Pciwd(0, 0), 0)).unwrap_err();
                catch_unwind(|| asm::movem(dir, size, AM::Pciwi8(0, bew), 0)).unwrap_err();
                catch_unwind(|| asm::movem(dir, size, AM::Immediate(0), 0)).unwrap_err();
            }
        }
    }
}

#[test]
fn assembler_movep() {
    for data in 0..10 {
        for dir in ALL_DIRECTIONS {
            for size in [Byte, Word, Long] {
                for addr in 0..10 {
                    for disp in (i16::MIN..=i16::MAX).step_by(8191) {
                        if data <= 7 && addr <= 7 &&
                           (dir == MemoryToRegister || dir == RegisterToMemory) &&
                           (size == Word || size == Long) {
                            let movep = 0x0108 |
                                (data as u16) << 9 |
                                if dir == RegisterToMemory { 1 << 7 } else { 0 } |
                                if size == Long { 1 << 6 } else { 0 } |
                                addr as u16;
                            assert_eq!(asm::movep(data, dir, size, addr, disp), [movep, disp as u16]);
                        } else {
                            catch_unwind(|| asm::movep(data, dir, size, addr, disp)).unwrap_err();
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn assembler_moveq() {
    for reg in 0..10 {
        if reg <= 7 {
            let moveq = 0x70B3 | (reg as u16) << 9;
            assert_eq!(asm::moveq(reg, -77), moveq);
        } else {
            catch_unwind(|| asm::moveq(reg, 0)).unwrap_err();
        }
    }
}

#[test]
fn assembler_muls() {
    register_effective_address(asm::muls, 0xC1C0);
}

#[test]
fn assembler_mulu() {
    register_effective_address(asm::mulu, 0xC0C0);
}

#[test]
fn assembler_nbcd() {
    effective_address(asm::nbcd, 0x4800);
}

#[test]
fn assembler_neg() {
    size_effective_address(asm::neg, 0x4400);
}

#[test]
fn assembler_negx() {
    size_effective_address(asm::negx, 0x4000);
}

#[test]
fn assembler_nop() {
    assert_eq!(asm::nop(), 0x4E71);
}

#[test]
fn assembler_not() {
    size_effective_address(asm::not, 0x4600);
}

#[test]
fn assembler_or() {
    register_direction_size_effective_address(asm::or, 0x8000, &[DstEa, DstReg]);
}

#[test]
fn assembler_ori() {
    size_effective_address_immediate(asm::ori, 0x0000);
}

#[test]
fn assembler_oriccr() {
    for imm in 0..=u16::MAX {
        assert_eq!(asm::oriccr(imm), [0x003C, imm & 0x00FF]);
    }
}

#[test]
fn assembler_orisr() {
    for imm in 0..=u16::MAX {
        assert_eq!(asm::orisr(imm), [0x007C, imm]);
    }
}

#[test]
fn assembler_pea() {
    effective_address(asm::pea, 0x4840);
}

#[test]
fn assembler_reset() {
    assert_eq!(asm::reset(), 0x4E70);
}

#[test]
fn assembler_rom() {
    direction_effective_address(asm::rom, 0xE6C0);
}

#[test]
fn assembler_ror() {
    rotation_direction_size_mode_register(asm::ror, 0xE018);
}

#[test]
fn assembler_roxm() {
    direction_effective_address(asm::roxm, 0xE4C0);
}

#[test]
fn assembler_roxr() {
    rotation_direction_size_mode_register(asm::roxr, 0xE010);
}

#[test]
fn assembler_rte() {
    assert_eq!(asm::rte(), 0x4E73);
}

#[test]
fn assembler_rtr() {
    assert_eq!(asm::rtr(), 0x4E77);
}

#[test]
fn assembler_rts() {
    assert_eq!(asm::rts(), 0x4E75);
}

#[test]
fn assembler_stop() {
    for sr in 0..=u16::MAX {
        assert_eq!(asm::stop(sr), [0x4E72, sr]);
    }
}

#[test]
fn assembler_sbcd() {
    for dstreg in 0..10 {
        for mode in ALL_DIRECTIONS {
            for srcreg in 0..10 {
                if dstreg <= 7 && srcreg <= 7 &&
                   (mode == RegisterToRegister || mode == MemoryToMemory) {
                    let sbcd = 0x8100 |
                        (dstreg as u16) << 9 |
                        if mode == MemoryToMemory { 1 << 3 } else { 0 } |
                        srcreg as u16;
                    assert_eq!(asm::sbcd(dstreg, mode, srcreg), sbcd);
                } else {
                    catch_unwind(|| asm::sbcd(dstreg, mode, srcreg)).unwrap_err();
                }
            }
        }
    }
}

#[test]
fn assembler_scc() {
    for cond in 0..=0xF {
        let bew = BEW::new(true, 0, true, 0);
        let scc = 0x50C0 | cond << 8;

        assert_eq!(asm::scc(get_condition(cond), AM::Drd(0)).as_slice(), &assemble_am(scc, AM::Drd(0)));
        assert_eq!(asm::scc(get_condition(cond), AM::Ari(0)).as_slice(), &assemble_am(scc, AM::Ari(0)));
        assert_eq!(asm::scc(get_condition(cond), AM::Ariwd(0, -3)).as_slice(), &assemble_am(scc, AM::Ariwd(0, -3)));
        assert_eq!(asm::scc(get_condition(cond), AM::Ariwi8(0, bew)).as_slice(), &assemble_am(scc, AM::Ariwi8(0, bew)));
        assert_eq!(asm::scc(get_condition(cond), AM::AbsShort(0xFF00)).as_slice(), &assemble_am(scc, AM::AbsShort(0xFF00)));
        assert_eq!(asm::scc(get_condition(cond), AM::AbsLong(0xFF_0000)).as_slice(), &assemble_am(scc, AM::AbsLong(0xFF_0000)));

        catch_unwind(|| asm::scc(CC::T, AM::Ard(0))).unwrap_err();
        catch_unwind(|| asm::scc(CC::T, AM::Pciwd(0, -76))).unwrap_err();
        catch_unwind(|| asm::scc(CC::T, AM::Pciwi8(0, bew))).unwrap_err();
        catch_unwind(|| asm::scc(CC::T, AM::Immediate(0))).unwrap_err();
    }
}

#[test]
fn assembler_sub() {
    register_direction_size_effective_address(asm::sub, 0x9000, &[DstEa, DstReg]);
}

#[test]
fn assembler_suba() {
    register_size_effective_address(asm::suba, 0x9000);
}

#[test]
fn assembler_subi() {
    size_effective_address_immediate(asm::subi, 0x0400);
}

#[test]
fn assembler_subq() {
    data_size_effective_address(asm::subq, 0x5100);
}

#[test]
fn assembler_subx() {
    for dstreg in 0..10 {
        for size in [Byte, Word, Long] {
            for mode in ALL_DIRECTIONS {
                for srcreg in 0..10 {
                    if dstreg <= 7 && srcreg <= 7 &&
                       (mode == RegisterToRegister || mode == MemoryToMemory) {
                        let subx = 0x9100 |
                            (dstreg as u16) << 9 |
                            Into::<u16>::into(size) << 6 |
                            if mode == MemoryToMemory { 1 << 3 } else { 0 } |
                            srcreg as u16;
                        assert_eq!(asm::subx(dstreg, size, mode, srcreg), subx);
                    } else {
                        catch_unwind(|| asm::subx(dstreg, size, mode, srcreg)).unwrap_err();
                    }
                }
            }
        }
    }
}

#[test]
fn assembler_swap() {
    for reg in 0..10 {
        if reg <= 7 {
            let swap = 0x4840 | reg as u16;
            assert_eq!(asm::swap(reg), swap);
        } else {
            catch_unwind(|| asm::swap(reg)).unwrap_err();
        }
    }
}

#[test]
fn assembler_tas() {
    effective_address(asm::tas, 0x4AC0);
}

#[test]
fn assembler_trap() {
    for t in 0..0x1F {
        if t <= 0xF {
            let trap = 0x4E40 | t as u16;
            assert_eq!(asm::trap(t), trap);
        } else {
            catch_unwind(|| asm::trap(t)).unwrap_err();
        }
    }
}

#[test]
fn assembler_trapv() {
    assert_eq!(asm::trapv(), 0x4E76);
}

#[test]
fn assembler_tst() {
    size_effective_address(asm::tst, 0x4A00);
}

#[test]
fn assembler_unlk() {
    for reg in 0..10 {
        if reg <= 7 {
            let unlk = 0x4E58 | reg as u16;
            assert_eq!(asm::unlk(reg), unlk);
        } else {
            catch_unwind(|| asm::unlk(reg)).unwrap_err();
        }
    }
}
