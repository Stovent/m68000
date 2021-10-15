use super::{M68000, MemoryAccess};
use super::addressing_modes::EffectiveAddress;
use super::decoder::DECODER;
use super::isa::Isa;
use super::memory_access::MemoryIter;
use super::utils::Bits;

/// Specify the direction of the operation.
///
/// `RegisterToMemory` and `MemoryToRegister` are used by MOVEM, MOVEP and MOVE USP.
///
/// `DstReg` and `DstEa` are used by ADD, AND, OR and SUB.
///
/// `Left` and `Right` are used by the Shift and Rotate instructions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
    /// For MOVE USP only.
    UspToRegister,
    /// For MOVE USP only.
    RegisterToUsp,
    /// Register to register operation.
    RegisterToRegister,
    /// Memory to Memory operation.
    MemoryToMemory,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Size {
    Byte = 1,
    Word = 2,
    Long = 4,
}

impl Size {
    // /// returns Word when self is Byte, self otherwise.
    // ///
    // /// This is used in addressing modes, where byte post/pre increment
    // /// increments the register by 2 instead of 1.
    // pub(super) fn as_word_long(self) -> Self {
    //     if self == Self::Byte {
    //         Self::Word
    //     } else {
    //         self
    //     }
    // }

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

    // /// Returns true if it is Size::Byte, false otherwise.
    // #[inline(always)]
    // pub fn byte(self) -> bool {
    //     self == Size::Byte
    // }

    // /// Returns true if it is Size::Word, false otherwise.
    // #[inline(always)]
    // pub fn word(self) -> bool {
    //     self == Size::Word
    // }

