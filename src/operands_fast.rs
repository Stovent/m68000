//! Instruction-related structs, enums and functions.
//!
//! The functions returns the operands and the size in bytes of the extension words.
//! They take as parameters the opcode of the instruction and an iterator over the extension words.

use crate::M68000;
use crate::addressing_modes::AddressingMode;
use crate::decoder::DECODER;
use crate::instruction::{Direction, Size};
use crate::isa::Isa;
use crate::memory_access::MemoryAccess;
use crate::utils::bits;

impl M68000 {
    /// ANDI/EORI/ORI CCR/SR, STOP
    pub(super) fn immediate(&mut self, memory: &mut impl MemoryAccess) -> u16 {
        self.get_next_word(memory).expect("Access error occured when fetching immediate operand.")
    }

    /// ADDI, ANDI, CMPI, EORI, ORI, SUBI
    pub(super) fn size_effective_address_immediate(&mut self, memory: &mut impl MemoryAccess) -> (Size, AddressingMode, u32) {
        let size = Size::from(bits(self.current_opcode, 6, 7));

        let imm = if size.is_long() {
            self.get_next_long(memory).expect("Access error occured when fetching immediate operand.")
        } else {
            self.get_next_word(memory).expect("Access error occured when fetching immediate operand.") as u32
        };

        let eareg = bits(self.current_opcode, 0, 2) as u8;
        let eamode = bits(self.current_opcode, 3, 5);
        let am = AddressingMode::new_fast(eamode, eareg, Some(size), self, memory);

        (size, am, imm)
    }

    /// BCHG, BCLR, BSET, BTST
    pub(super) fn effective_address_count(&mut self, memory: &mut impl MemoryAccess) -> (AddressingMode, u8) {
        let count = if bits(self.current_opcode, 8, 8) != 0 { // dynamic bit number
            bits(self.current_opcode, 9, 11) as u8
        } else { // Static bit number
            self.get_next_word(memory).expect("Access error occured when fetching count operand.") as u8
        };

        let eareg = bits(self.current_opcode, 0, 2) as u8;
        let eamode = bits(self.current_opcode, 3, 5);
        let size = if eamode == 0 { Some(Size::Long) } else { Some(Size::Byte) };
        let am = AddressingMode::new_fast(eamode, eareg, size, self, memory);

        (am, count)
    }

    /// JMP, JSR, MOVE (f) SR CCR, NBCD, PEA, TAS
    pub(super) fn effective_address(&mut self, memory: &mut impl MemoryAccess) -> AddressingMode {
        let isa = DECODER[self.current_opcode as usize];

        let size = if isa == Isa::Nbcd || isa == Isa::Tas {
            Some(Size::Byte)
        } else if isa == Isa::Moveccr || isa == Isa::Movefsr || isa == Isa::Movesr {
            Some(Size::Word)
        } else if isa == Isa::Pea {
            Some(Size::Long)
        } else {
            None
        };

        let eareg = bits(self.current_opcode, 0, 2) as u8;
        let eamode = bits(self.current_opcode, 3, 5);
        let am = AddressingMode::new_fast(eamode, eareg, size, self, memory);

        am
    }

    /// CLR, NEG, NEGX, NOT, TST
    pub(super) fn size_effective_address(&mut self, memory: &mut impl MemoryAccess) -> (Size, AddressingMode) {
        let eareg = bits(self.current_opcode, 0, 2) as u8;
        let eamode = bits(self.current_opcode, 3, 5);
        let size = Size::from(bits(self.current_opcode, 6, 7));
        let am = AddressingMode::new_fast(eamode, eareg, Some(size), self, memory);

        (size, am)
    }

    /// CHK, DIVS, DIVU, LEA, MULS, MULU
    pub(super) fn register_effective_address(&mut self, memory: &mut impl MemoryAccess) -> (u8, AddressingMode) {
        let isa = DECODER[self.current_opcode as usize];

        let reg = bits(self.current_opcode, 9, 11) as u8;
        let size = if isa == Isa::Lea {
            Some(Size::Long)
        } else {
            Some(Size::Word)
        };

        let eareg = bits(self.current_opcode, 0, 2) as u8;
        let eamode = bits(self.current_opcode, 3, 5);
        let am = AddressingMode::new_fast(eamode, eareg, size, self, memory);

        (reg, am)
    }

    /// MOVEP
    pub(super) fn register_direction_size_register_displacement(&mut self, memory: &mut impl MemoryAccess) -> (u8, Direction, Size, u8, i16) {
        let dreg = bits(self.current_opcode, 9, 11) as u8;
        let dir = if bits(self.current_opcode, 7, 7) != 0 { Direction::RegisterToMemory } else { Direction::MemoryToRegister };
        let size = if bits(self.current_opcode, 6, 6) != 0 { Size::Long } else { Size::Word };
        let areg = bits(self.current_opcode, 0, 2) as u8;
        let disp = self.get_next_word(memory).expect("Access error occured when fetching displacement operand.") as i16;

        (dreg, dir, size, areg, disp)
    }

