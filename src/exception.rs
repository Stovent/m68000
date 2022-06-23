//! Exception processing.

use crate::{M68000, MemoryAccess};
use crate::execution_times::vector_execution_time;
use crate::interpreter::InterpreterResult;

use std::cmp::Ordering;
use std::collections::BTreeSet;

/// Exception vectors of the 68000.
///
/// You can directly cast the enum to u8 to get the vector number.
/// ```
/// use m68000::exception::Vector;
/// assert_eq!(Vector::AccessError as u8, 2);
/// ```
///
/// The `FormatError` and `OnChipInterrupt` vectors are only used by the SCC68070.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[repr(C)]
pub enum Vector {
    /// Bus error. Sent when the accessed address is not in the memory range of the system.
    AccessError = 2,
    AddressError,
    IllegalInstruction,
    ZeroDivide,
    ChkInstruction,
    TrapVInstruction,
    PrivilegeViolation,
    Trace,
    FormatError = 14,
    UninitializedInterrupt,
    /// The spurious interrupt vector is taken when there is a bus error indication during interrupt processing.
    SpuriousInterrupt = 24,
    Level1Interrupt,
    Level2Interrupt,
    Level3Interrupt,
    Level4Interrupt,
    Level5Interrupt,
    Level6Interrupt,
    Level7Interrupt,
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
    Level1OnChipInterrupt = 57,
    Level2OnChipInterrupt,
    Level3OnChipInterrupt,
    Level4OnChipInterrupt,
    Level5OnChipInterrupt,
    Level6OnChipInterrupt,
    Level7OnChipInterrupt,
    UserInterrupt,
}

const fn get_vector_priority(vector: u8) -> u8 {
    match vector {
        3 => 0, // Address error.
        2 => 1, // Access Error.
        9 => 2, // Trace.
        24..=31 => 3, // Interrupt.
        64..=255 => 3, // User Interrupt.
        4 => 4, // Illegal.
        8 => 5, // Privilege.
        _ => u8::MAX, // Other vectors.
    }
}

/// M68000 exception, with a vector number and a priority.
///
/// This struct implements `From<u8>` and `From<Vector>`, to create an exception from the raw vector number or from the nammed vector, respectively.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Exception {
    pub vector: u8,
    /// Lower means higher priority.
    priority: u8,
}

impl From<u8> for Exception {
    fn from(vector: u8) -> Self {
        let priority = get_vector_priority(vector);
        Self { vector, priority }
    }
}

impl From<Vector> for Exception {
    fn from(vector: Vector) -> Self {
        Self::from(vector as u8)
    }
}

impl PartialOrd for Exception {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Exception {
    /// For BTreeSet, compare by actual priority and not by the value itself, so higher number means less priority.
    fn cmp(&self, other: &Self) -> Ordering {
        if self.priority < other.priority {
            Ordering::Greater
        } else if self.priority > other.priority {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    }
}

impl M68000 {
    /// Requests the CPU to process the given exception.
    pub fn exception(&mut self, ex: Exception) {
        if ex.vector == Vector::Trace as u8 ||
           ex.vector >= Vector::Level1Interrupt as u8 && ex.vector <= Vector::Level7Interrupt as u8 ||
           ex.vector >= Vector::Level1OnChipInterrupt as u8 && ex.vector <= Vector::Level7OnChipInterrupt as u8 {
            self.stop = false;
        }

        self.exceptions.insert(ex);
    }

    /// Attempts to process all the pending exceptions
    pub(super) fn process_pending_exceptions(&mut self, memory: &mut impl MemoryAccess) -> usize {
        // Extract the exceptions to process and keep the masked interrupts.
        let exceptions: BTreeSet<_> = self.exceptions.drain_filter(|ex| {
            if ex.vector >= Vector::Level1Interrupt as u8 && ex.vector <= Vector::Level7Interrupt as u8 ||
               ex.vector >= Vector::Level1OnChipInterrupt as u8 && ex.vector <= Vector::Level7OnChipInterrupt as u8 {
                // If the interrupt is lower or equal to the interrupt mask, then it is not processed.
                let level = ex.vector & 0x7;
                if level <= self.regs.sr.interrupt_mask {
                    return false;
                }
            }

            return true;
        }).collect();

        let mut total = 0;

        // Iterates from the lowest priority to highest, so that when all exceptions have been processed,
        // the one with the highest priority will be the one treated first.
        let mut iter = exceptions.iter();
        while let Some(exception) = iter.next() {
            total += match self.process_exception(memory, exception.vector) {
                Ok(cycles) => cycles,
                Err(e) => {
                    if e == Vector::AccessError as u8 {
                        if exception.vector == Vector::AccessError as u8 {
                            panic!("An access error occured during access error processing (at {:#X})", self.regs.pc);
                        }

                        if exception.vector >= Vector::Level1Interrupt as u8 && exception.vector <= Vector::Level7Interrupt as u8 ||
                           exception.vector >= Vector::Level1OnChipInterrupt as u8 && exception.vector <= Vector::Level7OnChipInterrupt as u8 {
                            self.exception(Exception::from(Vector::SpuriousInterrupt));
                        } else {
                            self.exception(Exception::from(e));
                        }
                    } else {
                        self.exception(Exception::from(e));
                    }

                    0
                },
            };
        }

        total
    }

    /// Effectively processes an exception.
    ///
    /// The execution time added is the one when the exception occured (CHK and TRAPV).
    /// If exception didn't occured, the interpreter function returns the other number of cycles.
    ///
    /// CHK and Zero divide have an effective address field. If the exception occurs, the interpreter returns the effective
    /// address calculation time, and this method returns the exception processing time.
    ///
    /// TODO: the timing may not be perfect here. If two words can be pushed but not the third, then the time taken to push
    /// the first two words is not counted.
    fn process_exception(&mut self, memory: &mut impl MemoryAccess, vector: u8) -> InterpreterResult {
        let sr = self.regs.sr.into();
        self.regs.sr.t = false;
        self.regs.sr.s = true;

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
