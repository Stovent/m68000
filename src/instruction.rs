// use super::addressing_modes::AddressingMode;
use super::{M68000, MemoryAccess};
use super::isa::ISA;
use super::operand::*;
use super::utils::SliceAs;

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
    /// No direction in the instruction.
    None,
}

pub(super) struct Instruction {
    pub isa: ISA,
    pub opcode: u16,
    /// The address of the instruction.
    pub pc: u32,
    pub size: Option<Size>,

    /// Source operand, if any.
    pub src: Option<Operand>,
    /// Destination operand, if any.
    pub dst: Option<Operand>,
    /// Direction of the operation.
    pub direction: Option<Direction>,
    /// Register number in the case of a fixed addressing mode (LINK, UNLK, MOCE USP, Shift, Rotate, etc.).
    pub reg: Option<u16>,
}

impl Instruction {
    /// Creates a new instruction, returning it and the number of bytes read from the slice.
    ///
    /// decode from what is at d[0] and d[1].
    pub(super) fn new<M: MemoryAccess>(pc: u32, mut d: &[u8]) -> (Self, u32) {
        let opcode = d.get_next_word();
        let mut width = 2;
        let isa = M68000::<M>::DECODER[opcode as usize];
        (Self {
            isa,
            opcode,
            pc,
            size: None,

            src: None,
            dst: None,
            direction: None,
            reg: None,
        }, width)
    }
}