    /// MOVEA
    pub(super) fn size_register_effective_address(&mut self, memory: &mut impl MemoryAccess) -> (Size, u8, AddressingMode) {
        let eareg = bits(self.current_opcode, 0, 2) as u8;
        let eamode = bits(self.current_opcode, 3, 5);
        let areg = bits(self.current_opcode, 9, 11) as u8;
        let size = Size::from_move(bits(self.current_opcode, 12, 13));
        let am = AddressingMode::new_fast(eamode, eareg, Some(size), self, memory);

        (size, areg, am)
    }

    /// MOVE
    pub(super) fn size_effective_address_effective_address(&mut self, memory: &mut impl MemoryAccess) -> (Size, AddressingMode, AddressingMode) {
        let size = Size::from_move(bits(self.current_opcode, 12, 13));

        // First read the source operand then the destination.
        let eareg = bits(self.current_opcode, 0, 2) as u8;
        let eamode = bits(self.current_opcode, 3, 5);
        let src = AddressingMode::new_fast(eamode, eareg, Some(size), self, memory);

        let eamode = bits(self.current_opcode, 6, 8);
        let eareg = bits(self.current_opcode, 9, 11) as u8;
        let dst = AddressingMode::new_fast(eamode, eareg, Some(size), self, memory);

        (size, dst, src)
    }

    /// EXG
    pub(super) const fn register_opmode_register(&self) -> (u8, Direction, u8) {
        let regl = bits(self.current_opcode, 9, 11) as u8;
        let opmode = bits(self.current_opcode, 3, 7) as u8;
        let regr = bits(self.current_opcode, 0, 2) as u8;
        let dir = if opmode == 0b01000 {
            Direction::ExchangeData
        } else if opmode == 0b01001 {
            Direction::ExchangeAddress
        } else {
            Direction::ExchangeDataAddress
        };

        (regl, dir, regr)
    }

    /// EXT
    pub(super) const fn opmode_register(&self) -> (u8, u8) {
        let opmode = bits(self.current_opcode, 6, 8) as u8;
        let reg = bits(self.current_opcode, 0, 2) as u8;
        (opmode, reg)
    }

    /// TRAP
    pub(super) const fn vector(&self) -> u8 {
        bits(self.current_opcode, 0, 3) as u8
    }

    /// LINK
    pub(super) fn register_displacement(&mut self, memory: &mut impl MemoryAccess) -> (u8, i16) {
        let reg = bits(self.current_opcode, 0, 2) as u8;
        let disp = self.get_next_word(memory).expect("Access error occured when fetching displacement operand.") as i16;

        (reg, disp)
    }

    /// SWAP, UNLK
    pub(super) const fn register(&self) -> u8 {
        bits(self.current_opcode, 0, 2) as u8
    }

    /// MOVE USP
    pub(super) const fn direction_register(&self) -> (Direction, u8) {
        let dir = if bits(self.current_opcode, 3, 3) != 0 { Direction::UspToRegister } else { Direction::RegisterToUsp };
        let reg = bits(self.current_opcode, 0, 2) as u8;
        (dir, reg)
    }

    /// MOVEM
    pub(super) fn direction_size_effective_address_list(&mut self, memory: &mut impl MemoryAccess) -> (Direction, Size, AddressingMode, u16) {
        let list = self.get_next_word(memory).expect("Access error occured when fetching list operand.");
        let dir = if bits(self.current_opcode, 10, 10) != 0 { Direction::MemoryToRegister } else { Direction::RegisterToMemory };
        let size = Size::from_bit(bits(self.current_opcode, 6, 6));

        let eareg = bits(self.current_opcode, 0, 2) as u8;
        let eamode = bits(self.current_opcode, 3, 5);
        let am = AddressingMode::new_fast(eamode, eareg, Some(size), self, memory);

        (dir, size, am, list)
    }

    /// ADDQ, SUBQ
    pub(super) fn data_size_effective_address(&mut self, memory: &mut impl MemoryAccess) -> (u8, Size, AddressingMode) {
        let data = bits(self.current_opcode, 9, 11) as u8;
        let size = Size::from(bits(self.current_opcode, 6, 7));

        let eareg = bits(self.current_opcode, 0, 2) as u8;
        let eamode = bits(self.current_opcode, 3, 5);
        let am = AddressingMode::new_fast(eamode, eareg, Some(size), self, memory);

        (data, size, am)
    }

    /// Scc
    pub(super) fn condition_effective_address(&mut self, memory: &mut impl MemoryAccess) -> (u8, AddressingMode) {
        let condition = bits(self.current_opcode, 8, 11) as u8;

        let eareg = bits(self.current_opcode, 0, 2) as u8;
        let eamode = bits(self.current_opcode, 3, 5);
        let am = AddressingMode::new_fast(eamode, eareg, Some(Size::Byte), self, memory);

        (condition, am)
    }

