//! Instruction-related structs, enums and functions.
//!
//! The functions returns the operands and the number of extention words used by the instruction.
//! They take as parameters the opcode of the instruction and an iterator over the extention words.

use crate::addressing_modes::EffectiveAddress;
use crate::decoder::DECODER;
use crate::isa::Isa;
use crate::memory_access::MemoryIter;
use crate::utils::bits;

/// M68000 instruction.
pub struct Instruction {
    /// The opcode itself.
    pub opcode: u16,
    /// The address of the instruction.
    pub pc: u32,
    /// The operands.
    pub operands: Operands,
}

/// Specify the direction of the operation.
///
/// `RegisterToMemory` and `MemoryToRegister` are used by MOVEM and MOVEP.
///
/// `DstReg` and `DstEa` are used by ADD, AND, OR and SUB.
///
/// `Left` and `Right` are used by the Shift and Rotate instructions.
///
/// `UspToRegister` and `RegisterToUsp` are used by MOVE USP.
///
/// `RegisterToRegister` and `MemoryToMemory` is used by ABCD, ADDX, SBCD and SUBX.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Direction {
    /// Transfert from a register to memory.
    RegisterToMemory,
    /// Transfert from memory to a register.
    MemoryToRegister,
    /// Destination is a register.
    DstReg,
    /// Destination is in memory.
    DstEa,
    /// Left shift or rotation.
    Left,
    /// Right shift or rotation.
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

impl std::fmt::Display for Direction {
    /// Disassembles the `Left` (`"L"`) or `Right` (`"R"`) direction.
    ///
    /// Other directions are not disassembled and does nothing.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Left => write!(f, "L"),
            Self::Right => write!(f, "R"),
            _ => Ok(()),
        }
    }
}

/// Size of an operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Size {
    Byte = 1,
    Word = 2,
    Long = 4,
}

impl Size {
    /// Returns Word when self is Byte, self otherwise.
    ///
    /// This is used in addressing modes, where byte post/pre increment
    /// increments the register by 2 instead of 1.
    pub fn as_word_long(self) -> Self {
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

    /// Returns true if it is Size::Byte, false otherwise.
    #[inline(always)]
    pub fn byte(self) -> bool {
        self == Self::Byte
    }

    /// Returns true if it is Size::Word, false otherwise.
    #[inline(always)]
    pub fn word(self) -> bool {
        self == Self::Word
    }

    /// Returns true if it is Size::long, false otherwise.
    #[inline(always)]
    pub fn long(self) -> bool {
        self == Self::Long
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

impl std::fmt::Display for Size {
    /// Disassembles to `"B"`, `"W"` or `"L"`
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Size::Byte => write!(f, "B"),
            Size::Word => write!(f, "W"),
            Size::Long => write!(f, "L"),
        }
    }
}

/// Operands of an instruction.
#[derive(Clone, Debug)]
pub enum Operands {
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

/// In the returned values, the first operand is the left-most operand in the instruction word (high-order bits).
/// The last operand is the right-most operand in the instruction word (low-order bits) or the extention words (if any).
impl Operands {
    /// ANDI/EORI/ORI CCR/SR, STOP
    pub fn immediate(&self) -> u16 {
        match *self {
            Self::Immediate(i) => i,
            _ => panic!("[Operands::immediate]"),
        }
    }

    /// ADDI, ANDI, CMPI, EORI, ORI, SUBI
    pub fn size_effective_address_immediate(&mut self) -> (Size, &mut EffectiveAddress, u32) {
        match &mut *self {
            Self::SizeEffectiveAddressImmediate(s, e, i) => (*s, e, *i),
            _ => panic!("[Operands::size_effective_address_immediate]"),
        }
    }

    /// BCHG, BCLR, BSET, BTST
    pub fn effective_address_count(&mut self) -> (&mut EffectiveAddress, u8) {
        match &mut *self {
            Self::EffectiveAddressCount(e, c) => (e, *c),
            _ => panic!("[Operands::effective_address_count]"),
        }
    }

    /// JMP, JSR, MOVE (f) SR CCR, NBCD, PEA, TAS
    pub fn effective_address(&mut self) -> &mut EffectiveAddress {
        match &mut *self {
            Self::EffectiveAddress(e) => e,
            _ => panic!("[Operands::effective_address]"),
        }
    }

    /// CLR, NEG, NEGX, NOT, TST
    pub fn size_effective_address(&mut self) -> (Size, &mut EffectiveAddress) {
        match &mut *self {
            Self::SizeEffectiveAddress(s, e) => (*s, e),
            _ => panic!("[Operands::size_effective_address]"),
        }
    }

