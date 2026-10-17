// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Cached interpreter with a trie for block caching.
//!
//! # Important
//!
//! This cached interpreter caches the whole address space and is not invalidating.
//! If this is an issue, use [crate::cached_interpreter_flat_block] or [crate::cached_interpreter_single_block].

use crate::{CpuDetails, MemoryAccess, M68000};
use crate::instruction::Instruction;

type Block = Option<Vec<Instruction>>;
const EMPTY_BLOCK: Block = None;
type Subcache = Option<Vec<Block>>;
const EMPTY_SUBCACHE: Subcache = None;

const SUBCACHE_BITS: usize = 12;
const SUBCACHE_SIZE: usize = 1 << SUBCACHE_BITS;
const SUBCACHE_MASK: usize = SUBCACHE_SIZE - 1;
/// Physical bus on a 68000 is 24 bits, but it doesn't hurt to support the whole 32 bits, minus 1 for even alignment.
const CACHE_BITS: usize = 31 - SUBCACHE_BITS;
const CACHE_SIZE: usize = 1 << CACHE_BITS;
const CACHE_MASK: usize = CACHE_SIZE - 1;

/// Converts the given address to its instruction cache index.
#[inline(always)]
const fn cache_index(addr: u32) -> usize {
    (addr as usize) >> (1 + SUBCACHE_BITS) & CACHE_MASK // 1 for even address.
}

/// Converts the given address to its index in the subcache.
#[inline(always)]
const fn subcache_index(addr: u32) -> usize {
    (addr as usize) >> 1 & SUBCACHE_MASK
}

/// Cached interpreter with a trie look-up table.
///
/// https://web.archive.org/web/20210301060701/https://ps1.asuramaru.com/emulator-development/cached-interpreters
pub struct CachedInterpreterTrieBlock {
    /// Each element is a subrow for each PC value.
    instruction_cache: Box<[Subcache]>,
}

impl Default for CachedInterpreterTrieBlock {
    fn default() -> Self {
        Self::new()
    }
}

impl CachedInterpreterTrieBlock {
    pub fn new() -> Self {
        Self {
            instruction_cache: vec![EMPTY_SUBCACHE; CACHE_SIZE].into_boxed_slice(),
        }
    }

    pub fn interpreter<M: MemoryAccess + ?Sized, CPU: CpuDetails>(&mut self, m68000: &mut M68000<CPU>, memory: &mut M) -> usize {
        let (cycles, exception) = self.interpreter_exception(m68000, memory);
        if let Some(e) = exception {
            m68000.exception(e.into());
        }
        cycles
    }

    pub fn interpreter_exception<M: MemoryAccess + ?Sized, CPU: CpuDetails>(&mut self, m68000: &mut M68000<CPU>, memory: &mut M) -> (usize, Option<u8>) {
        if m68000.stop {
            return (0, None);
        }

        let mut cycle_count = 0;

        if !m68000.exceptions.is_empty() {
            cycle_count += m68000.process_pending_exceptions(memory);
        }

        let pc = m68000.regs.pc.0;
        let cache_index = cache_index(pc);

        // If the cache does not cover the whole address space, uncomment this.
//         if cache_index >= self.instruction_cache.len() {
//             return m68000.interpreter_exception(memory);
//         }

        let subcache = if let Some(subcache) = self.instruction_cache[cache_index].as_mut() {
            subcache
        } else {
            self.instruction_cache[cache_index] = Some(vec![EMPTY_BLOCK; SUBCACHE_SIZE]);
            // SAFETY: we just inserted it so it must be valid.
            unsafe {
                self.instruction_cache[cache_index].as_mut().unwrap_unchecked()
            }
        };

        let subcache_index = subcache_index(pc);
        if let Some(block) = &subcache[subcache_index] {
            for instruction in block {
                if !m68000.exceptions.is_empty() { // External exception.
                    return (cycle_count, None);
                }

                m68000.regs.pc += instruction.length as u32;
                let (cycles, exception) = m68000.execute_instruction(memory, instruction);
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
            if !m68000.exceptions.is_empty() { // External exception.
                // Don't store cache if an exception occured.
                return (cycle_count, None);
            }

            let instruction = match m68000.get_next_instruction(memory) {
                Ok(i) => i,
                Err(e) => return (cycle_count, Some(e)),
            };

            let (cycles, exception) = m68000.execute_instruction(memory, &instruction);
            cycle_count += cycles;

            if exception.is_some() {
                // Don't store cache if an exception occured.
                return (cycle_count, exception);
            }

            block.push(instruction);

            if instruction.ends_block() {
                break;
            }
        }

        subcache[subcache_index] = Some(block);

        (cycle_count, None)
    }
}
