// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/** @file c_bindings.c
 * @brief File compiled along with m68000-fii to make sure the bindings compile.
 */

#include "../include/m68000/m68000-ffi.h"

int main(void) {
    bool* stop;
    uint8_t vector = 0;
    size_t cycles;
    uint16_t data_word;
    uint32_t data_long;
    char dis[2];
    m68000_disassembler_result_t disassembler_result;
    m68000_disassembler_exception_result_t disassembler_exception_result;
    m68000_exception_result_t exception_result;
    m68000_memory_result_t memory_result;
    const m68000_registers_t* registers;
    m68000_registers_t registers_mut;
    m68000_callbacks_t memory;
    m68000_fastmem_t fastmem;

    M68000_Scc68070* scc68070;
    M68000_Mc68000* mc68000;

    mc68000 = m68000_mc68000_new();
    mc68000 = m68000_mc68000_new_no_reset();
    scc68070 = m68000_scc68070_new();
    scc68070 = m68000_scc68070_new_no_reset();

    cycles = m68000_mc68000_cycle(mc68000, &memory, 0);
    cycles = m68000_scc68070_cycle(scc68070, &memory, 0);
    exception_result = m68000_mc68000_cycle_until_exception(mc68000, &memory, 0);
    exception_result = m68000_scc68070_cycle_until_exception(scc68070, &memory, 0);
    exception_result = m68000_mc68000_loop_until_exception_stop(mc68000, &memory);
    exception_result = m68000_scc68070_loop_until_exception_stop(scc68070, &memory);
    cycles = m68000_mc68000_interpreter(mc68000, &memory);
    cycles = m68000_scc68070_interpreter(scc68070, &memory);
    exception_result = m68000_mc68000_interpreter_exception(mc68000, &memory);
    exception_result = m68000_scc68070_interpreter_exception(scc68070, &memory);
    disassembler_result = m68000_mc68000_disassembler_interpreter(mc68000, &memory, dis, sizeof dis);
    disassembler_result = m68000_scc68070_disassembler_interpreter(scc68070, &memory, dis, sizeof dis);
    disassembler_exception_result = m68000_mc68000_disassembler_interpreter_exception(mc68000, &memory, dis, sizeof dis);
    disassembler_exception_result = m68000_scc68070_disassembler_interpreter_exception(scc68070, &memory, dis, sizeof dis);
    m68000_mc68000_exception(mc68000, vector);
    m68000_scc68070_exception(scc68070, vector);

    memory_result = m68000_mc68000_peek_next_word(mc68000, &memory);
    memory_result = m68000_scc68070_peek_next_word(scc68070, &memory);
    memory_result = m68000_mc68000_get_next_word(mc68000, &memory);
    memory_result = m68000_scc68070_get_next_word(scc68070, &memory);
    memory_result = m68000_mc68000_get_next_long(mc68000, &memory);
    memory_result = m68000_scc68070_get_next_long(scc68070, &memory);

    registers = m68000_mc68000_registers(mc68000);
    registers = m68000_scc68070_registers(scc68070);
    registers = m68000_mc68000_registers_mut(mc68000);
    registers = m68000_scc68070_registers_mut(scc68070);
    registers_mut = m68000_mc68000_get_registers(mc68000);
    registers_mut = m68000_scc68070_get_registers(scc68070);
    m68000_mc68000_set_registers(mc68000, *registers);
    m68000_scc68070_set_registers(scc68070, *registers);
    stop = m68000_mc68000_stop_mut(mc68000);
    stop = m68000_scc68070_stop_mut(scc68070);

    m68000_mc68000_delete(mc68000);
    m68000_scc68070_delete(scc68070);

    // Fastmem section

    cycles = m68000_mc68000_fastmem_cycle(mc68000, &fastmem, 0);
    cycles = m68000_scc68070_fastmem_cycle(scc68070, &fastmem, 0);
    exception_result = m68000_mc68000_fastmem_cycle_until_exception(mc68000, &fastmem, 0);
    exception_result = m68000_scc68070_fastmem_cycle_until_exception(scc68070, &fastmem, 0);
    exception_result  = m68000_mc68000_fastmem_loop_until_exception_stop(mc68000, &fastmem);
    exception_result  = m68000_scc68070_fastmem_loop_until_exception_stop(scc68070, &fastmem);
    cycles = m68000_mc68000_fastmem_interpreter(mc68000, &fastmem);
    cycles = m68000_scc68070_fastmem_interpreter(scc68070, &fastmem);
    exception_result = m68000_mc68000_fastmem_interpreter_exception(mc68000, &fastmem);
    exception_result = m68000_scc68070_fastmem_interpreter_exception(scc68070, &fastmem);
    disassembler_result = m68000_mc68000_fastmem_disassembler_interpreter(mc68000, &fastmem, dis, sizeof dis);
    disassembler_result = m68000_scc68070_fastmem_disassembler_interpreter(scc68070, &fastmem, dis, sizeof dis);
    disassembler_exception_result = m68000_mc68000_fastmem_disassembler_interpreter_exception(mc68000, &fastmem, dis, sizeof dis);
    disassembler_exception_result = m68000_scc68070_fastmem_disassembler_interpreter_exception(scc68070, &fastmem, dis, sizeof dis);
    memory_result = m68000_mc68000_fastmem_peek_next_word(mc68000, &fastmem);
    memory_result = m68000_scc68070_fastmem_peek_next_word(scc68070, &fastmem);
    memory_result = m68000_mc68000_fastmem_get_next_word(mc68000, &fastmem);
    memory_result = m68000_scc68070_fastmem_get_next_word(scc68070, &fastmem);
    memory_result = m68000_mc68000_fastmem_get_next_long(mc68000, &fastmem);
    memory_result = m68000_scc68070_fastmem_get_next_long(scc68070, &fastmem);

    // Cached interpreters section

    CachedInterpreterFlatBlock* flat_block = m68000_cached_interpreter_flat_block_new(0, 1);
    cycles = m68000_mc68000_cached_interpreter_flat_block_interpreter(flat_block, mc68000, &memory);
    cycles = m68000_scc68070_cached_interpreter_flat_block_interpreter(flat_block, scc68070, &memory);
    exception_result = m68000_mc68000_cached_interpreter_flat_block_interpreter_exception(flat_block, mc68000, &memory);
    exception_result = m68000_scc68070_cached_interpreter_flat_block_interpreter_exception(flat_block, scc68070, &memory);
    cycles = m68000_mc68000_fastmem_cached_interpreter_flat_block_interpreter(flat_block, mc68000, &fastmem);
    cycles = m68000_scc68070_fastmem_cached_interpreter_flat_block_interpreter(flat_block, scc68070, &fastmem);
    exception_result = m68000_mc68000_fastmem_cached_interpreter_flat_block_interpreter_exception(flat_block, mc68000, &fastmem);
    exception_result = m68000_scc68070_fastmem_cached_interpreter_flat_block_interpreter_exception(flat_block, scc68070, &fastmem);
    m68000_cached_interpreter_flat_block_delete(flat_block);

    CachedInterpreterSingleBlock* single_block = m68000_cached_interpreter_single_block_new(0, 1);
    cycles = m68000_mc68000_cached_interpreter_single_block_interpreter(single_block, mc68000, &memory);
    cycles = m68000_scc68070_cached_interpreter_single_block_interpreter(single_block, scc68070, &memory);
    exception_result = m68000_mc68000_cached_interpreter_single_block_interpreter_exception(single_block, mc68000, &memory);
    exception_result = m68000_scc68070_cached_interpreter_single_block_interpreter_exception(single_block, scc68070, &memory);
    cycles = m68000_mc68000_fastmem_cached_interpreter_single_block_interpreter(single_block, mc68000, &fastmem);
    cycles = m68000_scc68070_fastmem_cached_interpreter_single_block_interpreter(single_block, scc68070, &fastmem);
    exception_result = m68000_mc68000_fastmem_cached_interpreter_single_block_interpreter_exception(single_block, mc68000, &fastmem);
    exception_result = m68000_scc68070_fastmem_cached_interpreter_single_block_interpreter_exception(single_block, scc68070, &fastmem);
    m68000_cached_interpreter_single_block_delete(single_block);

    CachedInterpreterTrieBlock* trie_block = m68000_cached_interpreter_trie_block_new();
    cycles = m68000_mc68000_cached_interpreter_trie_block_interpreter(trie_block, mc68000, &memory);
    cycles = m68000_scc68070_cached_interpreter_trie_block_interpreter(trie_block, scc68070, &memory);
    exception_result = m68000_mc68000_cached_interpreter_trie_block_interpreter_exception(trie_block, mc68000, &memory);
    exception_result = m68000_scc68070_cached_interpreter_trie_block_interpreter_exception(trie_block, scc68070, &memory);
    cycles = m68000_mc68000_fastmem_cached_interpreter_trie_block_interpreter(trie_block, mc68000, &fastmem);
    cycles = m68000_scc68070_fastmem_cached_interpreter_trie_block_interpreter(trie_block, scc68070, &fastmem);
    exception_result = m68000_mc68000_fastmem_cached_interpreter_trie_block_interpreter_exception(trie_block, mc68000, &fastmem);
    exception_result = m68000_scc68070_fastmem_cached_interpreter_trie_block_interpreter_exception(trie_block, scc68070, &fastmem);
    m68000_cached_interpreter_trie_block_delete(trie_block);

    return 0;
}