    /// CHK, DIVS, DIVU, LEA, MULS, MULU
    pub fn register_effective_address(&mut self) -> (u8, &mut EffectiveAddress) {
        match &mut *self {
            Self::RegisterEffectiveAddress(r, e) => (*r, e),
            _ => panic!("[Operands::register_effective_address]"),
        }
    }

    /// MOVEP
    pub fn register_direction_size_register_displacement(&self) -> (u8, Direction, Size, u8, i16) {
        match *self {
            Self::RegisterDirectionSizeRegisterDisp(r, d, s, rr, dd) => (r, d, s, rr, dd),
            _ => panic!("[Operands::register_direction_size_register_disp]"),
        }
    }

    /// MOVEA
    pub fn size_register_effective_address(&mut self) -> (Size, u8, &mut EffectiveAddress) {
        match &mut *self {
            Self::SizeRegisterEffectiveAddress(s, r, e) => (*s, *r, e),
            _ => panic!("[Operands::size_register_effective_address]"),
        }
    }

    /// MOVE
    pub fn size_effective_address_effective_address(&mut self) -> (Size, &mut EffectiveAddress, &mut EffectiveAddress) {
        match &mut *self {
            Self::SizeEffectiveAddressEffectiveAddress(s, e, ee) => (*s, e, ee),
            _ => panic!("[Operands::size_effective_address_effective_address]"),
        }
    }

    /// EXG
    pub fn register_opmode_register(&self) -> (u8, u8, u8) {
        match *self {
            Self::RegisterOpmodeRegister(r, o, rr) => (r, o, rr),
            _ => panic!("[Operands::register_opmode_register]"),
        }
    }

    /// EXT
    pub fn opmode_register(&self) -> (u8, u8) {
        match *self {
            Self::OpmodeRegister(o, r) => (o, r),
            _ => panic!("[Operands::opmode_register]"),
        }
    }

    /// TRAP
    pub fn vector(&self) -> u8 {
        match *self {
            Self::Vector(v) => v,
            _ => panic!("[Operands::vector]"),
        }
    }

    /// LINK
    pub fn register_displacement(&self) -> (u8, i16) {
        match *self {
            Self::RegisterDisp(r, d) => (r, d),
            _ => panic!("[Operands::register_disp]"),
        }
    }

    /// SWAP, UNLK
    pub fn register(&self) -> u8 {
        match *self {
            Self::Register(r) => r,
            _ => panic!("[Operands::register]"),
        }
    }

    /// MOVE USP
    pub fn direction_register(&self) -> (Direction, u8) {
        match *self {
            Self::DirectionRegister(d, r) => (d, r),
            _ => panic!("[Operands::direction_register]"),
        }
    }

    /// MOVEM
    pub fn direction_size_effective_address_list(&mut self) -> (Direction, Size, &mut EffectiveAddress, u16) {
        match &mut *self {
            Self::DirectionSizeEffectiveAddressList(d, s, e, l) => (*d, *s, e, *l),
            _ => panic!("[Operands::direction_size_effective_address_list]"),
        }
    }

    /// ADDQ, SUBQ
    pub fn data_size_effective_address(&mut self) -> (u8, Size, &mut EffectiveAddress) {
        match &mut *self {
            Self::DataSizeEffectiveAddress(d, s, e) => (*d, *s, e),
            _ => panic!("[Operands::data_size_effective_address]"),
        }
    }

    /// Scc
    pub fn condition_effective_address(&mut self) -> (u8, &mut EffectiveAddress) {
        match &mut *self {
            Self::ConditionEffectiveAddress(c, e) => (*c, e),
            _ => panic!("[Operands::condition_effective_address]"),
        }
    }

    /// DBcc
    pub fn condition_register_displacement(&self) -> (u8, u8, i16) {
        match *self {
            Self::ConditionRegisterDisp(c, r, d) => (c, r, d),
            _ => panic!("[Operands::condition_register_disp]"),
        }
    }

    /// BRA, BSR
    pub fn displacement(&self) -> i16 {
        match *self {
            Self::Displacement(d) => d,
            _ => panic!("[Operands::displacement]"),
        }
    }

    /// Bcc
    pub fn condition_displacement(&self) -> (u8, i16) {
        match *self {
            Self::ConditionDisplacement(c, d) => (c, d),
            _ => panic!("[Operands::condition_displacement]"),
        }
    }

    /// MOVEQ
    pub fn register_data(&self) -> (u8, i8) {
        match *self {
            Self::RegisterData(r, d) => (r, d),
            _ => panic!("[Operands::register_data]"),
        }
    }

