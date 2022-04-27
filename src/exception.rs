//! Exception processing.

use crate::{M68000, MemoryAccess};
use crate::interpreter::InterpreterResult;

/// Exception vectors of the 68000.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Vector {
    ResetSspPc = 0,
    AccessError = 2,
    AddressError,
    IllegalInstruction,
    ZeroDivide,
    ChkInstruction,
    TrapVInstruction,
    PrivilegeViolation,
    Trace,
    Line1010Emulator,
    Line1111Emulator,
    FormatError = 14,
    UninitializedInterrupt,
    SpuriousInterrupt = 24,
    Level1InterruptAutovector,
    Level2InterruptAutovector,
    Level3InterruptAutovector,
    Level4InterruptAutovector,
    Level5InterruptAutovector,
    Level6InterruptAutovector,
    Level7InterruptAutovector,
    Trap0Instruction,
    Trap1Instruction,
    Trap2Instruction,
    Trap3Instruction,
    Trap4Instruction,
    Trap5Instruction,
    Trap6Instruction,
    Trap7Instruction,
    Trap8Instruction,
    Trap9Instruction,
    Trap10Instruction,
    Trap11Instruction,
    Trap12Instruction,
    Trap13Instruction,
    Trap14Instruction,
    Trap15Instruction,
}

impl M68000 {
    /// Requests the CPU to process the given exception.
    pub fn exception(&mut self, vector: u8) {
        self.exceptions.push_back(vector);
    }

    /// Effectively processes an exception.
    pub(super) fn process_exception(&mut self, memory: &mut impl MemoryAccess, vector: u8) -> InterpreterResult {
        let sr = self.sr.into();
        self.sr.s = true;

        if vector == 0 {
            self.ssp = memory.get_long(0)?;
            self.pc  = memory.get_long(4)?;
            self.stop = false;
            return Ok(1);
        }

        if vector == Vector::Trace as u8 ||
           vector >= Vector::SpuriousInterrupt as u8 && vector <= Vector::Level7InterruptAutovector as u8 {
            self.stop = false;
        }

        if self.stack_format.is_68000() {
            self.push_long(memory, self.pc)?;
            self.push_word(memory, sr)?;

            if vector == 2 || vector == 3 { // Long format
                self.push_word(memory, self.current_opcode)?;
                self.push_long(memory, 0)?; // Access address
                self.push_word(memory, 0)?; // function code
            }
        } else { // if self.stack_format.is_68010() || self.stack_format.is_68070() {
            if vector == 2 || vector == 3 { // TODO: Long format
                self.push_word(memory, 0)?;
                self.push_word(memory, 0)?;
                self.push_word(memory, 0)?;
                self.push_long(memory, 0)?;
                self.push_long(memory, 0)?;
                self.push_long(memory, 0)?;
                self.push_word(memory, 0)?;
                self.push_word(memory, 0)?;
                self.push_word(memory, 0)?;
                self.push_word(memory, 0)?;
                if self.stack_format.is_68010() {
                    self.push_word(memory, 0x8000 | vector as u16 * 4)?;
                } else { // if self.stack_format.is_68070() {
                    self.push_word(memory, 0xF000 | vector as u16 * 4)?;
                }
            } else { // Short format
                self.push_word(memory, vector as u16 * 4)?;
            }

            self.push_long(memory, self.pc)?;
            self.push_word(memory, sr)?;
        }

        self.pc = memory.get_long(vector as u32 * 4)?;

        Ok(1)
    }
}
