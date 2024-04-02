// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Dynamically assemble M68000 instructions.
//!
//! The fields are as described in the M68000 Programming Reference Manual, left (high order bits) to right (low order bits).
//! Refer to it to know which values are valid for the instructions.
//! If a bad parameter is send to an assembler function, it panics.
//!
//! The shift/rotate instructions are regrouped by their destination location and not by their shift/rotate direction.
//! So [asm] is the arithmetic shift with the data in memory, and [asr] is the arithmetic shift with data in register.
//! The direction is specified as a parameter in these functions.

use crate::addressing_modes::AddressingMode;
use crate::instruction::{Direction, Size};

/// Conditions for the conditional instructions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Condition {
    /// True.
    T,
    /// False.
    F,
    /// High.
    HI,
    /// Lower or Same.
    LS,
    /// Carry Clear.
    CC,
    /// Carry Set.
    CS,
    /// Not Equal.
    NE,
    /// Equal.
    EQ,
    /// Overflow Clear.
    VC,
    /// Overflow Set.
    VS,
    /// Plus.
    PL,
    /// Minus.
    MI,
    /// Greater or Equal.
    GE,
    /// Less Than.
    LT,
    /// Greater Than.
    GT,
    /// Less or Equal.
    LE,
}

/// Modes 2, 5, 6 and 7.
const MODES_2567: [u8; 4] = [2, 5, 6, 7];
/// Modes 2, 3, 4, 5, 6 and 7.
const MODES_234567: [u8; 6] = [2, 3, 4, 5, 6, 7];
/// Modes 0, 2, 3, 4, 5, 6 and 7.
const MODES_0234567: [u8; 7] = [0, 2, 3, 4, 5, 6, 7];
/// Modes 0, 1, 2, 3, 4, 5, 6 and 7.
const MODES_01234567: [u8; 8] = [0, 1, 2, 3, 4, 5, 6, 7];

/// ADDI, ANDI, CMPI, EORI, ORI, SUBI
fn size_effective_address_immediate(bits8_15: u8, size: Size, am: AddressingMode, mut imm: u32) -> Vec<u16> {
    let mut vec = Vec::new();

    let (eafield, eaext) = am.assemble(size.is_long());
    let opcode = (bits8_15 as u16) << 8
               | Into::<u16>::into(size) << 6
               | eafield;
    vec.push(opcode);

    if size.is_long() {
        vec.push((imm >> 16) as u16);
    } else if size.is_byte() {
        imm &= 0x0000_00FF;
    }
    vec.push(imm as u16);
    vec.extend(eaext.iter());

    vec
}

/// static BCHG, BCLR, BSET, BTST
fn effective_address_count(bits6_15: u16, am: AddressingMode, count: u8) -> Vec<u16> {
    let mut vec = Vec::new();

    let (eafield, eaext) = am.assemble(false);
    let opcode = (bits6_15 & 0x3FF) << 6
               | eafield;
    vec.push(opcode);
    vec.push(count as u16);
    vec.extend(eaext.iter());

    vec
}

/// JMP, JSR, MOVE (f) SR CCR, NBCD, PEA, TAS
fn effective_address(bits6_15: u16, am: AddressingMode) -> Vec<u16> {
    let mut vec = Vec::new();

    let (eafield, eaext) = am.assemble(false);
    let opcode = (bits6_15 & 0x3FF) << 6
               | eafield;
    vec.push(opcode);
    vec.extend(eaext.iter());

    vec
}

/// CLR, NEG, NEGX, NOT, TST
fn size_effective_address(bits8_15: u8, size: Size, am: AddressingMode) -> Vec<u16> {
    let mut vec = Vec::new();

    let (eafield, eaext) = am.assemble(size.is_long());
    let opcode = (bits8_15 as u16) << 8
               | Into::<u16>::into(size) << 6
               | eafield;
    vec.push(opcode);
    vec.extend(eaext.iter());

    vec
}

/// dynamic BCHG, BCLR, BSET, BTST, CHK, DIVS, DIVU, LEA, MULS, MULU
fn register_effective_address(bits12_15: u16, reg: u16, bits6_8: u16, am: AddressingMode) -> Vec<u16> {
    let mut vec = Vec::new();

    let (eafield, eaext) = am.assemble(false);
    let opcode = (bits12_15 & 0xF) << 12
               | (reg & 7) << 9
               | (bits6_8 & 7) << 6
               | eafield;
    vec.push(opcode);
    vec.extend(eaext.iter());

    vec
}

/// MOVE, MOVEA
fn size_effective_address_effective_address(size: Size, dst: AddressingMode, src: AddressingMode) -> Vec<u16> {
    let mut vec = Vec::new();

    let src = src.assemble(size.is_long());
    let dst = dst.assemble_move_dst();
    let opcode = size.into_move() << 12 | dst.0 | src.0;

    vec.push(opcode);
    vec.extend(src.1.iter());
    vec.extend(dst.1.iter());

    vec
}

/// SWAP, UNLK
fn register(bits3_15: u16, reg: u8) -> u16 {
    bits3_15 << 3 | reg as u16 & 7
}

