//! Instruction-related structs, enums and functions.
//!
//! The functions returns the operands and the size in bytes of the extension words.
//! They take as parameters the opcode of the instruction and an iterator over the extension words.

use crate::addressing_modes::AddressingMode;
use crate::decoder::DECODER;
use crate::isa::{Isa, IsaEntry};
use crate::memory_access::MemoryIter;
use crate::utils::bits;

/// M68000 instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Instruction {
    /// The opcode itself.
    pub opcode: u16,
    /// The address of the instruction.
    pub pc: u32,
    /// The operands.
    pub operands: Operands,
}

impl Instruction {
    /// Decodes the given opcode.
    ///
    /// Returns the decoded instruction and the number of bytes of the extension words.
    pub fn from_opcode(opcode: u16, pc: u32, memory: &mut MemoryIter) -> (Self, usize) {
        let isa: Isa = opcode.into();
        let decode = IsaEntry::ISA_ENTRY[isa as usize].decode;
        let (operands, len) = decode(opcode, memory);

        (Instruction {
            opcode,
            pc,
            operands,
        }, len)
    }

    /// Decodes the opcode at the given memory location.
    ///
    /// Returns the decoded instruction and the number of bytes of the instruction (opcode + extension words).
    /// Returns Err when there was an error when reading memory (Access error).
    pub fn from_memory(memory: &mut MemoryIter) -> Result<(Self, usize), u8> {
        let pc = memory.next_addr;
        let opcode = memory.next().unwrap()?;
        let mut res = Self::from_opcode(opcode, pc, memory);
        res.1 += 2; // Add opcode length.
        Ok(res)
    }

    /// Disassemble the intruction.
    pub fn disassemble(&self) -> String {
        let isa: Isa = self.opcode.into();
        (IsaEntry::ISA_ENTRY[isa as usize].disassemble)(self)
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.disassemble())
    }
}