    /// DBcc
    pub(super) fn condition_register_displacement(&mut self, memory: &mut impl MemoryAccess) -> (u8, u8, i16) {
        let disp = self.get_next_word(memory).expect("Access error occured when fetching displacement operand.") as i16;
        let condition = bits(self.current_opcode, 8, 11) as u8;
        let reg = bits(self.current_opcode, 0, 2) as u8;

        (condition, reg, disp)
    }

    /// BRA, BSR
    pub(super) fn displacement(&mut self, memory: &mut impl MemoryAccess) -> i16 {
        let disp = self.current_opcode as i8 as i16;
        if disp == 0 {
            self.get_next_word(memory).expect("Access error occured when fetching displacement operand.") as i16
        } else {
            disp
        }
    }

    /// Bcc
    pub(super) fn condition_displacement(&mut self, memory: &mut impl MemoryAccess) -> (u8, i16) {
        let mut disp = self.current_opcode as i8 as i16;
        if disp == 0 {
            disp = self.get_next_word(memory).expect("Access error occured when fetching displacement operand.") as i16;
        }
        let condition = bits(self.current_opcode, 8, 11) as u8;

        (condition, disp)
    }

    /// MOVEQ
    pub(super) const fn register_data(&self) -> (u8, i8) {
        let reg = bits(self.current_opcode, 9, 11) as u8;
        let data = self.current_opcode as i8;
        (reg, data)
    }

    /// ADD, AND, CMP, EOR, OR, SUB
    pub(super) fn register_direction_size_effective_address(&mut self, memory: &mut impl MemoryAccess) -> (u8, Direction, Size, AddressingMode) {
        let reg = bits(self.current_opcode, 9, 11) as u8;
        let dir = if bits(self.current_opcode, 8, 8) != 0 { Direction::DstEa } else { Direction::DstReg }; // CMP and EOR ignores it
        let size = Size::from(bits(self.current_opcode, 6, 7));

        let eareg = bits(self.current_opcode, 0, 2) as u8;
        let eamode = bits(self.current_opcode, 3, 5);
        let am = AddressingMode::new_fast(eamode, eareg, Some(size), self, memory);

        (reg, dir, size, am)
    }

    /// ADDA, CMPA, SUBA
    pub(super) fn register_size_effective_address(&mut self, memory: &mut impl MemoryAccess) -> (u8, Size, AddressingMode) {
        let reg = bits(self.current_opcode, 9, 11) as u8;
        let size = Size::from_bit(bits(self.current_opcode, 8, 8));

        let eareg = bits(self.current_opcode, 0, 2) as u8;
        let eamode = bits(self.current_opcode, 3, 5);
        let am = AddressingMode::new_fast(eamode, eareg, Some(size), self, memory);

        (reg, size, am)
    }

    /// ABCD, ADDX, SBCD, SUBX
    pub(super) fn register_size_mode_register(&self) -> (u8, Size, Direction, u8) {
        let regl = bits(self.current_opcode, 9, 11) as u8;
        let size = Size::from(bits(self.current_opcode, 6, 7));
        let mode = if bits(self.current_opcode, 3, 3) != 0 { Direction::MemoryToMemory } else { Direction::RegisterToRegister };
        let regr = bits(self.current_opcode, 0, 2) as u8;
        (regl, size, mode, regr)
    }

    /// CMPM
    pub(super) fn register_size_register(&self) -> (u8, Size, u8) {
        let regl = bits(self.current_opcode, 9, 11) as u8;
        let size = Size::from(bits(self.current_opcode, 6, 7));
        let regr = bits(self.current_opcode, 0, 2) as u8;
        (regl, size, regr)
    }

    /// ASm, LSm, ROm, ROXm
    pub(super) fn direction_effective_address(&mut self, memory: &mut impl MemoryAccess) -> (Direction, AddressingMode) {
        let eareg = bits(self.current_opcode, 0, 2) as u8;
        let eamode = bits(self.current_opcode, 3, 5);
        let dir = if bits(self.current_opcode, 8, 8) != 0 { Direction::Left } else { Direction::Right };
        let am = AddressingMode::new_fast(eamode, eareg, Some(Size::Byte), self, memory);
        (dir, am)
    }

    /// ASr, LSr, ROr, ROXr
    pub(super) fn rotation_direction_size_mode_register(&self) -> (u8, Direction, Size, u8, u8) {
        let count = bits(self.current_opcode, 9, 11) as u8;
        let dir = if bits(self.current_opcode, 8, 8) != 0 { Direction::Left } else { Direction::Right };
        let size = Size::from(bits(self.current_opcode, 6, 7));
        let mode = bits(self.current_opcode, 5, 5) as u8;
        let reg = bits(self.current_opcode, 0, 2) as u8;
        (count, dir, size, mode, reg)
    }
}