/// ADDQ, SUBQ
fn data_size_effective_address(data: u8, bit8: u16, size: Size, am: AddressingMode) -> Vec<u16> {
    let mut vec = Vec::new();

    let (eafield, eaext) = am.assemble(size.is_long());
    let opcode = 0b0101 << 12
               | (data as u16 & 7) << 9
               | (bit8 & 1) << 8
               | Into::<u16>::into(size) << 6
               | eafield;
    vec.push(opcode);
    vec.extend(eaext.iter());

    vec
}

/// Bcc, BRA, BSR
///
/// If the displacement fits in an i8 and is not 0, 1 opcode is used, otherwise 2.
fn condition_displacement(cond: Condition, disp: i16) -> Vec<u16> {
    let mut vec = Vec::new();

    let mut opcode = 0b0110 << 12 | (cond as u16) << 8;

    if disp < i8::MIN as i16 || disp > i8::MAX as i16 || disp == 0 {
        vec.push(opcode);
        vec.push(disp as u16);
    } else {
        opcode |= disp as u8 as u16;
        vec.push(opcode);
    }

    vec
}

/// ADD, AND, CMP, EOR, OR, SUB
///
/// [Direction::DstReg] or [Direction::DstEa].
fn register_direction_size_effective_address(bits12_15: u16, reg: u8, dir: Direction, size: Size, am: AddressingMode) -> Vec<u16> {
    let mut vec = Vec::new();

    let (eafield, eaext) = am.assemble(size.is_long());
    let opcode = bits12_15 << 12
               | (reg as u16) << 9
               | if dir == Direction::DstEa { 1 } else { 0 } << 8
               | Into::<u16>::into(size) << 6
               | eafield;
    vec.push(opcode);
    vec.extend(eaext.iter());

    vec
}

/// ADDA, CMPA, SUBA
fn register_size_effective_address(bits12_15: u16, reg: u8, size: Size, am: AddressingMode) -> Vec<u16> {
    let mut vec = Vec::new();

    let (eafield, eaext) = am.assemble(size.is_long());
    let opcode = bits12_15 << 12
               | (reg as u16 & 7) << 9
               | size.into_bit() << 8
               | 0b11 << 6
               | eafield;
    vec.push(opcode);
    vec.extend(eaext.iter());

    vec
}

/// ABCD, ADDX, SBCD, SUBX
///
/// [Direction::RegisterToRegister] or [Direction::MemoryToMemory].
fn register_size_mode_register(bits12_15: u16, dst: u8, size: Size, bits4_5: u16, mode: Direction, src: u8) -> u16 {
    let mut opcode = (bits12_15 & 0xF) << 12
                   | (dst as u16 & 7) << 9
                   | 1 << 8
                   | Into::<u16>::into(size) << 6
                   | (bits4_5 & 3) << 4
                   | src as u16 & 7;
    if mode == Direction::MemoryToMemory {
        opcode |= 0x0008;
    }

    opcode
}

/// ASm, LSm, ROm, ROXm
fn direction_effective_address(bits9_15: u16, dir: Direction, bits6_7: u16, am: AddressingMode) -> Vec<u16> {
    let mut vec = Vec::new();

    let (eafield, eaext) = am.assemble(false);
    let mut opcode = (bits9_15 & 0x7F) << 9
                   | (bits6_7 & 3) << 6
                   | eafield;
    if dir == Direction::Left {
        opcode |= 0x0100;
    }
    vec.push(opcode);
    vec.extend(eaext.iter());

    vec
}

/// ASr, LSr, ROr, ROXr
fn rotation_direction_size_mode_register(bits12_15: u16, count_reg: u16, dir: Direction, size: Size, ir: u16, bits3_4: u16, reg: u16) -> u16 {
    let mut opcode = (bits12_15 & 0xF) << 12
                   | (count_reg & 7) << 9
                   | Into::<u16>::into(size) << 6
                   | (ir & 1) << 5
                   | (bits3_4 & 3) << 3
                   | reg & 7;
    if dir == Direction::Left {
        opcode |= 0x0100;
    }

    opcode
}

/// `mode` must be [Direction::RegisterToRegister] or [Direction::MemoryToMemory].
pub fn abcd(dst: u8, mode: Direction, src: u8) -> u16 {
    assert!(dst <= 7, "Invalid destination register number {}.", dst);
    assert!(mode == Direction::RegisterToRegister || mode == Direction::MemoryToMemory, "Invalid mode.");
    assert!(src <= 7, "Invalid source register number {}.", dst);
    register_size_mode_register(0b1100, dst, Size::Byte, 0, mode, src)
}

/// `dir` must be [Direction::DstReg] or [Direction::DstEa].
pub fn add(reg: u8, dir: Direction, size: Size, am: AddressingMode) -> Vec<u16> {
    assert!(reg <= 7, "Invalid register.");
    assert!(dir == Direction::DstEa || dir == Direction::DstReg, "Invalid direction.");
    if dir == Direction::DstEa {
        assert!(am.verify(&MODES_234567, &[0, 1]), "Invalid addressing mode.");
    } else {
        assert!(!(am.is_ard() && size.is_byte()), "Byte size cannot be used with Address Register Direct source operand.");
        assert!(am.verify(&MODES_01234567, &[0, 1, 2, 3, 4]), "Invalid addressing mode.");
    }
    register_direction_size_effective_address(0b1101, reg, dir, size, am)
}

