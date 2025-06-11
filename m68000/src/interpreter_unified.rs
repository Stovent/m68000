// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{CpuDetails, M68000, MemoryAccess};
use crate::exception::{Exception, Vector};
use crate::interpreter_disassembler::Execute;
use crate::isa::Isa;

impl<CPU: CpuDetails> M68000<CPU> {
    /// Runs the interpreter loop once and disassembles the next instruction if any.
    ///
    /// Returns the address of the instruction, its disassembled string and the cycle count necessary to execute it.
    /// The disassembled string is empty if no instruction has been executed.
    ///
    /// See [Self::interpreter] for the potential caveat.
    pub fn interpreter_unified<const DIS: bool, M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> (usize, Option<(u32, String)>) {
        let (cycles, disassembly, exception) = self.interpreter_exception_unified::<DIS, M>(memory);
        if let Some(e) = exception {
            self.exception(Exception::from(e));
        }
        (cycles, disassembly)
    }

    /// Runs the interpreter loop once and disassembles the next instruction if any.
    ///
    /// Returns the address of the instruction that has been executed, its disassembled string, the cycle count
    /// necessary to execute it, and the vector of the exception that occured during the execution if any.
    /// The disassembled string is empty if no instruction has been executed.
    ///
    /// To process the returned exception, call [M68000::exception].
    ///
    /// See [Self::interpreter_exception] for the potential caveat.
    pub fn interpreter_exception_unified<const DIS: bool, M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> (usize, Option<(u32, String)>, Option<u8>) {
        if self.stop {
            return (0, None, None);
        }

        let mut cycle_count = 0;

        if !self.exceptions.is_empty() {
            cycle_count += self.process_pending_exceptions(memory);
        }

        let pc = self.regs.pc.0;
        let instruction = match self.get_next_instruction(memory) {
            Ok(i) => i,
            Err(e) => return (cycle_count, None, Some(e)),
        };

        self.current_opcode = instruction.opcode;
        let isa = Isa::from(instruction.opcode);

        let trace = self.regs.sr.t;
        let exception = match Execute::<CPU, M>::EXECUTE[isa as usize](self, memory, &instruction) {
            Ok(cycles) => {
                cycle_count += cycles;
                if trace && !isa.is_privileged() {
                    Some(Vector::Trace as u8)
                } else {
                    None
                }
            },
            Err(e) => Some(e),
        };

        let dis = if DIS {
            Some((pc, instruction.disassemble()))
        } else {
            None
        };

        (cycle_count, dis, exception)
    }
}
