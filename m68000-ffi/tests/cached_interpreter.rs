// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! FFI cached interpreters unit tests.

use core::ffi::c_void;

use m68000::addressing_modes::AddressingMode;
use m68000::assembler::{add, bra};
use m68000::instruction::{Direction, Size};
use m68000_ffi::cached_interpreter::*;
use m68000_ffi::mc68000::*;
use m68000_ffi::{m68000_callbacks_t, m68000_memory_result_t};

struct Memory {
    memory: Box<[u16]>,
}

extern "C" fn get_byte(_addr: u32, _user_data: *mut c_void) -> m68000_memory_result_t {
    panic!();
}

extern "C" fn get_word(addr: u32, user_data: *mut c_void) -> m68000_memory_result_t {
    let memory = unsafe { &mut *(user_data as *mut Memory) };

    // array of words, not bytes.
    // Guaranteed even address.
    if addr < (memory.memory.len() << 1) as u32 {
        m68000_memory_result_t { data: memory.memory[(addr >> 1) as usize] as u32, exception: 0 }
    } else {
        m68000_memory_result_t { data: 0, exception: 2 }
    }
}

extern "C" fn get_long(_addr: u32, _user_data: *mut c_void) -> m68000_memory_result_t {
    panic!();
}

extern "C" fn set_byte(_addr: u32, _data: u8, _user_data: *mut c_void) -> bool {
    panic!();
}

extern "C" fn set_word(_addr: u32, _data: u16, _user_data: *mut c_void) -> bool {
    panic!();
}

extern "C" fn set_long(_addr: u32, _data: u32, _user_data: *mut c_void) -> bool {
    panic!();
}

extern "C" fn reset_instruction(_user_data: *mut c_void) {}

#[test]
fn cached_interpreter() {
    let mut mem = vec![];
    mem.extend_from_slice(&add(0, Direction::DstReg, Size::Long, AddressingMode::Immediate(1)));
    mem.extend_from_slice(&bra(-8));
    let mem_end = (mem.len() << 1) as u32;

    let mut memory = Memory {
        memory: mem.into_boxed_slice(),
    };
    let user_data = (&raw mut memory) as *mut c_void;
    let mut callbacks = m68000_callbacks_t {
        get_byte,
        get_word,
        get_long,
        set_byte,
        set_word,
        set_long,
        reset_instruction,
        user_data,
    };

    let cpu = m68000_mc68000_new_no_reset();
    let single_block = m68000_new_cached_interpreter_single_block(0, mem_end);
    let flat_block = m68000_new_cached_interpreter_flat_block(0, mem_end);
    let trie_block = m68000_new_cached_interpreter_trie_block();

    unsafe {
        let regs = m68000_mc68000_registers(cpu);
        assert_eq!((*regs).d[0].0, 0);
        assert_eq!((*regs).pc.0, 0);

        let count = m68000_mc68000_interpreter(cpu, &mut callbacks); // ADD
        assert_eq!(count, 16); // Assert it actually executed.
        assert_eq!((*regs).d[0].0, 1);
        assert_eq!((*regs).pc.0, 6);

        let count = m68000_mc68000_cached_interpreter_single_block_interpreter(single_block, cpu, &mut callbacks); // BRA
        assert_eq!(count, 10); // Assert it actually executed.
        assert_eq!((*regs).d[0].0, 1);
        assert_eq!((*regs).pc.0, 0);

        let count = m68000_mc68000_cached_interpreter_flat_block_interpreter(flat_block, cpu, &mut callbacks); // ADD + BRA
        assert_eq!(count, 26); // Assert it actually executed.
        assert_eq!((*regs).d[0].0, 2);
        assert_eq!((*regs).pc.0, 0);

        let count = m68000_mc68000_cached_interpreter_trie_block_interpreter(trie_block, cpu, &mut callbacks); // ADD + BRA
        assert_eq!(count, 26); // Assert it actually executed.
        assert_eq!((*regs).d[0].0, 3);
        assert_eq!((*regs).pc.0, 0);

        m68000_mc68000_delete(cpu);
        m68000_delete_cached_interpreter_single_block(single_block);
        m68000_delete_cached_interpreter_flat_block(flat_block);
        m68000_delete_cached_interpreter_trie_block(trie_block);
    }
}