pub fn adda(reg: u8, size: Size, am: AddressingMode) -> Vec<u16> {
    assert!(reg <= 7, "Invalid register.");
    assert!(!size.is_byte(), "ADDA cannot be byte sized.");
    register_size_effective_address(0b1101, reg, size, am)
}

pub fn addi(size: Size, am: AddressingMode, imm: u32) -> Vec<u16> {
    assert!(am.verify(&MODES_0234567, &[0, 1]), "Invalid addressing mode in ADDI assembler");
    size_effective_address_immediate(0b0000_0110, size, am, imm)
}

/// `data` must be 1 to 8.
pub fn addq(data: u8, size: Size, am: AddressingMode) -> Vec<u16> {
    assert!(am.verify(&MODES_01234567, &[0, 1]), "Invalid addressing mode.");
    assert!(!(am.is_ard() && size.is_byte()), "Byte size cannot be used with Address Register Direct destination operand.");
    assert!(data >= 1 && data <= 8, "Invalid data.");
    let data = if data == 8 { 0 } else { data };
    data_size_effective_address(data, 0, size, am)
}

/// `mode` must be [Direction::RegisterToRegister] or [Direction::MemoryToMemory].
pub fn addx(dst: u8, size: Size, mode: Direction, src: u8) -> u16 {
    assert!(dst <= 7, "Invalid destination register number {}.", dst);
    assert!(mode == Direction::RegisterToRegister || mode == Direction::MemoryToMemory, "Invalid mode.");
    assert!(src <= 7, "Invalid source register number {}.", dst);
    register_size_mode_register(0b1101, dst, size, 0, mode, src)
}

/// `dir` must be [Direction::DstReg] or [Direction::DstEa].
pub fn and(reg: u8, dir: Direction, size: Size, am: AddressingMode) -> Vec<u16> {
    assert!(reg <= 7, "Invalid register.");
    assert!(dir == Direction::DstEa || dir == Direction::DstReg, "Invalid direction.");
    if dir == Direction::DstEa {
        assert!(am.verify(&MODES_234567, &[0, 1]), "Invalid addressing mode.");
    } else {
        assert!(am.verify(&MODES_0234567, &[0, 1, 2, 3, 4]), "Invalid addressing mode.");
    }
    register_direction_size_effective_address(0b1100, reg, dir, size, am)
}

pub fn andi(size: Size, am: AddressingMode, imm: u32) -> Vec<u16> {
    assert!(am.verify(&MODES_0234567, &[0, 1]), "Invalid addressing mode in ANDI assembler");
    size_effective_address_immediate(0b0000_0010, size, am, imm)
}

pub fn andiccr(imm: u16) -> [u16; 2] {
    [0x023C, imm & 0x00FF]
}

pub fn andisr(imm: u16) -> [u16; 2] {
    [0x027C, imm]
}

/// Arithmetic Shift in memory (BYTE size only). `dir` must be [Direction::Left] or [Direction::Right].
pub fn asm(dir: Direction, am: AddressingMode) -> Vec<u16> {
    assert!(dir == Direction::Left || dir == Direction::Right, "Invalid direction field in ASm assembler: expected left or right, got {:?}", dir);
    assert!(am.verify(&MODES_234567, &[0, 1]), "Invalid addressing mode field in ASm assembler");
    direction_effective_address(0b1110_000, dir, 0b11, am)
}

/// Arithmetic Shift in register. `dir` must be [Direction::Left] or [Direction::Right].
pub fn asr(count_reg: u16, dir: Direction, size: Size, reg_shift: bool, reg: u16) -> u16 {
    assert!(count_reg <= 7, "Invalid count/register field in ASr assembler: expected 0 to 7, got {}", count_reg);
    assert!(dir == Direction::Left || dir == Direction::Right, "Invalid direction field in ASr assembler: expected left or right, got {:?}", dir);
    assert!(reg <= 7, "Invalid register field in ASr assembler: expected 0 to 7, got {}", reg);
    rotation_direction_size_mode_register(0b1110, count_reg, dir, size, reg_shift as u16, 0b00, reg)
}

/// If the displacement fits in an i8 and is not 0, 1 opcode is used, otherwise 2.
pub fn bcc(cond: Condition, disp: i16) -> Vec<u16> {
    assert!(cond != Condition::T && cond != Condition::F, "Invalid condition.");
    condition_displacement(cond, disp)
}

pub fn bchg_dynamic(reg: u8, am: AddressingMode) -> Vec<u16> {
    assert!(reg <= 7, "Invalid register field in BCHG dynamic assembler: expected 0 to 7, got {}", reg);
    assert!(am.verify(&MODES_0234567, &[0, 1]), "Invalid addressing mode in BCHG dynamic assembler");
    register_effective_address(0b0000, reg as u16, 0b101, am)
}

