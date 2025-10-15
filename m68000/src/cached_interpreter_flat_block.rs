//! Cached interpreter with a configurable flat array for block caching.

use crate::{CpuDetails, MemoryAccess, M68000};
use crate::instruction::Instruction;

type SUBCACHE = Option<Vec<Instruction>>;
const EMPTY_SUBCACHE: SUBCACHE = None;

/// Cached interpreter with a configurable flat array for block caching.
///
/// Indexes in the cache are computed with `(addr as usize) >> self.address_shift & self.address_mask`.
pub struct CachedInterpreterFlatBlock<CPU: CpuDetails> {
    pub m68000: M68000<CPU>,
    /// Indexed with the program counter >> 1.
    instruction_cache: Box<[SUBCACHE]>,
    /// The number of bits to shift the address by when indexing the cache.
    address_shift: usize,
    /// The mask to apply to the address after shifting.
    address_mask: usize,
}

impl<CPU: CpuDetails> CachedInterpreterFlatBlock<CPU> {
    /// Creates a new cached interpreter with the given cache size (number of addresses)
    ///
    /// Note that addresses are always even, so feel free to add one to your shift count if you want.
    /// You must adapt your mask and buffer size if you do so.
    ///
    /// For example if your PC is only in the range `[0x1000, 0x4000)`:
    /// - size can be 0x3000 or 0xC00 (0x3000 >> 1).
    /// - shift can be 12 or 13.
    /// - mask will be 3.
    ///
    /// Physical bus on a 68000 is 24 bits, minus 1 for even alignment, so you can use the following values:
    /// - size = 1 << 23;
    /// - shift = 23;
    /// - mask = 0x7F_FFFF;
    pub fn new(m68000: M68000<CPU>, size: usize, shift: usize, mask: usize) -> Self {
        Self {
            m68000,
            instruction_cache: vec![EMPTY_SUBCACHE; size].into_boxed_slice(),
            address_shift: shift,
            address_mask: mask,
        }
    }

    /// Converts the given address to its instruction cache index.
    #[inline(always)]
    const fn get_cache_index(&self, addr: u32) -> usize {
        (addr as usize) >> self.address_shift & self.address_mask
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

        let cache_index = self.get_cache_index(self.m68000.regs.pc.0);

        if cache_index >= self.instruction_cache.len() {
            return self.m68000.interpreter_exception(memory);
        }

        if let Some(block) = &self.instruction_cache[cache_index] {
            for instruction in block {
                if !self.m68000.exceptions.is_empty() { // External exception.
                    return (cycle_count, None);
                }

                self.m68000.regs.pc += instruction.length as u32;
                let (cycles, exception) = self.m68000.execute_instruction(memory, instruction);
                cycle_count += cycles;
                if exception.is_some() {
                    return (cycle_count, exception);
                }
            }
            return (cycle_count, None);
        }

        // No block, make one.

        // Compromise between wasted RAM and fear of reallocation. 16 doesn't seem too bad.
        let mut block = Vec::with_capacity(16);

        loop {
            if !self.m68000.exceptions.is_empty() { // External exception.
                // Don't store cache if an exception occured.
                return (cycle_count, None);
            }

            let instruction = match self.m68000.get_next_instruction(memory) {
                Ok(i) => i,
                Err(e) => return (cycle_count, Some(e)),
            };

            let (cycles, exception) = self.m68000.execute_instruction(memory, &instruction);
            cycle_count += cycles;
            if exception.is_some() {
                return (cycle_count, exception);
            }

            if exception.is_some() {
                // Don't store cache if an exception occured.
                return (cycle_count, exception);
            }

            block.push(instruction);

            if instruction.ends_block() {
                break;
            }
        }

        self.instruction_cache[cache_index] = Some(block);

        (cycle_count, None)
    }
}