    /// ADD, AND, CMP, EOR, OR, SUB
    pub fn register_direction_size_effective_address(&mut self) -> (u8, Direction, Size, &mut EffectiveAddress) {
        match &mut *self {
            Self::RegisterDirectionSizeEffectiveAddress(r, d, s, e) => (*r, *d, *s, e),
            _ => panic!("[Operands::register_direction_size_effective_address]"),
        }
    }

    /// ADDA, CMPA, SUBA
    pub fn register_size_effective_address(&mut self) -> (u8, Size, &mut EffectiveAddress) {
        match &mut *self {
            Self::RegisterSizeEffectiveAddress(r, s, e) => (*r, *s, e),
            _ => panic!("[Operands::register_size_effective_address]"),
        }
    }

    /// ABCD, ADDX, SBCD, SUBX
    pub fn register_size_mode_register(&self) -> (u8, Size, Direction, u8) {
        match *self {
            Self::RegisterSizeModeRegister(r, s, m, rr) => (r, s, m, rr),
            _ => panic!("[Operands::register_size_mode_register]"),
        }
    }

    /// CMPM
    pub fn register_size_register(&self) -> (u8, Size, u8) {
        match *self {
            Self::RegisterSizeRegister(r, s, rr) => (r, s, rr),
            _ => panic!("[Operands::register_size_register]"),
        }
    }

    /// ASm, LSm, ROm, ROXm
    pub fn direction_effective_address(&mut self) -> (Direction, &mut EffectiveAddress) {
        match &mut *self {
            Self::DirectionEffectiveAddress(d, e) => (*d, e),
            _ => panic!("[Operands::direction_effective_address]"),
        }
    }