pub fn bchg_static(am: AddressingMode, count: u8) -> Vec<u16> {
    assert!(am.verify(&MODES_0234567, &[0, 1]), "Invalid addressing mode in BCHG static assembler");
    effective_address_count(0b0000_1000_01, am, count)
}

pub fn bclr_dynamic(reg: u8, am: AddressingMode) -> Vec<u16> {
    assert!(reg <= 7, "Invalid register field in BCLR dynamic assembler: expected 0 to 7, got {}", reg);
    assert!(am.verify(&MODES_0234567, &[0, 1]), "Invalid addressing mode in BCLR dynamic assembler");
    register_effective_address(0b0000, reg as u16, 0b110, am)
}

pub fn bclr_static(am: AddressingMode, count: u8) -> Vec<u16> {
    assert!(am.verify(&MODES_0234567, &[0, 1]), "Invalid addressing mode in BCLR static assembler");
    effective_address_count(0b0000_1000_10, am, count)
}

/// If the displacement fits in an i8 and is not 0, 1 opcode is used, otherwise 2.
pub fn bra(disp: i16) -> Vec<u16> {
    condition_displacement(Condition::T, disp)
}

pub fn bset_dynamic(reg: u8, am: AddressingMode) -> Vec<u16> {
    assert!(reg <= 7, "Invalid register field in BSET dynamic assembler: expected 0 to 7, got {}", reg);
    assert!(am.verify(&MODES_0234567, &[0, 1]), "Invalid addressing mode in BSET dynamic assembler");
    register_effective_address(0b0000, reg as u16, 0b111, am)
}

pub fn bset_static(am: AddressingMode, count: u8) -> Vec<u16> {
    assert!(am.verify(&MODES_0234567, &[0, 1]), "Invalid addressing mode in BSET static assembler");
    effective_address_count(0b0000_1000_11, am, count)
}

/// If the displacement fits in an i8 and is not 0, 1 opcode is used, otherwise 2.
pub fn bsr(disp: i16) -> Vec<u16> {
    condition_displacement(Condition::F, disp)
}

pub fn btst_dynamic(reg: u8, am: AddressingMode) -> Vec<u16> {
    assert!(reg <= 7, "Invalid register field in BTST dynamic assembler: expected 0 to 7, got {}", reg);
    assert!(am.verify(&MODES_0234567, &[0, 1, 2, 3, 4]), "Invalid addressing mode in BTST dynamic assembler");
    register_effective_address(0b0000, reg as u16, 0b100, am)
}

pub fn btst_static(am: AddressingMode, count: u8) -> Vec<u16> {
    assert!(am.verify(&MODES_0234567, &[0, 1, 2, 3, 4]), "Invalid addressing mode in BTST static assembler");
    effective_address_count(0b0000_1000_00, am, count)
}

pub fn chk(reg: u8, am: AddressingMode) -> Vec<u16> {
    assert!(reg <= 7, "Invalid register field in CHK assembler: expected 0 to 7, got {}", reg);
    assert!(am.verify(&MODES_0234567, &[0, 1, 2, 3, 4]), "Invalid addressing mode in CHK assembler");
    register_effective_address(0b0100, reg as u16, 0b110, am)
}

pub fn clr(size: Size, am: AddressingMode) -> Vec<u16> {
    assert!(am.verify(&MODES_0234567, &[0, 1]), "Invalid addressing mode in CLR assembler");
    size_effective_address(0b0100_0010, size, am)
}

pub fn cmp(reg: u8, size: Size, am: AddressingMode) -> Vec<u16> {
    assert!(reg <= 7, "Invalid register.");
    assert!(!(am.is_ard() && size.is_byte()), "Byte size cannot be used with Address Register Direct source operand.");
    assert!(am.verify(&MODES_01234567, &[0, 1, 2, 3, 4]), "Invalid addressing mode.");
    register_direction_size_effective_address(0b1011, reg, Direction::DstReg, size, am)
}

pub fn cmpa(reg: u8, size: Size, am: AddressingMode) -> Vec<u16> {
    assert!(reg <= 7, "Invalid register.");
    assert!(!size.is_byte(), "CMPA cannot be byte sized.");
    register_size_effective_address(0b1011, reg, size, am)
}

pub fn cmpi(size: Size, am: AddressingMode, imm: u32) -> Vec<u16> {
    assert!(am.verify(&MODES_0234567, &[0, 1]), "Invalid addressing mode in CMPI assembler");
    size_effective_address_immediate(0b0000_1100, size, am, imm)
}

pub fn cmpm(ax: u8, size: Size, ay: u8) -> u16 {
    assert!(ax <= 7, "Invalid destination register.");
    assert!(ay <= 7, "Invalid source register.");
    0b1011_0001 << 8 | (ax as u16 & 7) << 9 | Into::<u16>::into(size) << 6 | 0b001 << 3 | ay as u16 & 7
}

