//! Cached interpreter with a single block to store all the cached instructions.

use std::hint::cold_path;

use crate::{CpuDetails, MemoryAccess, M68000};
use crate::instruction::Instruction;
use crate::exception::ADDRESS_ERROR;
use crate::utils::IsEven;

/// Cached interpreter with a single block to store all the cached instructions (non-invalidating).
pub struct CachedInterpreterSingleBlock<CPU: CpuDetails> {
    /// TODO: make a version without ownership of the CPU.
    pub m68000: M68000<CPU>,
    /// Indexed with the program counter >> 1.
    ///
    /// If none, it means the instruction cannot be cached.
    /// If Some but an unknown instruction, it means it has not been cached yet.
    instruction_cache: Box<[Option<Instruction>]>,
    /// The range of the chachable instructions.
    cache_range: core::range::Range<u32>,
}

impl<CPU: CpuDetails> CachedInterpreterSingleBlock<CPU> {
    /// Creates a new cached interpreter with the given range as cachable.
    ///
    /// TODO: make an invalidating version and a version with a fixed non-invalidating address range.
    pub fn new(m68000: M68000<CPU>, begin: u32, end: u32) -> Self {
        Self {
            m68000,
            instruction_cache: vec![None; ((end - begin) >> 1) as usize].into_boxed_slice(),
            cache_range: (begin..end).into(), // TODO: remove .into() when possible.
        }
    }

    pub fn cached_interpreter<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> usize {
        let (cycles, exception) = self.cached_interpreter_exception(memory);
        if let Some(e) = exception {
            self.m68000.exception(e.into());
        }
        cycles
    }

    pub fn cached_interpreter_exception<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> (usize, Option<u8>) {
        if self.m68000.stop {
            return (0, None);
        }

        let mut cycle_count = 0;

        if !self.m68000.exceptions.is_empty() {
            cycle_count += self.m68000.process_pending_exceptions(memory);
        }

        if !self.m68000.regs.pc.0.is_even() {
            return (0, Some(ADDRESS_ERROR))
        }
        // If we are not in the cachable range, the wrapping_sub will underflow to an impossible address, or still be out of range of the buffer.
        let cache_index = (self.m68000.regs.pc.0.wrapping_sub(self.cache_range.start) >> 1) as usize; // Odd addresses are impossible.

        let (cycles, exception) = if let Some(element) = self.instruction_cache.get(cache_index) { // Cachable range.
            let instruction = match element {
                Some(instruction) => {
                    self.m68000.regs.pc += instruction.length as u32;
                    instruction
                },
                None => {
                    cold_path();
                    let instruction = match self.m68000.get_next_instruction(memory) {
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

            self.m68000.execute_instruction(memory, instruction)
        } else { // Uncachable range, use regular interpreter.
            self.m68000.execute_interpreter_exception(memory)
        };

        cycle_count += cycles;
        (cycle_count, exception)
    }
}
