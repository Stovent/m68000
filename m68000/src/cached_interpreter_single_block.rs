// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Cached interpreter with a single block that stores individually-cached instructions.

use std::hint::cold_path;

use crate::{CpuDetails, MemoryAccess, M68000};
use crate::instruction::Instruction;
use crate::exception::ADDRESS_ERROR;
use crate::utils::IsEven;

/// Cached interpreter with a single block that stores individually-cached instructions.
pub struct CachedInterpreterSingleBlock {
    /// Indexed with the program counter >> 1.
    ///
    /// If none, it means it has not been cached yet.
    instruction_cache: Box<[Option<Instruction>]>,
    /// The range of the chachable instructions.
    cache_range: core::range::Range<u32>,
}

impl CachedInterpreterSingleBlock {
    /// Creates a new cached interpreter with the given range as cachable.
    pub fn new(begin: u32, end: u32) -> Self {
        Self {
            instruction_cache: vec![None; ((end - begin) >> 1) as usize].into_boxed_slice(),
            cache_range: (begin..end).into(), // TODO: remove .into() when possible.
        }
    }

    pub fn cached_interpreter<M: MemoryAccess + ?Sized, CPU: CpuDetails>(&mut self, m68000: &mut M68000<CPU>, memory: &mut M) -> usize {
        let (cycles, exception) = self.cached_interpreter_exception(m68000, memory);
        if let Some(e) = exception {
            cold_path();
            m68000.exception(e.into());
        }
        cycles
    }

    pub fn cached_interpreter_exception<M: MemoryAccess + ?Sized, CPU: CpuDetails>(&mut self, m68000: &mut M68000<CPU>, memory: &mut M) -> (usize, Option<u8>) {
        if m68000.stop {
            return (0, None);
        }

        let mut cycle_count = 0;

        if !m68000.exceptions.is_empty() {
            cycle_count += m68000.process_pending_exceptions(memory);
        }

        if !m68000.regs.pc.0.is_even() {
            return (0, Some(ADDRESS_ERROR))
        }
        // If we are not in the cachable range, the wrapping_sub will underflow to an impossible address, or still be out of range of the buffer.
        let cache_index = (m68000.regs.pc.0.wrapping_sub(self.cache_range.start) >> 1) as usize; // Odd addresses are impossible.

        let (cycles, exception) = if let Some(element) = self.instruction_cache.get(cache_index) { // Cachable range.
            let instruction = match element {
                Some(instruction) => {
                    m68000.regs.pc += instruction.length as u32;
                    instruction
                },
                None => {
                    let instruction = match m68000.get_next_instruction(memory) {
                        Ok(i) => i,
                        Err(e) => return (cycle_count, Some(e)),
                    };
                    self.instruction_cache[cache_index] = Some(instruction);
                    // SAFETY: we just inserted it, so guaranteed to exist.
                    unsafe {
                        &self.instruction_cache.get_unchecked(cache_index).unwrap_unchecked()
                    }
                },
            };

            m68000.execute_instruction(memory, instruction)
        } else { // Uncachable range, use regular interpreter.
            m68000.execute_interpreter_exception(memory)
        };

        cycle_count += cycles;
        (cycle_count, exception)
    }
}