pub fn dbcc(cond: Condition, reg: u8, disp: i16) -> [u16; 2] {
    assert!(reg <= 7, "Invalid register.");
    [0b0101 << 12 | (cond as u16) << 8 | 0b1100_1 << 3 | reg as u16 & 7, disp as u16]
}

pub fn divs(reg: u8, am: AddressingMode) -> Vec<u16> {
    assert!(reg <= 7, "Invalid register field in DIVS assembler: expected 0 to 7, got {}", reg);
    assert!(am.verify(&MODES_0234567, &[0, 1, 2, 3, 4]), "Invalid addressing mode in DIVS assembler");
    register_effective_address(0b1000, reg as u16, 0b111, am)
}

pub fn divu(reg: u8, am: AddressingMode) -> Vec<u16> {
    assert!(reg <= 7, "Invalid register field in DIVU assembler: expected 0 to 7, got {}", reg);
    assert!(am.verify(&MODES_0234567, &[0, 1, 2, 3, 4]), "Invalid addressing mode in DIVU assembler");
    register_effective_address(0b1000, reg as u16, 0b011, am)
}

pub fn eor(reg: u8, size: Size, am: AddressingMode) -> Vec<u16> {
    assert!(reg <= 7, "Invalid register.");
    assert!(am.verify(&MODES_0234567, &[0, 1]), "Invalid addressing mode.");
    register_direction_size_effective_address(0b1011, reg, Direction::DstEa, size, am)
}

pub fn eori(size: Size, am: AddressingMode, imm: u32) -> Vec<u16> {
    assert!(am.verify(&MODES_0234567, &[0, 1]), "Invalid addressing mode in EORI assembler");
    size_effective_address_immediate(0b0000_1010, size, am, imm)
}

pub fn eoriccr(imm: u16) -> [u16; 2] {
    [0x0A3C, imm & 0x00FF]
}

pub fn eorisr(imm: u16) -> [u16; 2] {
    [0x0A7C, imm]
}

/// `dir` must be [Direction::ExchangeData], [Direction::ExchangeAddress] or [Direction::ExchangeDataAddress].
pub fn exg(rx: u8, dir: Direction, ry: u8) -> u16 {
    assert!(dir == Direction::ExchangeData || dir == Direction::ExchangeAddress || dir == Direction::ExchangeDataAddress, "Invalid operation");
    assert!(rx <= 7, "Invalid Rx register.");
    assert!(ry <= 7, "Invalid Ry register.");
    let opmode = if dir == Direction::ExchangeData {
        0b01000
    } else if dir == Direction::ExchangeAddress {
        0b01001
    } else {
        0b10001
    };
    0b1100_0001 << 8 | (rx as u16) << 9 | opmode << 3 | ry as u16 & 7
}

/// `word_to_long` is true for word to long sign extension, false for byte to word sign extension.
pub fn ext(word_to_long: bool, reg: u8) -> u16 {
    assert!(reg <= 7, "Invalid register.");
    0b0100_1000_1 << 7 | (word_to_long as u16) << 6 | reg as u16 & 7
}

pub fn illegal() -> u16 {
    0x4AFC
}

pub fn jmp(am: AddressingMode) -> Vec<u16> {
    assert!(am.verify(&MODES_2567, &[0, 1, 2, 3]), "Invalid addressing mode in JMP assembler");
    effective_address(0b0100_1110_11, am)
}

pub fn jsr(am: AddressingMode) -> Vec<u16> {
    assert!(am.verify(&MODES_2567, &[0, 1, 2, 3]), "Invalid addressing mode in JSR assembler");
    effective_address(0b0100_1110_10, am)
}

pub fn lea(reg: u8, am: AddressingMode) -> Vec<u16> {
    assert!(reg <= 7, "Invalid register field in LEA assembler: expected 0 to 7, got {}", reg);
    assert!(am.verify(&MODES_2567, &[0, 1, 2, 3]), "Invalid addressing mode in LEA assembler");
    register_effective_address(0b0100, reg as u16, 0b111, am)
}

pub fn link(reg: u8, disp: i16) -> [u16; 2] {
    assert!(reg <= 7, "Invalid register.");
    [0b0100_1110_0101_0 << 3 | reg as u16 & 7, disp as u16]
}

/// Logical Shift in memory (BYTE size only). `dir` must be [Direction::Left] or [Direction::Right].
pub fn lsm(dir: Direction, am: AddressingMode) -> Vec<u16> {
    assert!(dir == Direction::Left || dir == Direction::Right, "Invalid direction field in LSm assembler: expected left or right, got {:?}", dir);
    assert!(am.verify(&MODES_234567, &[0, 1]), "Invalid addressing mode field in LSm assembler");
    direction_effective_address(0b1110_001, dir, 0b11, am)
}

/// Logical Shift in register. `dir` must be [Direction::Left] or [Direction::Right].
pub fn lsr(count_reg: u16, dir: Direction, size: Size, reg_shift: bool, reg: u16) -> u16 {
    assert!(count_reg <= 7, "Invalid count/register field in LSr assembler: expected 0 to 7, got {}", count_reg);
    assert!(dir == Direction::Left || dir == Direction::Right, "Invalid direction field in LSr assembler: expected left or right, got {:?}", dir);
    assert!(reg <= 7, "Invalid register field in LSr assembler: expected 0 to 7, got {}", reg);
    rotation_direction_size_mode_register(0b1110, count_reg, dir, size, reg_shift as u16, 0b01, reg)
}