    /// Returns true if it is Size::long, false otherwise.
    #[inline(always)]
    pub fn long(self) -> bool {
        self == Size::Long
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

#[derive(Clone, Debug)]
pub(super) enum Operands {
    /// ILLEGAL, NOP, RESET, RTE, RTR, RTS, TRAPV
    NoOperands,
    /// ANDI/EORI/ORI CCR/SR, STOP
    Immediate(u16),
    /// ADDI, ANDI, CMPI, EORI, ORI, SUBI
    SizeEffectiveAddressImmediate(Size, EffectiveAddress, u32),
    /// BCHG, BCLR, BSET, BTST
    EffectiveAddressCount(EffectiveAddress, u8),
    /// JMP, JSR, MOVE (f) SR CCR, NBCD, PEA, TAS
    EffectiveAddress(EffectiveAddress),
    /// CLR, NEG, NEGX, NOT, TST
    SizeEffectiveAddress(Size, EffectiveAddress),
    /// CHK, DIVS, DIVU, LEA, MULS, MULU
    RegisterEffectiveAddress(u8, EffectiveAddress),
    /// MOVEP
    RegisterDirectionSizeRegisterDisp(u8, Direction, Size, u8, i16),
    /// MOVEA
    SizeRegisterEffectiveAddress(Size, u8, EffectiveAddress),
    /// MOVE
    SizeEffectiveAddressEffectiveAddress(Size, EffectiveAddress, EffectiveAddress),
    /// EXG
    RegisterOpmodeRegister(u8, u8, u8),
    /// EXT
    OpmodeRegister(u8, u8),
    /// TRAP
    Vector(u8),
    /// LINK
    RegisterDisp(u8, i16),
    /// SWAP, UNLK
    Register(u8),
    /// MOVE USP
    DirectionRegister(Direction, u8),
    /// MOVEM
    DirectionSizeEffectiveAddressList(Direction, Size, EffectiveAddress, u16),
    /// ADDQ, SUBQ
    DataSizeEffectiveAddress(u8, Size, EffectiveAddress),
    /// Scc
    ConditionEffectiveAddress(u8, EffectiveAddress),
    /// DBcc
    ConditionRegisterDisp(u8, u8, i16),
    /// BRA, BSR
    Displacement(i16),
    /// Bcc
    ConditionDisplacement(u8, i16),
    /// MOVEQ
    RegisterData(u8, i8),
    /// ADD, AND, CMP, EOR, OR, SUB
    RegisterDirectionSizeEffectiveAddress(u8, Direction, Size, EffectiveAddress),
    /// ADDA, CMPA, SUBA
    RegisterSizeEffectiveAddress(u8, Size, EffectiveAddress),
    /// ABCD, ADDX, SBCD, SUBX
    RegisterSizeModeRegister(u8, Size, Direction, u8),
    /// CMPM
    RegisterSizeRegister(u8, Size, u8),
    /// ASm, LSm, ROm, ROXm
    DirectionEffectiveAddress(Direction, EffectiveAddress),
    /// ASr, LSr, ROr, ROXr
    RotationDirectionSizeModeRegister(u8, Direction, Size, u8, u8),
}

/// All these functions returns the operands and the number of extention words used by the instruction.
///
/// The idea is to send an iterator over u16 values, starting at the first extention word.
impl<M: MemoryAccess> M68000<M> {
    /// ILLEGAL, NOP, RESET, RTE, RTR, RTS, TRAPV
    pub(super) fn no_operands(_: u16, _: &mut MemoryIter) -> (Operands, usize) {
        (Operands::NoOperands, 0)
    }

    /// ANDI/EORI/ORI CCR/SR, STOP
    pub(super) fn immediate(_: u16, memory: &mut MemoryIter) -> (Operands, usize) {
        let imm = memory.next().unwrap(); // get immediate word
        (Operands::Immediate(imm), 2)
    }

    /// ADDI, ANDI, CMPI, EORI, ORI, SUBI
    pub(super) fn size_effective_address_immediate(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
        let mut len = 0;

        let size = Size::from(opcode.bits::<6, 7>());

        let imm = if size.long() {
            len += 4;
            let high = memory.next().unwrap();
            let low = memory.next().unwrap();
            (high as u32) << 16 | low as u32
        } else {
            len += 2;
            memory.next().unwrap() as u32
        };

        let ea = EffectiveAddress::new(opcode, Some(size), memory);
        len += ea.ext.len();

        (Operands::SizeEffectiveAddressImmediate(size, ea, imm), len)
    }

    /// BCHG, BCLR, BSET, BTST
    pub(super) fn effective_address_count(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
        let mut len = 0;
        let count = if opcode.bits::<8, 8>() != 0 { // dynamic bit number
            opcode.bits::<9, 11>() as u8
        } else { // Static bit number
            len += 2;
            memory.next().unwrap() as u8
        };

        let mut ea = EffectiveAddress::new(opcode, None, memory);
        ea.size = if ea.mode.drd() { Some(Size::Long) } else { Some(Size::Byte) };
        len += ea.ext.len();

        (Operands::EffectiveAddressCount(ea, count), len)
    }

    /// JMP, JSR, MOVE (f) SR CCR, NBCD, PEA, TAS
    pub(super) fn effective_address(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
        let mut len = 0;
        let isa = DECODER[opcode as usize];

        let size = if isa == Isa::Nbcd || isa == Isa::Tas {
            Some(Size::Byte)
        } else if isa == Isa::Moveccr || isa == Isa::Movefsr || isa == Isa::Movesr {
            Some(Size::Word)
        } else if isa == Isa::Pea {
            Some(Size::Long)
        } else {
            None
        };

        let ea = EffectiveAddress::new(opcode, size, memory);
        len += ea.ext.len();
        (Operands::EffectiveAddress(ea), len)
    }

    /// CLR, NEG, NEGX, NOT, TST
    pub(super) fn size_effective_address(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
        let mut len = 0;
        let size = Size::from(opcode.bits::<6, 7>());
        let ea = EffectiveAddress::new(opcode, Some(size), memory);
        len += ea.ext.len();
        (Operands::SizeEffectiveAddress(size, ea), len)
    }

    /// CHK, DIVS, DIVU, LEA, MULS, MULU
    pub(super) fn register_effective_address(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
        let mut len = 0;
        let isa = DECODER[opcode as usize];

        let reg = opcode.bits::<9, 11>() as u8;
        let size = if isa == Isa::Lea {
            Some(Size::Long)
        } else {
            Some(Size::Word)
        };

        let ea = EffectiveAddress::new(opcode, size, memory);
        len += ea.ext.len();
        (Operands::RegisterEffectiveAddress(reg, ea), len)
    }

    /// MOVEP
    pub(super) fn register_direction_size_register_displacement(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
        let dreg = opcode.bits::<9, 11>() as u8;
        let dir = if opcode.bits::<7, 7>() != 0 { Direction::RegisterToMemory } else { Direction::MemoryToRegister };
        let size = if opcode.bits::<6, 6>() != 0 { Size::Long } else { Size::Word };
        let areg = opcode.bits::<0, 2>() as u8;
        let disp = memory.next().unwrap() as i16;
        (Operands::RegisterDirectionSizeRegisterDisp(dreg, dir, size, areg, disp), 2)
    }

    /// MOVEA
    pub(super) fn size_register_effective_address(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
        let mut len = 0;
        let size = Size::from_move(opcode.bits::<12, 13>());
        let areg = opcode.bits::<9, 11>() as u8;
        let ea = EffectiveAddress::new(opcode, Some(size), memory);
        len += ea.ext.len();
        (Operands::SizeRegisterEffectiveAddress(size, areg, ea), len)
    }

    /// MOVE
    pub(super) fn size_effective_address_effective_address(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
        let mut len = 0;
        let size = Size::from_move(opcode.bits::<12, 13>());

        let src = EffectiveAddress::new(opcode, Some(size), memory);
        len += src.ext.len();

        let dst = EffectiveAddress::new_move(opcode, Some(size), memory);
        len += dst.ext.len();

        (Operands::SizeEffectiveAddressEffectiveAddress(size, dst, src), len)
    }

    /// EXG
    pub(super) fn register_opmode_register(opcode: u16, _: &mut MemoryIter) -> (Operands, usize) {
        let regl = opcode.bits::<9, 11>() as u8;
        let opmode = opcode.bits::<3, 7>() as u8;
        let regr = opcode.bits::<0, 2>() as u8;
        (Operands::RegisterOpmodeRegister(regl, opmode, regr), 0)
    }

    /// EXT
    pub(super) fn opmode_register(opcode: u16, _: &mut MemoryIter) -> (Operands, usize) {
        let opmode = opcode.bits::<6, 8>() as u8;
        let reg = opcode.bits::<0, 2>() as u8;
        (Operands::OpmodeRegister(opmode, reg), 0)
    }

    /// TRAP
    pub(super) fn vector(opcode: u16, _: &mut MemoryIter) -> (Operands, usize) {
        let vector = opcode.bits::<0, 3>() as u8;
        (Operands::Vector(vector), 0)
    }

    /// LINK
    pub(super) fn register_displacement(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
        let reg = opcode.bits::<0, 2>() as u8;
        let disp = memory.next().unwrap() as i16;
        (Operands::RegisterDisp(reg, disp), 2)
    }

    /// UNLK
    pub(super) fn register(opcode: u16, _: &mut MemoryIter) -> (Operands, usize) {
        let reg = opcode.bits::<0, 2>() as u8;
        (Operands::Register(reg), 0)
    }

    /// MOVE USP
    pub(super) fn direction_register(opcode: u16, _: &mut MemoryIter) -> (Operands, usize) {
        let dir = if opcode.bits::<3, 3>() != 0 { Direction::UspToRegister } else { Direction::RegisterToUsp };
        let reg = opcode.bits::<0, 2>() as u8;
        (Operands::DirectionRegister(dir, reg), 0)
    }

    /// MOVEM
    pub(super) fn direction_size_effective_address_list(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
        let mut len = 2;
        let list = memory.next().unwrap();
        let dir = if opcode.bits::<10, 10>() != 0 { Direction::MemoryToRegister } else { Direction::RegisterToMemory };
        let size = Size::from_bit(opcode.bits::<6, 6>());

        let ea = EffectiveAddress::new(opcode, Some(size), memory);
        len += ea.ext.len();

        (Operands::DirectionSizeEffectiveAddressList(dir, size, ea, list), len)
    }

    /// ADDQ, SUBQ
    pub(super) fn data_size_effective_address(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
        let mut len = 0;
        let data = opcode.bits::<9, 11>() as u8;
        let size = Size::from(opcode.bits::<6, 7>());

        let ea = EffectiveAddress::new(opcode, Some(size), memory);
        len += ea.ext.len();

        (Operands::DataSizeEffectiveAddress(data, size, ea), len)
    }

    /// Scc
    pub(super) fn condition_effective_address(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
        let mut len = 0;
        let condition = opcode.bits::<8, 11>() as u8;

        let ea = EffectiveAddress::new(opcode, Some(Size::Byte), memory);
        len += ea.ext.len();

        (Operands::ConditionEffectiveAddress(condition, ea), len)
    }

    /// DBcc
    pub(super) fn condition_register_displacement(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
        let disp = memory.next().unwrap() as i16;
        let condition = opcode.bits::<8, 11>() as u8;
        let reg = opcode.bits::<0, 2>() as u8;
        (Operands::ConditionRegisterDisp(condition, reg, disp), 2)
    }

    /// BRA, BSR
    pub(super) fn displacement(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
        let mut len = 0;
        let mut disp = opcode as i8 as i16;
        if disp == 0 {
            len += 2;
            disp = memory.next().unwrap() as i16;
        }
        (Operands::Displacement(disp), len)
    }

    /// Bcc
    pub(super) fn condition_displacement(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
        let mut len = 0;
        let mut disp = opcode as i8 as i16;
        if disp == 0 {
            len += 2;
            disp = memory.next().unwrap() as i16;
        }
        let condition = opcode.bits::<8, 11>() as u8;
        (Operands::ConditionDisplacement(condition, disp), len)
    }

    /// MOVEQ
    pub(super) fn register_data(opcode: u16, _: &mut MemoryIter) -> (Operands, usize) {
        let reg = opcode.bits::<9, 11>() as u8;
        let data = opcode as i8;
        (Operands::RegisterData(reg, data), 0)
    }

    /// ADD, AND, CMP, EOR, OR, SUB
    pub(super) fn register_direction_size_effective_address(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
        let mut len = 0;
        let reg = opcode.bits::<9, 11>() as u8;
        let dir = if opcode.bits::<8, 8>() != 0 { Direction::DstEa } else { Direction::DstReg }; // CMP ignores it
        let size = Size::from(opcode.bits::<6, 7>());

        let ea = EffectiveAddress::new(opcode, Some(size), memory);
        len += ea.ext.len();

        (Operands::RegisterDirectionSizeEffectiveAddress(reg, dir, size, ea), len)
    }

    /// ADDA, CMPA, SUBA
    pub(super) fn register_size_effective_address(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
        let mut len = 0;
        let reg = opcode.bits::<9, 11>() as u8;
        let size = Size::from_bit(opcode.bits::<11, 11>());

        let ea = EffectiveAddress::new(opcode, Some(size), memory);
        len += ea.ext.len();

        (Operands::RegisterSizeEffectiveAddress(reg, size, ea), len)
    }

    /// ABCD, ADDX, SBCD, SUBX
    pub(super) fn register_size_mode_register(opcode: u16, _: &mut MemoryIter) -> (Operands, usize) {
        let regl = opcode.bits::<9, 11>() as u8;
        let size = Size::from(opcode.bits::<6, 7>());
        let mode = if opcode.bits::<3, 3>() != 0 { Direction::MemoryToMemory } else { Direction::RegisterToRegister };
        let regr = opcode.bits::<0, 2>() as u8;
        (Operands::RegisterSizeModeRegister(regl, size, mode, regr), 0)
    }

    /// CMPM
    pub(super) fn register_size_register(opcode: u16, _: &mut MemoryIter) -> (Operands, usize) {
        let regl = opcode.bits::<9, 11>() as u8;
        let size = Size::from(opcode.bits::<6, 7>());
        let regr = opcode.bits::<0, 2>() as u8;
        (Operands::RegisterSizeRegister(regl, size, regr), 0)
    }

    /// ASm, LSm, ROm, ROXm
    pub(super) fn direction_effective_address(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
        let mut len = 0;
        let dir = if opcode.bits::<8, 8>() != 0 { Direction::Left } else { Direction::Right };
        let ea = EffectiveAddress::new(opcode, Some(Size::Byte), memory);
        len += ea.ext.len();
        (Operands::DirectionEffectiveAddress(dir, ea), len)
    }

    /// ASr, LSr, ROr, ROXr
    pub(super) fn rotation_direction_size_mode_register(opcode: u16, _: &mut MemoryIter) -> (Operands, usize) {
        let count = opcode.bits::<9, 11>() as u8;
        let dir = if opcode.bits::<8, 8>() != 0 { Direction::Left } else { Direction::Right };
        let size = Size::from(opcode.bits::<6, 7>());
        let mode = opcode.bits::<5, 5>() as u8;
        let reg = opcode.bits::<0, 2>() as u8;
        (Operands::RotationDirectionSizeModeRegister(count, dir, size, mode, reg), 0)
    }
}
