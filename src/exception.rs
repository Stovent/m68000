#![allow(dead_code)]

use super::{M68000, MemoryAccess};

/// The exception vectors of the 68000.
pub enum Vector {
    ResetSsp = 0,
    ResetPc,
    BusError, // Access error
    AddressError,
    IllegalInstruction,
    ZeroDivide,
    ChkInstruction,
    TRAPV,
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

impl<M: MemoryAccess> M68000<M> {
    pub(super) fn exception(&mut self, vector: u32) -> usize {
        let sr = self.sr.into();
        self.sr.s = true;

        println!("exception {}", vector);

        if vector == 0 {
            self.ssp = self.memory.get_long(0);
            return 1;
        }
        if vector == 1 {
            self.pc = self.memory.get_long(4);
            return 1;
        }

        // if self.config.stack == StackFrame::Stack68000 {
        //     if vector == 2 || vector == 3 { // Long format
        //         self.push_word(self.current_opcode);
        //         self.push_long(0); // Access address
        //         self.push_word(0); // function code
        //     }

        //     self.push_long(self.pc);
        //     self.push_word(sr);
        // } else { // if self.config.stack == StackFrame::Stack68070
            if vector == 2 || vector == 3 { // TODO: Long format
                self.push_word(0);
                self.push_word(0);
                self.push_word(0);
                self.push_long(0);
                self.push_long(0);
                self.push_long(0);
                self.push_word(0);
                self.push_word(0);
                self.push_word(0);
                self.push_word(0);
                self.push_word(0xF000 | (vector * 4) as u16);
            } else { // Short format
                self.push_word(vector as u16 * 4);
            }

            self.push_long(self.pc);
            self.push_word(sr);
        // }

        self.pc = self.memory.get_long(vector * 4);

        1
    }
}