pub fn r#move(size: Size, dst: AddressingMode, src: AddressingMode) -> Vec<u16> {
    assert!(dst.verify(&MODES_0234567, &[0, 1]), "Invalid destination addressing mode.");
    assert!(!(src.is_ard() && size.is_byte()), "Byte size cannot be used with Address Register Direct source operand.");
    size_effective_address_effective_address(size, dst, src)
}

pub fn movea(size: Size, dst_reg: u8, src: AddressingMode) -> Vec<u16> {
    assert!(dst_reg <= 7, "Invalid address register.");
    assert!(size != Size::Byte, "MOVEA cannot be byte sized.");
    size_effective_address_effective_address(size, AddressingMode::Ard(dst_reg), src)
}

pub fn moveccr(am: AddressingMode) -> Vec<u16> {
    assert!(am.verify(&MODES_0234567, &[0, 1, 2, 3, 4]), "Invalid addressing mode in MOVE to CCR assembler");
    effective_address(0b0100_0100_11, am)
}

pub fn movefsr(am: AddressingMode) -> Vec<u16> {
    assert!(am.verify(&MODES_0234567, &[0, 1]), "Invalid addressing mode in MOVE from SR assembler");
    effective_address(0b0100_0000_11, am)
}

pub fn movesr(am: AddressingMode) -> Vec<u16> {
    assert!(am.verify(&MODES_0234567, &[0, 1, 2, 3, 4]), "Invalid addressing mode in MOVE to SR assembler");
    effective_address(0b0100_0110_11, am)
}

/// `dir` must be [Direction::UspToRegister] or [Direction::RegisterToUsp].
pub fn moveusp(dir: Direction, reg: u8) -> u16 {
    assert!(dir == Direction::UspToRegister || dir == Direction::RegisterToUsp, "Invalid direction.");
    assert!(reg <= 7, "Invalid register");
    let d = if dir == Direction::UspToRegister { 1 } else { 0 };
    0b0100_1110_0110 << 4 | d << 3 | reg as u16 & 7
}

/// `dir` must be [Direction::RegisterToMemory] or [Direction::MemoryToRegister]. `mask` is the raw mask list.
pub fn movem(dir: Direction, size: Size, am: AddressingMode, mask: u16) -> Vec<u16> {
    assert!(dir == Direction::RegisterToMemory || dir == Direction::MemoryToRegister, "Invalid direction.");
    assert!(!size.is_byte(), "Invalid byte size for MOVEM.");
    let d = if dir == Direction::MemoryToRegister {
        assert!(am.verify(&[2, 3, 5, 6, 7], &[0, 1, 2, 3]), "Invalid addressing mode.");
        1
    } else {
        assert!(am.verify(&[2, 4, 5, 6, 7], &[0, 1]), "Invalid addressing mode.");
        0
    };

    let mut vec = Vec::new();

    let (eafield, eaext) = am.assemble(size.is_long());
    let opcode = 0b0100_1 << 11
               | d << 10
               | 0b001 << 7
               | size.into_bit() << 6
               | eafield;

    vec.push(opcode);
    vec.push(mask);
    vec.extend(eaext.iter());

    vec
}

/// `dir` must be [Direction::MemoryToRegister] or [Direction::RegisterToMemory].
pub fn movep(data_reg: u8, dir: Direction, size: Size, addr_reg: u8, disp: i16) -> [u16; 2] {
    assert!(data_reg <= 7, "Invalid data register.");
    assert!(dir == Direction::RegisterToMemory || dir == Direction::MemoryToRegister, "Invalid direction.");
    assert!(!size.is_byte(), "Invalid byte size for MOVEP.");
    assert!(addr_reg <= 7, "Invalid address register.");

    let mut opcode = (data_reg as u16 & 7) << 9
                   | 0b1_0000_1 << 3
                   | addr_reg as u16 & 7;
    if dir == Direction::RegisterToMemory {
        opcode |= 0x0080;
    }
    if size.is_long() {
        opcode |= 0x0040;
    }
    [opcode, disp as u16]
}

pub fn moveq(reg: u8, data: i8) -> u16 {
    assert!(reg <= 7, "Invalid register.");
    0b0111 << 12 | (reg as u16) << 9 | data as u8 as u16
}

pub fn muls(reg: u8, am: AddressingMode) -> Vec<u16> {
    assert!(reg <= 7, "Invalid register field in MULS assembler: expected 0 to 7, got {}", reg);
    assert!(am.verify(&MODES_0234567, &[0, 1, 2, 3, 4]), "Invalid addressing mode in MULS assembler");
    register_effective_address(0b1100, reg as u16, 0b111, am)
}

