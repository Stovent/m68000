//! Exception processing.

use crate::{M68000, MemoryAccess};
use crate::execution_times::vector_execution_time;
use crate::execution_times as EXEC;
use crate::interpreter::InterpreterResult;

use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// Exception vectors of the 68000.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Vector {
    ResetSspPc = 0,
    /// Bus error.
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
    /// The spurious interrupt vector is taken when there is a bus error indication during interrupt processing.
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

// TODO: group of the remaining vectors (Line A/F emulation, Format error, uninitialized interrupt vector).
pub(crate) fn get_vector_group(vector: u8) -> u8 {
    match vector {
        0..=3 => 0,
        4 => 1,
        5..=7 => 2,
        8..=9 => 1,
        24..=31 => 1,
        32..=47 => 2,
        64..=255 => 1,
        _ => panic!("[get_vector_group] Unkown vector {}.", vector),
    }
}

// TODO: priority of the remaining vectors (Line A/F emulation, Format error, uninitialized interrupt vector).
pub(crate) fn get_vector_priority(vector: u8, group: u8) -> u8 {
    match group {
        0 => match vector {
            0 => 0,
            3 => 1,
            2 => 2,
            _ => panic!("[get_vector_priority] Unkown vector {} for group 0.", vector),
        },
        1 => match vector {
            9 => 0,
            24..=31 => 1,
            64..=255 => 1,
            4 => 2,
            8 => 3,
            _ => panic!("[get_vector_priority] Unkown vector {} for group 1.", vector),
        },
        2 => 0, // Only one instruction can be executed at a time, so no priority necessary.
        _ => panic!("[get_vector_priority] Unkown group {}.", group),
    }
}

/// Lowest number = higher priority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Exception {
    pub vector: u8,
    group: u8,
    /// Priority inside the group.
    priority: u8,
}

impl Exception {
    pub(crate) fn new(vector: u8) -> Self {
        let group = get_vector_group(vector);
        let priority = get_vector_priority(vector, group);
        Self { vector, group, priority }
    }
}

impl PartialOrd for Exception {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Exception {
    /// Max-heap regarding the group and priority number.
    fn cmp(&self, other: &Self) -> Ordering {
        if self.group < other.group {
            Ordering::Less
        } else if self.group > other.group {
            Ordering::Greater
        } else {
            if self.priority < other.priority {
                Ordering::Less
            } else if self.priority > other.priority {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        }
    }
}

impl M68000 {
    /// Requests the CPU to process the given exception.
    pub fn exception(&mut self, vector: u8) {
        let exception = Exception::new(vector);
        self.exceptions.push(exception);
    }

    /// Attempts to process all the pending exceptions
    pub(crate) fn process_pending_exceptions(&mut self, memory: &mut impl MemoryAccess) -> usize {
        // The reset vector clears all the pending interrupts.
        let mut has_reset = false;
        for ex in self.exceptions.iter() {
            if *ex == Exception::new(Vector::ResetSspPc as u8) {
                has_reset = true;
            }
        }
        if has_reset {
            self.exceptions.clear();
            return self.process_exception(memory, Vector::ResetSspPc as u8)
                .unwrap_or_else(|_| panic!("An Access Error occured during reset vector."));
        }

        let mut total = 0;

        // Save the unprocessed interrupts.
        let mut masked_interrupts = BinaryHeap::new();

        // Pops from the lowest priority to highest, so that when all exceptions has been processed,
        // the one with the highest priority will be the one treated first.
        while let Some(exception) = self.exceptions.pop() {
            if exception.vector >= Vector::Level1InterruptAutovector as u8 && exception.vector <= Vector::Level7InterruptAutovector as u8 {
                // If the interrupt is lower or equal to the interrupt mask, then it is not processed.
                let level = exception.vector - (Vector::Level1InterruptAutovector as u8 - 1);
                if level <= self.regs.sr.interrupt_mask {
                    masked_interrupts.push(exception);
                    continue;
                }
            }

            total += match self.process_exception(memory, exception.vector) {
                Ok(cycles) => cycles,
                Err(e) => {
                    if exception.vector == e && e == Vector::AccessError as u8 {
                        panic!("An exception occured during exception processing: {} (at {:#X})", e, self.regs.pc);
                    } else {
                        self.exception(e);
                        0
                    }
                },
            };
        }

        self.exceptions.append(&mut masked_interrupts);

        total
    }

    /// Effectively processes an exception.
    ///
    /// The execution time added is the one when the exception occured (CHK and TRAPV).
    /// If exception didn't occured, the interpreter function returns the other number of cycles.
    ///
    /// CHK and Zero divide have an effective address field. If the exception occurs, the interpreter returns the effective
    /// address calculation time, and this method returns the exception processing time.
    pub(super) fn process_exception(&mut self, memory: &mut impl MemoryAccess, vector: u8) -> InterpreterResult {
        let sr = self.regs.sr.into();
        self.regs.sr.s = true;

        if vector == 0 {
            self.regs.ssp = memory.get_long(0)?;
            self.regs.pc  = memory.get_long(4)?;
            self.stop = false;
            self.exceptions.clear();
            return Ok(EXEC::VECTOR_RESET);
        }

        if vector == Vector::Trace as u8 ||
           vector >= Vector::SpuriousInterrupt as u8 && vector <= Vector::Level7InterruptAutovector as u8 {
            self.stop = false;
        }

        #[cfg(feature = "cpu-mc68000")] {
            self.push_long(memory, self.regs.pc)?;
            self.push_word(memory, sr)?;

            if vector == 2 || vector == 3 { // TODO: Long format
                self.push_word(memory, self.current_opcode)?;
                self.push_long(memory, 0)?; // Access address
                self.push_word(memory, 0)?; // function code
                // MC68000UM 6.3.9.1: It is the responsibility of the error handler routine
                // to clean up the stack and determine where to continue execution.
            }
        }

        #[cfg(feature = "cpu-scc68070")] {
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
                self.push_word(memory, 0xF000 | vector as u16 * 4)?;
            } else { // Short format
                self.push_word(memory, vector as u16 * 4)?;
            }

            self.push_long(memory, self.regs.pc)?;
            self.push_word(memory, sr)?;
        }

        self.regs.pc = memory.get_long(vector as u32 * 4)?;

        Ok(vector_execution_time(vector))
    }
}