/// Specify the direction of the operation.
///
/// `RegisterToMemory` and `MemoryToRegister` are used by MOVEM and MOVEP.
///
/// `DstReg` and `DstEa` are used by ADD, AND, OR and SUB.
///
/// `Left` and `Right` are used by the Shift and Rotate instructions.
///
/// `RegisterToUsp` and `UspToRegister` are used by MOVE USP.
///
/// `RegisterToRegister` and `MemoryToMemory` are used by ABCD, ADDX, SBCD and SUBX.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    RegisterToUsp,
    /// For MOVE USP only.
    UspToRegister,
    /// Register to register operation.
    RegisterToRegister,
    /// Memory to Memory operation.
    MemoryToMemory,
    /// Exchange Data Registers (EXG only).
    ExchangeData,
    /// Exchange Address Registers (EXG only).
    ExchangeAddress,
    /// Exchange Data and Address Registers (EXG only).
    ExchangeDataAddress,
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
    #[inline(always)]
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
    #[inline(always)]
    pub fn from_bit(d: u16) -> Self {
        match d {
            0 => Self::Word,
            1 => Self::Long,
            _ => panic!("[Size::from_bit] Wrong size : expected 0 or 1, got {}", d),
        }
    }

    /// Returns the binary encoding of the size as used by MOVEM and EXT.
    ///
    /// - Word => 0
    /// - Long => 1
    #[inline(always)]
    pub fn into_bit(self) -> u16 {
        match self {
            Self::Word => 0,
            Self::Long => 1,
            _ => panic!("[Size::into_bit] Wrong size : expected word or long, got {}", self),
        }
    }

    /// Creates a new size from the size bits of a MOVE or MOVEA instruction.
    ///
    /// - 1 => Byte
    /// - 3 => Word
    /// - 2 => Long
    #[inline(always)]
    pub fn from_move(d: u16) -> Self {
        match d {
            1 => Self::Byte,
            3 => Self::Word,
            2 => Self::Long,
            _ => panic!("[Size::from_move] Wrong Size : expected 1, 3 or 2, got {}", d),
        }
    }

    /// Returns the binary encoding of the size as used by MOVE and MOVEA.
    ///
    /// - Byte => 1
    /// - Word => 3
    /// - Long => 2
    #[inline(always)]
    pub const fn into_move(self) -> u16 {
        match self {
            Self::Byte => 1,
            Self::Word => 3,
            Self::Long => 2,
        }
    }

    /// Returns true if it is Size::Byte, false otherwise.
    #[inline(always)]
    pub fn is_byte(self) -> bool {
        self == Self::Byte
    }

    /// Returns true if it is Size::Word, false otherwise.
    #[inline(always)]
    pub fn is_word(self) -> bool {
        self == Self::Word
    }

    /// Returns true if it is Size::long, false otherwise.
    #[inline(always)]
    pub fn is_long(self) -> bool {
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

impl Into<u16> for Size {
    fn into(self) -> u16 {
        match self {
            Size::Byte => 0,
            Size::Word => 1,
            Size::Long => 2,
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operands {
    /// ILLEGAL, NOP, RESET, RTE, RTR, RTS, TRAPV
    NoOperands,
    /// ANDI/EORI/ORI CCR/SR, STOP
    Immediate(u16),
    /// ADDI, ANDI, CMPI, EORI, ORI, SUBI
    SizeEffectiveAddressImmediate(Size, AddressingMode, u32),
    /// BCHG, BCLR, BSET, BTST
    EffectiveAddressCount(AddressingMode, u8),
    /// JMP, JSR, MOVE (f) SR CCR, NBCD, PEA, TAS
    EffectiveAddress(AddressingMode),
    /// CLR, NEG, NEGX, NOT, TST
    SizeEffectiveAddress(Size, AddressingMode),
    /// CHK, DIVS, DIVU, LEA, MULS, MULU
    RegisterEffectiveAddress(u8, AddressingMode),
    /// MOVEP
    RegisterDirectionSizeRegisterDisplacement(u8, Direction, Size, u8, i16),
    /// MOVEA
    SizeRegisterEffectiveAddress(Size, u8, AddressingMode),
    /// MOVE
    SizeEffectiveAddressEffectiveAddress(Size, AddressingMode, AddressingMode),
    /// EXG
    RegisterOpmodeRegister(u8, Direction, u8),
    /// EXT
    OpmodeRegister(u8, u8),
    /// TRAP
    Vector(u8),
    /// LINK
    RegisterDisplacement(u8, i16),
    /// SWAP, UNLK
    Register(u8),
    /// MOVE USP
    DirectionRegister(Direction, u8),
    /// MOVEM
    DirectionSizeEffectiveAddressList(Direction, Size, AddressingMode, u16),
    /// ADDQ, SUBQ
    DataSizeEffectiveAddress(u8, Size, AddressingMode),
    /// Scc
    ConditionEffectiveAddress(u8, AddressingMode),
    /// DBcc
    ConditionRegisterDisplacement(u8, u8, i16),
    /// BRA, BSR
    Displacement(i16),
    /// Bcc
    ConditionDisplacement(u8, i16),
    /// MOVEQ
    RegisterData(u8, i8),
    /// ADD, AND, CMP, EOR, OR, SUB
    RegisterDirectionSizeEffectiveAddress(u8, Direction, Size, AddressingMode),
    /// ADDA, CMPA, SUBA
    RegisterSizeEffectiveAddress(u8, Size, AddressingMode),
    /// ABCD, ADDX, SBCD, SUBX
    RegisterSizeModeRegister(u8, Size, Direction, u8),
    /// CMPM
    RegisterSizeRegister(u8, Size, u8),
    /// ASm, LSm, ROm, ROXm
    DirectionEffectiveAddress(Direction, AddressingMode),
    /// ASr, LSr, ROr, ROXr
    RotationDirectionSizeModeRegister(u8, Direction, Size, u8, u8),
}

/// In the returned values, the first operand is the left-most operand in the instruction word (high-order bits).
/// The last operand is the right-most operand in the instruction word (low-order bits) or the extension words (if any).
impl Operands {
    /// ANDI/EORI/ORI CCR/SR, STOP
    pub const fn immediate(self) -> u16 {
        match self {
            Self::Immediate(i) => i,
            _ => panic!("[Operands::immediate]"),
        }
    }

    /// ADDI, ANDI, CMPI, EORI, ORI, SUBI
    pub const fn size_effective_address_immediate(self) -> (Size, AddressingMode, u32) {
        match self {
            Self::SizeEffectiveAddressImmediate(s, e, i) => (s, e, i),
            _ => panic!("[Operands::size_effective_address_immediate]"),
        }
    }

    /// BCHG, BCLR, BSET, BTST
    pub const fn effective_address_count(self) -> (AddressingMode, u8) {
        match self {
            Self::EffectiveAddressCount(e, c) => (e, c),
            _ => panic!("[Operands::effective_address_count]"),
        }
    }

    /// JMP, JSR, MOVE (f) SR CCR, NBCD, PEA, TAS
    pub const fn effective_address(self) -> AddressingMode {
        match self {
            Self::EffectiveAddress(e) => e,
            _ => panic!("[Operands::effective_address]"),
        }
    }

    /// CLR, NEG, NEGX, NOT, TST
    pub const fn size_effective_address(self) -> (Size, AddressingMode) {
        match self {
            Self::SizeEffectiveAddress(s, e) => (s, e),
            _ => panic!("[Operands::size_effective_address]"),
        }
    }

    /// CHK, DIVS, DIVU, LEA, MULS, MULU
    pub const fn register_effective_address(self) -> (u8, AddressingMode) {
        match self {
            Self::RegisterEffectiveAddress(r, e) => (r, e),
            _ => panic!("[Operands::register_effective_address]"),
        }
    }

    /// MOVEP
    pub const fn register_direction_size_register_displacement(self) -> (u8, Direction, Size, u8, i16) {
        match self {
            Self::RegisterDirectionSizeRegisterDisplacement(r, d, s, rr, dd) => (r, d, s, rr, dd),
            _ => panic!("[Operands::register_direction_size_register_displacement]"),
        }
    }

    /// MOVEA
    pub const fn size_register_effective_address(self) -> (Size, u8, AddressingMode) {
        match self {
            Self::SizeRegisterEffectiveAddress(s, r, e) => (s, r, e),
            _ => panic!("[Operands::size_register_effective_address]"),
        }
    }

    /// MOVE
    pub const fn size_effective_address_effective_address(self) -> (Size, AddressingMode, AddressingMode) {
        match self {
            Self::SizeEffectiveAddressEffectiveAddress(s, e, ee) => (s, e, ee),
            _ => panic!("[Operands::size_effective_address_effective_address]"),
        }
    }

    /// EXG
    pub const fn register_opmode_register(self) -> (u8, Direction, u8) {
        match self {
            Self::RegisterOpmodeRegister(r, o, rr) => (r, o, rr),
            _ => panic!("[Operands::register_opmode_register]"),
        }
    }

    /// EXT
    pub const fn opmode_register(self) -> (u8, u8) {
        match self {
            Self::OpmodeRegister(o, r) => (o, r),
            _ => panic!("[Operands::opmode_register]"),
        }
    }

    /// TRAP
    pub const fn vector(self) -> u8 {
        match self {
            Self::Vector(v) => v,
            _ => panic!("[Operands::vector]"),
        }
    }

    /// LINK
    pub const fn register_displacement(self) -> (u8, i16) {
        match self {
            Self::RegisterDisplacement(r, d) => (r, d),
            _ => panic!("[Operands::register_displacement]"),
        }
    }

    /// SWAP, UNLK
    pub const fn register(self) -> u8 {
        match self {
            Self::Register(r) => r,
            _ => panic!("[Operands::register]"),
        }
    }

    /// MOVE USP
    pub const fn direction_register(self) -> (Direction, u8) {
        match self {
            Self::DirectionRegister(d, r) => (d, r),
            _ => panic!("[Operands::direction_register]"),
        }
    }

    /// MOVEM
    pub const fn direction_size_effective_address_list(self) -> (Direction, Size, AddressingMode, u16) {
        match self {
            Self::DirectionSizeEffectiveAddressList(d, s, e, l) => (d, s, e, l),
            _ => panic!("[Operands::direction_size_effective_address_list]"),
        }
    }

    /// ADDQ, SUBQ
    pub const fn data_size_effective_address(self) -> (u8, Size, AddressingMode) {
        match self {
            Self::DataSizeEffectiveAddress(d, s, e) => (d, s, e),
            _ => panic!("[Operands::data_size_effective_address]"),
        }
    }

    /// Scc
    pub const fn condition_effective_address(self) -> (u8, AddressingMode) {
        match self {
            Self::ConditionEffectiveAddress(c, e) => (c, e),
            _ => panic!("[Operands::condition_effective_address]"),
        }
    }

    /// DBcc
    pub const fn condition_register_displacement(self) -> (u8, u8, i16) {
        match self {
            Self::ConditionRegisterDisplacement(c, r, d) => (c, r, d),
            _ => panic!("[Operands::condition_register_displacement]"),
        }
    }

    /// BRA, BSR
    pub const fn displacement(self) -> i16 {
        match self {
            Self::Displacement(d) => d,
            _ => panic!("[Operands::displacement]"),
        }
    }

    /// Bcc
    pub const fn condition_displacement(self) -> (u8, i16) {
        match self {
            Self::ConditionDisplacement(c, d) => (c, d),
            _ => panic!("[Operands::condition_displacement]"),
        }
    }

    /// MOVEQ
    pub const fn register_data(self) -> (u8, i8) {
        match self {
            Self::RegisterData(r, d) => (r, d),
            _ => panic!("[Operands::register_data]"),
        }
    }

    /// ADD, AND, CMP, EOR, OR, SUB
    pub const fn register_direction_size_effective_address(self) -> (u8, Direction, Size, AddressingMode) {
        match self {
            Self::RegisterDirectionSizeEffectiveAddress(r, d, s, e) => (r, d, s, e),
            _ => panic!("[Operands::register_direction_size_effective_address]"),
        }
    }

    /// ADDA, CMPA, SUBA
    pub const fn register_size_effective_address(self) -> (u8, Size, AddressingMode) {
        match self {
            Self::RegisterSizeEffectiveAddress(r, s, e) => (r, s, e),
            _ => panic!("[Operands::register_size_effective_address]"),
        }
    }

    /// ABCD, ADDX, SBCD, SUBX
    pub const fn register_size_mode_register(self) -> (u8, Size, Direction, u8) {
        match self {
            Self::RegisterSizeModeRegister(r, s, m, rr) => (r, s, m, rr),
            _ => panic!("[Operands::register_size_mode_register]"),
        }
    }

    /// CMPM
    pub const fn register_size_register(self) -> (u8, Size, u8) {
        match self {
            Self::RegisterSizeRegister(r, s, rr) => (r, s, rr),
            _ => panic!("[Operands::register_size_register]"),
        }
    }

    /// ASm, LSm, ROm, ROXm
    pub const fn direction_effective_address(self) -> (Direction, AddressingMode) {
        match self {
            Self::DirectionEffectiveAddress(d, e) => (d, e),
            _ => panic!("[Operands::direction_effective_address]"),
        }
    }

    /// ASr, LSr, ROr, ROXr
    pub const fn rotation_direction_size_mode_register(self) -> (u8, Direction, Size, u8, u8) {
        match self {
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
    let imm = memory.next().unwrap().expect("Access error occured when fetching immediate operand."); // Get immediate word
    (Operands::Immediate(imm), 2)
}

/// ADDI, ANDI, CMPI, EORI, ORI, SUBI
pub fn size_effective_address_immediate(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
    let mut len = 2;

    let size = Size::from(bits(opcode, 6, 7));

    let imm = if size.is_long() {
        len += 2;
        let high = memory.next().unwrap().expect("Access error occured when fetching immediate operand high.");
        let low = memory.next().unwrap().expect("Access error occured when fetching immediate operand low.");
        (high as u32) << 16 | low as u32
    } else {
        memory.next().unwrap().expect("Access error occured when fetching immediate operand.") as u32
    };

    let naddr = memory.next_addr;
    let eareg = bits(opcode, 0, 2) as u8;
    let eamode = bits(opcode, 3, 5);
    let am = AddressingMode::new(eamode, eareg, Some(size), memory);
    len += memory.next_addr - naddr;

    (Operands::SizeEffectiveAddressImmediate(size, am, imm), len as usize)
}

/// BCHG, BCLR, BSET, BTST
pub fn effective_address_count(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
    let mut len = 0;
    let count = if bits(opcode, 8, 8) != 0 { // dynamic bit number
        bits(opcode, 9, 11) as u8
    } else { // Static bit number
        len += 2;
        memory.next().unwrap().expect("Access error occured when fetching count operand.") as u8
    };

    let naddr = memory.next_addr;
    let eareg = bits(opcode, 0, 2) as u8;
    let eamode = bits(opcode, 3, 5);
    let size = if eamode == 0 { Some(Size::Long) } else { Some(Size::Byte) };
    let am = AddressingMode::new(eamode, eareg, size, memory);
    len += memory.next_addr - naddr;

    (Operands::EffectiveAddressCount(am, count), len as usize)
}

/// JMP, JSR, MOVE (f) SR CCR, NBCD, PEA, TAS
pub fn effective_address(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
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

    let naddr = memory.next_addr;
    let eareg = bits(opcode, 0, 2) as u8;
    let eamode = bits(opcode, 3, 5);
    let am = AddressingMode::new(eamode, eareg, size, memory);
    let len = memory.next_addr - naddr;
    (Operands::EffectiveAddress(am), len as usize)
}

/// CLR, NEG, NEGX, NOT, TST
pub fn size_effective_address(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
    let naddr = memory.next_addr;
    let eareg = bits(opcode, 0, 2) as u8;
    let eamode = bits(opcode, 3, 5);
    let size = Size::from(bits(opcode, 6, 7));
    let am = AddressingMode::new(eamode, eareg, Some(size), memory);
    let len = memory.next_addr - naddr;
    (Operands::SizeEffectiveAddress(size, am), len as usize)
}

/// CHK, DIVS, DIVU, LEA, MULS, MULU
pub fn register_effective_address(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
    let isa = DECODER[opcode as usize];

    let reg = bits(opcode, 9, 11) as u8;
    let size = if isa == Isa::Lea {
        Some(Size::Long)
    } else {
        Some(Size::Word)
    };

    let naddr = memory.next_addr;
    let eareg = bits(opcode, 0, 2) as u8;
    let eamode = bits(opcode, 3, 5);
    let am = AddressingMode::new(eamode, eareg, size, memory);
    let len = memory.next_addr - naddr;
    (Operands::RegisterEffectiveAddress(reg, am), len as usize)
}

/// MOVEP
pub fn register_direction_size_register_displacement(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
    let dreg = bits(opcode, 9, 11) as u8;
    let dir = if bits(opcode, 7, 7) != 0 { Direction::RegisterToMemory } else { Direction::MemoryToRegister };
    let size = if bits(opcode, 6, 6) != 0 { Size::Long } else { Size::Word };
    let areg = bits(opcode, 0, 2) as u8;
    let disp = memory.next().unwrap().expect("Access error occured when fetching displacement operand.") as i16;
    (Operands::RegisterDirectionSizeRegisterDisplacement(dreg, dir, size, areg, disp), 2)
}

/// MOVEA
pub fn size_register_effective_address(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
    let naddr = memory.next_addr;
    let eareg = bits(opcode, 0, 2) as u8;
    let eamode = bits(opcode, 3, 5);
    let areg = bits(opcode, 9, 11) as u8;
    let size = Size::from_move(bits(opcode, 12, 13));
    let am = AddressingMode::new(eamode, eareg, Some(size), memory);
    let len = memory.next_addr - naddr;
    (Operands::SizeRegisterEffectiveAddress(size, areg, am), len as usize)
}

/// MOVE
pub fn size_effective_address_effective_address(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
    let size = Size::from_move(bits(opcode, 12, 13));

    // First read the source operand then the destination.
    let naddr = memory.next_addr;
    let eareg = bits(opcode, 0, 2) as u8;
    let eamode = bits(opcode, 3, 5);
    let src = AddressingMode::new(eamode, eareg, Some(size), memory);
    let lensrc = memory.next_addr - naddr;

    let naddr = memory.next_addr;
    let eamode = bits(opcode, 6, 8);
    let eareg = bits(opcode, 9, 11) as u8;
    let dst = AddressingMode::new(eamode, eareg, Some(size), memory);
    let lendst = memory.next_addr - naddr;

    (Operands::SizeEffectiveAddressEffectiveAddress(size, dst, src), lensrc as usize + lendst as usize)
}

/// EXG
pub fn register_opmode_register(opcode: u16, _: &mut MemoryIter) -> (Operands, usize) {
    let regl = bits(opcode, 9, 11) as u8;
    let opmode = bits(opcode, 3, 7) as u8;
    let regr = bits(opcode, 0, 2) as u8;
    let dir = if opmode == 0b01000 {
        Direction::ExchangeData
    } else if opmode == 0b01001 {
        Direction::ExchangeAddress
    } else {
        Direction::ExchangeDataAddress
    };
    (Operands::RegisterOpmodeRegister(regl, dir, regr), 0)
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
    let disp = memory.next().unwrap().expect("Access error occured when fetching displacement operand.") as i16;
    (Operands::RegisterDisplacement(reg, disp), 2)
}

/// SWAP, UNLK
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
    let list = memory.next().unwrap().expect("Access error occured when fetching list operand.");
    let dir = if bits(opcode, 10, 10) != 0 { Direction::MemoryToRegister } else { Direction::RegisterToMemory };
    let size = Size::from_bit(bits(opcode, 6, 6));

    let naddr = memory.next_addr;
    let eareg = bits(opcode, 0, 2) as u8;
    let eamode = bits(opcode, 3, 5);
    let am = AddressingMode::new(eamode, eareg, Some(size), memory);
    let len = 2 + (memory.next_addr - naddr);

    (Operands::DirectionSizeEffectiveAddressList(dir, size, am, list), len as usize)
}

/// ADDQ, SUBQ
pub fn data_size_effective_address(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
    let data = bits(opcode, 9, 11) as u8;
    let size = Size::from(bits(opcode, 6, 7));

    let naddr = memory.next_addr;
    let eareg = bits(opcode, 0, 2) as u8;
    let eamode = bits(opcode, 3, 5);
    let am = AddressingMode::new(eamode, eareg, Some(size), memory);
    let len = memory.next_addr - naddr;

    (Operands::DataSizeEffectiveAddress(data, size, am), len as usize)
}

/// Scc
pub fn condition_effective_address(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
    let condition = bits(opcode, 8, 11) as u8;

    let naddr = memory.next_addr;
    let eareg = bits(opcode, 0, 2) as u8;
    let eamode = bits(opcode, 3, 5);
    let am = AddressingMode::new(eamode, eareg, Some(Size::Byte), memory);
    let len = memory.next_addr - naddr;

    (Operands::ConditionEffectiveAddress(condition, am), len as usize)
}

/// DBcc
pub fn condition_register_displacement(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
    let disp = memory.next().unwrap().expect("Access error occured when fetching displacement operand.") as i16;
    let condition = bits(opcode, 8, 11) as u8;
    let reg = bits(opcode, 0, 2) as u8;
    (Operands::ConditionRegisterDisplacement(condition, reg, disp), 2)
}

/// BRA, BSR
pub fn displacement(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
    let mut len = 0;
    let mut disp = opcode as i8 as i16;
    if disp == 0 {
        len += 2;
        disp = memory.next().unwrap().expect("Access error occured when fetching displacement operand.") as i16;
    }
    (Operands::Displacement(disp), len)
}

/// Bcc
pub fn condition_displacement(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
    let mut len = 0;
    let mut disp = opcode as i8 as i16;
    if disp == 0 {
        len += 2;
        disp = memory.next().unwrap().expect("Access error occured when fetching displacement operand.") as i16;
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
    let reg = bits(opcode, 9, 11) as u8;
    let dir = if bits(opcode, 8, 8) != 0 { Direction::DstEa } else { Direction::DstReg }; // CMP and EOR ignores it
    let size = Size::from(bits(opcode, 6, 7));

    let naddr = memory.next_addr;
    let eareg = bits(opcode, 0, 2) as u8;
    let eamode = bits(opcode, 3, 5);
    let am = AddressingMode::new(eamode, eareg, Some(size), memory);
    let len = memory.next_addr - naddr;

    (Operands::RegisterDirectionSizeEffectiveAddress(reg, dir, size, am), len as usize)
}

/// ADDA, CMPA, SUBA
pub fn register_size_effective_address(opcode: u16, memory: &mut MemoryIter) -> (Operands, usize) {
    let reg = bits(opcode, 9, 11) as u8;
    let size = Size::from_bit(bits(opcode, 8, 8));

    let naddr = memory.next_addr;
    let eareg = bits(opcode, 0, 2) as u8;
    let eamode = bits(opcode, 3, 5);
    let am = AddressingMode::new(eamode, eareg, Some(size), memory);
    let len = memory.next_addr - naddr;

    (Operands::RegisterSizeEffectiveAddress(reg, size, am), len as usize)
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
    let naddr = memory.next_addr;
    let eareg = bits(opcode, 0, 2) as u8;
    let eamode = bits(opcode, 3, 5);
    let dir = if bits(opcode, 8, 8) != 0 { Direction::Left } else { Direction::Right };
    let am = AddressingMode::new(eamode, eareg, Some(Size::Byte), memory);
    let len = memory.next_addr - naddr;
    (Operands::DirectionEffectiveAddress(dir, am), len as usize)
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