pub fn mulu(reg: u8, am: AddressingMode) -> Vec<u16> {
    assert!(reg <= 7, "Invalid register field in MULU assembler: expected 0 to 7, got {}", reg);
    assert!(am.verify(&MODES_0234567, &[0, 1, 2, 3, 4]), "Invalid addressing mode in MULU assembler");
    register_effective_address(0b1100, reg as u16, 0b011, am)
}

pub fn nbcd(am: AddressingMode) -> Vec<u16> {
    assert!(am.verify(&MODES_0234567, &[0, 1]), "Invalid addressing mode in NBCD assembler");
    effective_address(0b0100_1000_00, am)
}

pub fn neg(size: Size, am: AddressingMode) -> Vec<u16> {
    assert!(am.verify(&MODES_0234567, &[0, 1]), "Invalid addressing mode in NEG assembler");
    size_effective_address(0b0100_0100, size, am)
}

pub fn negx(size: Size, am: AddressingMode) -> Vec<u16> {
    assert!(am.verify(&MODES_0234567, &[0, 1]), "Invalid addressing mode in NEGX assembler");
    size_effective_address(0b0100_0000, size, am)
}

pub fn nop() -> u16 {
    0x4E71
}

pub fn not(size: Size, am: AddressingMode) -> Vec<u16> {
    assert!(am.verify(&MODES_0234567, &[0, 1]), "Invalid addressing mode in NOT assembler");
    size_effective_address(0b0100_0110, size, am)
}

/// `dir` must be [Direction::DstReg] or [Direction::DstEa].
pub fn or(reg: u8, dir: Direction, size: Size, am: AddressingMode) -> Vec<u16> {
    assert!(reg <= 7, "Invalid register.");
    assert!(dir == Direction::DstEa || dir == Direction::DstReg, "Invalid direction.");
    if dir == Direction::DstEa {
        assert!(am.verify(&MODES_234567, &[0, 1]), "Invalid addressing mode.");
    } else {
        assert!(am.verify(&MODES_0234567, &[0, 1, 2, 3, 4]), "Invalid addressing mode.");
    }
    register_direction_size_effective_address(0b1000, reg, dir, size, am)
}

pub fn ori(size: Size, am: AddressingMode, imm: u32) -> Vec<u16> {
    assert!(am.verify(&MODES_0234567, &[0, 1]), "Invalid addressing mode in ORI assembler");
    size_effective_address_immediate(0b0000_0000, size, am, imm)
}

pub fn oriccr(imm: u16) -> [u16; 2] {
    [0x003C, imm & 0x00FF]
}

pub fn orisr(imm: u16) -> [u16; 2] {
    [0x007C, imm]
}

pub fn pea(am: AddressingMode) -> Vec<u16> {
    assert!(am.verify(&MODES_2567, &[0, 1, 2, 3]), "Invalid addressing mode in PEA assembler");
    effective_address(0b0100_1000_01, am)
}

pub fn reset() -> u16 {
    0x4E70
}

/// Rotate in memory (BYTE size only). `dir` must be [Direction::Left] or [Direction::Right].
pub fn rom(dir: Direction, am: AddressingMode) -> Vec<u16> {
    assert!(dir == Direction::Left || dir == Direction::Right, "Invalid direction field in ROm assembler: expected left or right, got {:?}", dir);
    assert!(am.verify(&MODES_234567, &[0, 1]), "Invalid addressing mode field in ROm assembler");
    direction_effective_address(0b1110_011, dir, 0b11, am)
}

/// Rotate in register. `dir` must be [Direction::Left] or [Direction::Right].
pub fn ror(count_reg: u16, dir: Direction, size: Size, reg_shift: bool, reg: u16) -> u16 {
    assert!(count_reg <= 7, "Invalid count/register field in ROr assembler: expected 0 to 7, got {}", count_reg);
    assert!(dir == Direction::Left || dir == Direction::Right, "Invalid direction field in ROr assembler: expected left or right, got {:?}", dir);
    assert!(reg <= 7, "Invalid register field in ROr assembler: expected 0 to 7, got {}", reg);
    rotation_direction_size_mode_register(0b1110, count_reg, dir, size, reg_shift as u16, 0b11, reg)
}

/// Rotate with Extend in memory (BYTE size only). `dir` must be [Direction::Left] or [Direction::Right].
pub fn roxm(dir: Direction, am: AddressingMode) -> Vec<u16> {
    assert!(dir == Direction::Left || dir == Direction::Right, "Invalid direction field in ROXm assembler: expected left or right, got {:?}", dir);
    assert!(am.verify(&MODES_234567, &[0, 1]), "Invalid addressing mode field in ROXm assembler");
    direction_effective_address(0b1110_010, dir, 0b11, am)
}

/// Rotate with Extend in register. `dir` must be [Direction::Left] or [Direction::Right].
pub fn roxr(count_reg: u16, dir: Direction, size: Size, reg_shift: bool, reg: u16) -> u16 {
    assert!(count_reg <= 7, "Invalid count/register field in ROXr assembler: expected 0 to 7, got {}", count_reg);
    assert!(dir == Direction::Left || dir == Direction::Right, "Invalid direction field in ROXr assembler: expected left or right, got {:?}", dir);
    assert!(reg <= 7, "Invalid register field in ROXr assembler: expected 0 to 7, got {}", reg);
    rotation_direction_size_mode_register(0b1110, count_reg, dir, size, reg_shift as u16, 0b10, reg)
}

