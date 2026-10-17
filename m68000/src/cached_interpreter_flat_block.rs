// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Cached interpreter which stores instruction blocks in a flat array.

use crate::{CpuDetails, MemoryAccess, M68000};
use crate::instruction::Instruction;

type Block = Option<Vec<Instruction>>;
const EMPTY_BLOCK: Block = None;

/// Cached interpreter which stores instruction blocks in a flat array.
pub struct CachedInterpreterFlatBlock {
    /// Indexed with program counter >> 1.
    instruction_cache: Box<[Block]>,
    /// The range of the chachable instructions.
    cache_range: core::range::Range<u32>,
}

impl CachedInterpreterFlatBlock {
    /// Creates a new cached interpreter with the given memory range as cachable.
    pub fn new(begin: u32, end: u32) -> Self {
        let size = ((end - begin) >> 1) as usize;
        Self {
            instruction_cache: vec![EMPTY_BLOCK; size].into_boxed_slice(),
            cache_range: (begin..end).into(), // TODO: remove .into() when possible.
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

        if !self.cache_range.contains(&m68000.regs.pc.0) {
            return m68000.interpreter_exception(memory);
        }

        // Guaranteed to be in range, wrapping_sub to avoid checks.
        let cache_index = (m68000.regs.pc.0.wrapping_sub(self.cache_range.start) >> 1) as usize;
        if let Some(block) = &self.instruction_cache[cache_index] {
            for instruction in block {
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

        self.instruction_cache[cache_index] = Some(block);

        (cycle_count, None)
    }

}

/* Keep as artefact in case I want to try cache invalidation again.

impl CachedInterpreterFlatBlock {
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
    pub fn new(size: usize, shift: usize, mask: usize) -> Self {
        Self {
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

    pub fn cached_interpreter<M: MemoryAccess + ?Sized, CPU: CpuDetails>(&mut self, m68000: &mut M68000<CPU>, memory: &mut M) -> usize {
        let (cycles, exception) = self.cached_interpreter_exception(m68000, memory);
        if let Some(e) = exception {
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

        let cache_index = self.get_cache_index(m68000.regs.pc.0);

        if cache_index >= self.instruction_cache.len() {
            return m68000.interpreter_exception(memory);
        }

        let mut cached_memory = CachedMemoryAccess {
            memory,
            write_type: MemoryWriteType::None,
        };
        let mut invalidated_addresses = Vec::with_capacity(16); // Same as default cache length.

        if let Some(block) = &self.instruction_cache[cache_index] {
            for instruction in block {
                if !m68000.exceptions.is_empty() { // External exception.
                    return (cycle_count, None);
                }

                m68000.regs.pc += instruction.length as u32;
                let (cycles, exception) = m68000.execute_instruction(&mut cached_memory, instruction);
                cycle_count += cycles;

                // Invalidate cache.
                match cached_memory.write_type {
                    MemoryWriteType::ByteWord(addr) => {
                        invalidated_addresses.push(addr);
                    },
                    MemoryWriteType::Long(addr) => {
                        invalidated_addresses.push(addr);
                        invalidated_addresses.push(addr + 2);
                    },
                    _ => {},
                }
                cached_memory.write_type = MemoryWriteType::None;

                if exception.is_some() {
                    return (cycle_count, exception);
                }
            }

            // Invalidate caches.
            self.invalidate_cache(&invalidated_addresses);

            return (cycle_count, None);
        }

        // No block, make one.

        // Compromise between wasted RAM and fear of reallocation. 16 doesn't seem too bad.
        let mut block = Vec::with_capacity(16);

        let start_pc = m68000.regs.pc.0;
        loop {
            if !m68000.exceptions.is_empty() { // External exception.
                // Don't store cache if an exception occured.
                return (cycle_count, None);
            }

            let instruction = match m68000.get_next_instruction(&mut cached_memory) {
                Ok(i) => i,
                Err(e) => return (cycle_count, Some(e)),
            };

            let (cycles, exception) = m68000.execute_instruction(&mut cached_memory, &instruction);
            cycle_count += cycles;

            // Invalidate cache.
            match cached_memory.write_type {
                MemoryWriteType::ByteWord(addr) => {
                    invalidated_addresses.push(addr);
                    if addr >= start_pc && addr < m68000.regs.pc.0 {
                        return (cycle_count, exception); // Stop cache generation if it invalidated itself.
                    }
                },
                MemoryWriteType::Long(addr) => {
                    invalidated_addresses.push(addr);
                    invalidated_addresses.push(addr + 2);
                    if addr >= start_pc && addr + 2 < m68000.regs.pc.0 {
                        return (cycle_count, exception); // Stop cache generation if it invalidated itself.
                    }
                },
                _ => {},
            }
            cached_memory.write_type = MemoryWriteType::None;

            if exception.is_some() {
                // Don't store cache if an exception occured.
                return (cycle_count, exception);
            }

            block.push(instruction);

            if instruction.ends_block() {
                break;
            }
        }

        // Invalidate caches.
        self.invalidate_cache(&invalidated_addresses);

        self.instruction_cache[cache_index] = Some(block);

        (cycle_count, None)
    }

    #[inline]
    fn invalidate_cache(&mut self, addresses: &[u32]) {
        for addr in addresses {
            self.instruction_cache[self.get_cache_index(*addr)] = None;
        }
    }
}

/// The type of write to memory done by the last executed instruction.
enum MemoryWriteType {
    /// No write has been performed.
    None,
    /// A byte or word write has been done.
    ByteWord(u32),
    /// A long write has been done.
    Long(u32),
}

/// A memory access struct which remembers the memory accessed by the core in the last execution.
pub struct CachedMemoryAccess<'m, M: MemoryAccess + ?Sized> {
    memory: &'m mut M,
    write_type: MemoryWriteType,
}

impl<M: MemoryAccess + ?Sized> MemoryAccess for CachedMemoryAccess<'_, M> {
    #[inline]
    fn get_byte(&mut self, addr: u32) -> Option<u8> {
        self.memory.get_byte(addr)
    }

    #[inline]
    fn get_word(&mut self, addr: u32) -> Option<u16> {
        self.memory.get_word(addr)
    }

    #[inline]
    fn get_long(&mut self, addr: u32) -> Option<u32> {
        self.memory.get_long(addr)
    }

    #[inline]
    fn set_byte(&mut self, addr: u32, value: u8) -> Option<()> {
        self.memory.set_byte(addr, value)?;
        self.write_type = MemoryWriteType::ByteWord(addr); // Invalidate if the write is successful.
        Some(())
    }

    #[inline]
    fn set_word(&mut self, addr: u32, value: u16) -> Option<()> {
        self.memory.set_word(addr, value)?;
        self.write_type = MemoryWriteType::ByteWord(addr);
        Some(())
    }

    #[inline]
    fn set_long(&mut self, addr: u32, value: u32) -> Option<()> {
        self.memory.set_long(addr, value)?;
        self.write_type = MemoryWriteType::Long(addr);
        Some(())
    }

    #[inline]
    fn reset_instruction(&mut self) {
        self.memory.reset_instruction()
    }
}

*/
