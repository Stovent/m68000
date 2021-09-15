//! This module defines the Instruction structure and the different operands that can be found inside an opcode.
//! Its responsibility is only to retrive the operands and format them approprately.
//! It is the interpreter's role to interpret the operand.

use super::isa::ISA;

pub(super) struct Instruction {
    pub isa: ISA,
    /// The opcode itself.
    pub opcode: u16,
    /// The address of the instruction.
    pub pc: u32,
    /// The operands.
    pub operands: Operands,
}

use super::addressing_modes::EffectiveAddress;

/// Specify the direction of the operation.
///
/// `RegisterToMemory` and `MemoryToRegister` are used by MOVEM, MOVEP and MOVE USP.
///
/// `DstReg` and `DstEa` are used by ADD, AND, OR and SUB.
///
/// `Left` and `Right` are used by the Shift and Rotate instructions.
pub(super) enum Direction {
    /// Specify a transfert from a register to memory.
    RegisterToMemory,
    /// Specify a transfert from memory to a register.
    MemoryToRegister,
    /// Specify that the destination is a register.
    DstReg,
    /// Specify that the destination is in memory.
    DstEa,
    /// Specify a left shift of rotation.
    Left,
    /// Specify a right shift or rotation.
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Size {
    Byte = 1,
    Word = 2,
    Long = 4,
}

impl Size {
    /// returns Word when self is Byte, self otherwise.
    ///
    /// This is used in addressing modes, where byte post/pre increment
    /// increments the register by 2 instead of 1.
    pub(super) fn as_word_long(self) -> Self {
        if self == Self::Byte {
            Self::Word
        } else {
            self
        }
    }

    /// Creates a new size from a single size bit of the operand (like MOVEM).
    ///
    /// Size bit means:
    /// - 0 => Word
    /// - 1 => Long
    pub fn from_bit(d: u16) -> Self {
        match d {
            0 => Self::Word,
            1 => Self::Long,
            _ => panic!("[Size::from_bit] Wrong size : expected 0 or 1, got {}", d),
        }
    }

    /// Creates a new size from the size bits of a MOVE or MOVEA instruction.
    ///
    /// - 1 => Byte
    /// - 3 => Word
    /// - 2 => Long
    pub fn from_move(d: u16) -> Self {
        match d {
            1 => Self::Byte,
            3 => Self::Word,
            2 => Self::Long,
            _ => panic!("[Size::from_move] Wrong Size : expected 1, 3 or 2, got {}", d),
        }
    }
}

impl From<u16> for Size {
    /// Creates a new size from the primary size bits.
    ///
    /// Size bits must be:
    /// - 0 => Byte
    /// - 1 => Word
    /// - 2 => Long
    fn from(d: u16) -> Self {
        match d {
            0 => Self::Byte,
            1 => Self::Word,
            2 => Self::Long,
            _ => panic!("[Size::from<u16>] Wrong size : expected 0, 1 or 2, got {}", d),
        }
    }
}

pub(super) enum Operands {
    /// ANDI/EORI/ORI CCR/SR, ILLEGAL, NOP, RESET, RTE, RTR, RTS, STOP, TRAPV 14
    NoOperands,
    /// BCHG, BCLR, BSET, BTST, JMP, JSR, MOVE (f) SR CCR, NBCD, PEA, TAS 12
    EffectiveAddress(EffectiveAddress),
    /// ADDI, ANDI, CLR, CMPI, EORI, NEG, NEGX, NOT, ORI, SUBI, TST 11
    SizeEffectiveAddress(Size, EffectiveAddress),
    /// BCHG, BCLR, BSET, BTST (dynamic), CHK, DIVS, DIVU, LEA, MULS, MULU 10
    RegisterEffectiveAddress(u8, EffectiveAddress),
    /// MOVEP
    RegisterDirectionSizeRegister(u8, Direction, Size, u8),
    /// MOVEA
    SizeRegisterEffectiveAddress(Size, u8, EffectiveAddress),
    /// MOVE
    SizeEffectiveAddressAddresingMode(Size, EffectiveAddress, EffectiveAddress),
    /// EXG
    RegisterOpmodeRegister(u8, u8, u8),
    /// EXT
    SizeRegister(Size, u8),
    /// TRAP
    Vector(u8),
    /// LINK, UNLK
    Register(u8),
    /// MOVE USP
    DirectionRegister(Direction, u8),
    /// MOVEM
    DirectionSizeEffectiveAddress(Direction, Size, EffectiveAddress),
    /// ADDQ, SUBQ 
    DataSizeEffectiveAddress(u8, Size, EffectiveAddress),
    /// Scc
    ConditionEffectiveAddress(u8, EffectiveAddress),
    /// DBcc
    ConditionRegister(u8, u8),
    /// BRA, BSR
    Displacement(i8),
    /// Bcc
    ConditionDisplacement(u8, i8),
    /// MOVEQ
    RegisterData(u8, u8),
    /// ADD, AND, EOR, OR, SUB
    RegisterDirectionSizeEffectiveAddress(u8, Direction, Size, EffectiveAddress),
    /// ADDA, CMPA, SUBA
    RegisterSizeEffectiveAddress(u8, Size, EffectiveAddress),
    /// ABCD, ADDX, SBCD, SUBX
    RegisterSizeModeRegister(u8, Size, u8, u8),
    /// ASd, LSd, ROd, ROXd
    DirectionEffectiveAddress(Direction, EffectiveAddress),
    /// ASd, LSd, ROs, ROXs
    RotationDirectionSizeModeRegister(u8, Direction, Size, u8, u8),
}