pub fn rte() -> u16 {
    0x4E73
}

pub fn rtr() -> u16 {
    0x4E77
}

pub fn rts() -> u16 {
    0x4E75
}

/// `mode` must be [Direction::RegisterToRegister] or [Direction::MemoryToMemory].
pub fn sbcd(dst: u8, mode: Direction, src: u8) -> u16 {
    assert!(dst <= 7, "Invalid destination register number {}.", dst);
    assert!(mode == Direction::RegisterToRegister || mode == Direction::MemoryToMemory, "Invalid mode.");
    assert!(src <= 7, "Invalid source register number {}.", dst);
    register_size_mode_register(0b1000, dst, Size::Byte, 0, mode, src)
}

pub fn scc(cond: Condition, am: AddressingMode) -> Vec<u16> {
    assert!(am.verify(&MODES_0234567, &[0, 1]), "Invalid addressing mode");
    let mut vec = Vec::new();

    let (eafield, eaext) = am.assemble(false);
    let opcode = 0b0101 << 12
               | (cond as u16) << 8
               | 0b11 << 6
               | eafield;
    vec.push(opcode);
    vec.extend(eaext.iter());

    vec
}

pub fn stop(sr: u16) -> [u16; 2] {
    [0x4E72, sr]
}

/// `dir` must be [Direction::DstReg] or [Direction::DstEa].
pub fn sub(reg: u8, dir: Direction, size: Size, am: AddressingMode) -> Vec<u16> {
    assert!(reg <= 7, "Invalid register.");
    assert!(dir == Direction::DstEa || dir == Direction::DstReg, "Invalid direction.");
    if dir == Direction::DstEa {
        assert!(am.verify(&MODES_234567, &[0, 1]), "Invalid addressing mode.");
    } else {
        assert!(!(am.is_ard() && size.is_byte()), "Byte size cannot be used with Address Register Direct source operand.");
        assert!(am.verify(&MODES_01234567, &[0, 1, 2, 3, 4]), "Invalid addressing mode.");
    }
    register_direction_size_effective_address(0b1001, reg, dir, size, am)
}

pub fn suba(reg: u8, size: Size, am: AddressingMode) -> Vec<u16> {
    assert!(reg <= 7, "Invalid register.");
    assert!(!size.is_byte(), "SUBA cannot be byte sized.");
    register_size_effective_address(0b1001, reg, size, am)
}

pub fn subi(size: Size, am: AddressingMode, imm: u32) -> Vec<u16> {
    assert!(am.verify(&MODES_0234567, &[0, 1]), "Invalid addressing mode in SUBI assembler");
    size_effective_address_immediate(0b0000_0100, size, am, imm)
}

/// `data` must be 1 to 8.
pub fn subq(data: u8, size: Size, am: AddressingMode) -> Vec<u16> {
    assert!(am.verify(&MODES_01234567, &[0, 1]), "Invalid addressing mode.");
    assert!(!(am.is_ard() && size.is_byte()), "Byte size cannot be used with Address Register Direct destination operand.");
    assert!(data >= 1 && data <= 8, "Invalid data.");
    let data = if data == 8 { 0 } else { data };
    data_size_effective_address(data, 1, size, am)
}

/// `mode` must be [Direction::RegisterToRegister] or [Direction::MemoryToMemory].
pub fn subx(dst: u8, size: Size, mode: Direction, src: u8) -> u16 {
    assert!(dst <= 7, "Invalid destination register number {}.", dst);
    assert!(mode == Direction::RegisterToRegister || mode == Direction::MemoryToMemory, "Invalid mode.");
    assert!(src <= 7, "Invalid source register number {}.", dst);
    register_size_mode_register(0b1001, dst, size, 0, mode, src)
}

pub fn swap(reg: u8) -> u16 {
    assert!(reg <= 7, "Invalid register.");
    register(0b0100_1000_0100_0, reg)
}

pub fn tas(am: AddressingMode) -> Vec<u16> {
    assert!(am.verify(&MODES_0234567, &[0, 1]), "Invalid addressing mode in TAS assembler");
    effective_address(0b0100_1010_11, am)
}

pub fn trap(vector: u8) -> u16 {
    assert!(vector <= 15, "Invalid TRAP vector.");
    0b0100_1110_0100 << 4 | vector as u16
}

pub fn trapv() -> u16 {
    0x4E76
}

pub fn tst(size: Size, am: AddressingMode) -> Vec<u16> {
    assert!(am.verify(&MODES_0234567, &[0, 1]), "Invalid addressing mode in TST assembler");
    size_effective_address(0b0100_1010, size, am)
}

pub fn unlk(reg: u8) -> u16 {
    assert!(reg <= 7, "Invalid register.");
    register(0b0100_1110_0101_1, reg)
}