    /// ASr, LSr, ROr, ROXr
    pub fn rotation_direction_size_mode_register(&self) -> (u8, Direction, Size, u8, u8) {
        match *self {
            Self::RotationDirectionSizeModeRegister(r, d, s, m, rr) => (r, d, s, m, rr),
            _ => panic!("[Operands::rotation_direction_size_mode_register]"),
        }
    }
}

/// ILLEGAL, NOP, RESET, RTE, RTR, RTS, TRAPV
pub fn no_operands(_: u16, _: &mut MemoryIter) -> (Operands, usize) {
    (Operands::NoOperands, 0)
}

/// ANDI/EORI/ORI CCR/SR, STOP
pub fn immediate(_: u16, memory: &mut MemoryIter) -> (Operands, usize) {
    let imm = memory.next().unwrap(); // get immediate word
    (Operands::Immediate(imm), 2)
}

/// ADDI, ANDI, CMPI, EORI, ORI, SUBI
pub fn size_effective_address_immediate(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
    let mut len = 0;

    let size = Size::from(bits(opcode, 6, 7));

    let imm = if size.long() {
        len += 4;
        let high = memory.next().unwrap();
        let low = memory.next().unwrap();
        (high as u32) << 16 | low as u32
    } else {
        len += 2;
        memory.next().unwrap() as u32
    };

    let ea = EffectiveAddress::from_opcode(opcode, Some(size), memory);
    len += ea.ext.len();

    (Operands::SizeEffectiveAddressImmediate(size, ea, imm), len)
}

/// BCHG, BCLR, BSET, BTST
pub fn effective_address_count(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
    let mut len = 0;
    let count = if bits(opcode, 8, 8) != 0 { // dynamic bit number
        bits(opcode, 9, 11) as u8
    } else { // Static bit number
        len += 2;
        memory.next().unwrap() as u8
    };

    let size = if bits(opcode, 3, 5) == 0 { Some(Size::Long) } else { Some(Size::Byte) };
    let mut ea = EffectiveAddress::from_opcode(opcode, size, memory);
    ea.size = if ea.mode.drd() { Some(Size::Long) } else { Some(Size::Byte) };
    len += ea.ext.len();

    (Operands::EffectiveAddressCount(ea, count), len)
}

/// JMP, JSR, MOVE (f) SR CCR, NBCD, PEA, TAS
pub fn effective_address(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
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

    let ea = EffectiveAddress::from_opcode(opcode, size, memory);
    len += ea.ext.len();
    (Operands::EffectiveAddress(ea), len)
}

/// CLR, NEG, NEGX, NOT, TST
pub fn size_effective_address(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
    let mut len = 0;
    let size = Size::from(bits(opcode, 6, 7));
    let ea = EffectiveAddress::from_opcode(opcode, Some(size), memory);
    len += ea.ext.len();
    (Operands::SizeEffectiveAddress(size, ea), len)
}

/// CHK, DIVS, DIVU, LEA, MULS, MULU
pub fn register_effective_address(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
    let mut len = 0;
    let isa = DECODER[opcode as usize];

    let reg = bits(opcode, 9, 11) as u8;
    let size = if isa == Isa::Lea {
        Some(Size::Long)
    } else {
        Some(Size::Word)
    };

    let ea = EffectiveAddress::from_opcode(opcode, size, memory);
    len += ea.ext.len();
    (Operands::RegisterEffectiveAddress(reg, ea), len)
}

/// MOVEP
pub fn register_direction_size_register_displacement(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
    let dreg = bits(opcode, 9, 11) as u8;
    let dir = if bits(opcode, 7, 7) != 0 { Direction::RegisterToMemory } else { Direction::MemoryToRegister };
    let size = if bits(opcode, 6, 6) != 0 { Size::Long } else { Size::Word };
    let areg = bits(opcode, 0, 2) as u8;
    let disp = memory.next().unwrap() as i16;
    (Operands::RegisterDirectionSizeRegisterDisp(dreg, dir, size, areg, disp), 2)
}

/// MOVEA
pub fn size_register_effective_address(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
    let mut len = 0;
    let size = Size::from_move(bits(opcode, 12, 13));
    let areg = bits(opcode, 9, 11) as u8;
    let ea = EffectiveAddress::from_opcode(opcode, Some(size), memory);
    len += ea.ext.len();
    (Operands::SizeRegisterEffectiveAddress(size, areg, ea), len)
}

/// MOVE
pub fn size_effective_address_effective_address(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
    let mut len = 0;
    let size = Size::from_move(bits(opcode, 12, 13));

    let (dst, src) = EffectiveAddress::from_move(opcode, Some(size), memory);
    len += src.ext.len() + dst.ext.len();

    (Operands::SizeEffectiveAddressEffectiveAddress(size, dst, src), len)
}

/// EXG
pub fn register_opmode_register(opcode: u16, _: &mut MemoryIter) -> (Operands, usize) {
    let regl = bits(opcode, 9, 11) as u8;
    let opmode = bits(opcode, 3, 7) as u8;
    let regr = bits(opcode, 0, 2) as u8;
    (Operands::RegisterOpmodeRegister(regl, opmode, regr), 0)
}

/// EXT
pub fn opmode_register(opcode: u16, _: &mut MemoryIter) -> (Operands, usize) {
    let opmode = bits(opcode, 6, 8) as u8;
    let reg = bits(opcode, 0, 2) as u8;
    (Operands::OpmodeRegister(opmode, reg), 0)
}

/// TRAP
pub fn vector(opcode: u16, _: &mut MemoryIter) -> (Operands, usize) {
    let vector = bits(opcode, 0, 3) as u8;
    (Operands::Vector(vector), 0)
}

/// LINK
pub fn register_displacement(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
    let reg = bits(opcode, 0, 2) as u8;
    let disp = memory.next().unwrap() as i16;
    (Operands::RegisterDisp(reg, disp), 2)
}

/// UNLK
pub fn register(opcode: u16, _: &mut MemoryIter) -> (Operands, usize) {
    let reg = bits(opcode, 0, 2) as u8;
    (Operands::Register(reg), 0)
}

/// MOVE USP
pub fn direction_register(opcode: u16, _: &mut MemoryIter) -> (Operands, usize) {
    let dir = if bits(opcode, 3, 3) != 0 { Direction::UspToRegister } else { Direction::RegisterToUsp };
    let reg = bits(opcode, 0, 2) as u8;
    (Operands::DirectionRegister(dir, reg), 0)
}

/// MOVEM
pub fn direction_size_effective_address_list(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
    let mut len = 2;
    let list = memory.next().unwrap();
    let dir = if bits(opcode, 10, 10) != 0 { Direction::MemoryToRegister } else { Direction::RegisterToMemory };
    let size = Size::from_bit(bits(opcode, 6, 6));

    let ea = EffectiveAddress::from_opcode(opcode, Some(size), memory);
    len += ea.ext.len();

    (Operands::DirectionSizeEffectiveAddressList(dir, size, ea, list), len)
}

/// ADDQ, SUBQ
pub fn data_size_effective_address(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
    let mut len = 0;
    let data = bits(opcode, 9, 11) as u8;
    let size = Size::from(bits(opcode, 6, 7));

    let ea = EffectiveAddress::from_opcode(opcode, Some(size), memory);
    len += ea.ext.len();

    (Operands::DataSizeEffectiveAddress(data, size, ea), len)
}

/// Scc
pub fn condition_effective_address(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
    let mut len = 0;
    let condition = bits(opcode, 8, 11) as u8;

    let ea = EffectiveAddress::from_opcode(opcode, Some(Size::Byte), memory);
    len += ea.ext.len();

    (Operands::ConditionEffectiveAddress(condition, ea), len)
}

/// DBcc
pub fn condition_register_displacement(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
    let disp = memory.next().unwrap() as i16;
    let condition = bits(opcode, 8, 11) as u8;
    let reg = bits(opcode, 0, 2) as u8;
    (Operands::ConditionRegisterDisp(condition, reg, disp), 2)
}

/// BRA, BSR
pub fn displacement(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
    let mut len = 0;
    let mut disp = opcode as i8 as i16;
    if disp == 0 {
        len += 2;
        disp = memory.next().unwrap() as i16;
    }
    (Operands::Displacement(disp), len)
}

/// Bcc
pub fn condition_displacement(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
    let mut len = 0;
    let mut disp = opcode as i8 as i16;
    if disp == 0 {
        len += 2;
        disp = memory.next().unwrap() as i16;
    }
    let condition = bits(opcode, 8, 11) as u8;
    (Operands::ConditionDisplacement(condition, disp), len)
}

/// MOVEQ
pub fn register_data(opcode: u16, _: &mut MemoryIter) -> (Operands, usize) {
    let reg = bits(opcode, 9, 11) as u8;
    let data = opcode as i8;
    (Operands::RegisterData(reg, data), 0)
}

/// ADD, AND, CMP, EOR, OR, SUB
pub fn register_direction_size_effective_address(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
    let mut len = 0;
    let reg = bits(opcode, 9, 11) as u8;
    let dir = if bits(opcode, 8, 8) != 0 { Direction::DstEa } else { Direction::DstReg }; // CMP and EOR ignores it
    let size = Size::from(bits(opcode, 6, 7));

    let ea = EffectiveAddress::from_opcode(opcode, Some(size), memory);
    len += ea.ext.len();

    (Operands::RegisterDirectionSizeEffectiveAddress(reg, dir, size, ea), len)
}

/// ADDA, CMPA, SUBA
pub fn register_size_effective_address(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
    let mut len = 0;
    let reg = bits(opcode, 9, 11) as u8;
    let size = Size::from_bit(bits(opcode, 8, 8));

    let ea = EffectiveAddress::from_opcode(opcode, Some(size), memory);
    len += ea.ext.len();

    (Operands::RegisterSizeEffectiveAddress(reg, size, ea), len)
}

/// ABCD, ADDX, SBCD, SUBX
pub fn register_size_mode_register(opcode: u16, _: &mut MemoryIter) -> (Operands, usize) {
    let regl = bits(opcode, 9, 11) as u8;
    let size = Size::from(bits(opcode, 6, 7));
    let mode = if bits(opcode, 3, 3) != 0 { Direction::MemoryToMemory } else { Direction::RegisterToRegister };
    let regr = bits(opcode, 0, 2) as u8;
    (Operands::RegisterSizeModeRegister(regl, size, mode, regr), 0)
}

/// CMPM
pub fn register_size_register(opcode: u16, _: &mut MemoryIter) -> (Operands, usize) {
    let regl = bits(opcode, 9, 11) as u8;
    let size = Size::from(bits(opcode, 6, 7));
    let regr = bits(opcode, 0, 2) as u8;
    (Operands::RegisterSizeRegister(regl, size, regr), 0)
}

/// ASm, LSm, ROm, ROXm
pub fn direction_effective_address(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
    let mut len = 0;
    let dir = if bits(opcode, 8, 8) != 0 { Direction::Left } else { Direction::Right };
    let ea = EffectiveAddress::from_opcode(opcode, Some(Size::Byte), memory);
    len += ea.ext.len();
    (Operands::DirectionEffectiveAddress(dir, ea), len)
}

/// ASr, LSr, ROr, ROXr
pub fn rotation_direction_size_mode_register(opcode: u16, _: &mut MemoryIter) -> (Operands, usize) {
    let count = bits(opcode, 9, 11) as u8;
    let dir = if bits(opcode, 8, 8) != 0 { Direction::Left } else { Direction::Right };
    let size = Size::from(bits(opcode, 6, 7));
    let mode = bits(opcode, 5, 5) as u8;
    let reg = bits(opcode, 0, 2) as u8;
    (Operands::RotationDirectionSizeModeRegister(count, dir, size, mode, reg), 0)
}
