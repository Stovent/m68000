// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Benchmark for the memory access FFI.
//!
//! Make sure the result of the benchmarked function is used,
//! whether by sending it to black_box, or to return it from the closure.
//!
//! This benchmark has a hack in the get_word/get_long functions to always return a value instead of a bus error.

use core::ffi::c_void;
use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};

use m68000_ffi::{m68000_callbacks_t, m68000_memory_result_t};
use m68000_ffi::mc68000::{m68000_mc68000_delete, m68000_mc68000_get_next_long, m68000_mc68000_get_next_word, m68000_mc68000_new_no_reset, m68000_mc68000_peek_next_word};

struct Memory {
    ram: Box<[u16]>,
}

extern "C" fn get_byte(_addr: u32, _user_data: *mut c_void) -> m68000_memory_result_t {
    m68000_memory_result_t { data: 0, exception: 2 }
}

extern "C" fn get_word(addr: u32, user_data: *mut c_void) -> m68000_memory_result_t {
    let memory = user_data as *mut Memory;
    let address = addr as usize >> 1 & 0x07; // Never return a bus error.

    unsafe {
        if address < (&mut *memory).ram.len() {
            m68000_memory_result_t {
                data: (*memory).ram[address] as u32,
                exception: 0,
            }
        } else {
            m68000_memory_result_t { data: 0, exception: 2 }
        }
    }
}

extern "C" fn get_long(addr: u32, user_data: *mut c_void) -> m68000_memory_result_t {
    let memory = user_data as *mut Memory;
    let address = addr as usize >> 1 & 0x07; // Never return a bus error.

    unsafe {
        if address < (&mut *memory).ram.len() - 1 {
            m68000_memory_result_t {
                data: ((*memory).ram[address] as u32) << 16 | (*memory).ram[address + 1] as u32,
                exception: 0,
            }
        } else {
            m68000_memory_result_t { data: 0, exception: 2 }
        }
    }
}

extern "C" fn set_byte(_addr: u32, _data: u8, _user_data: *mut c_void) -> m68000_memory_result_t {
    m68000_memory_result_t { data: 0, exception: 2 }
}

extern "C" fn set_word(_addr: u32, _data: u16, _user_data: *mut c_void) -> m68000_memory_result_t {
    m68000_memory_result_t { data: 0, exception: 2 }
}

extern "C" fn set_long(_addr: u32, _data: u32, _user_data: *mut c_void) -> m68000_memory_result_t {
    m68000_memory_result_t { data: 0, exception: 2 }
}

extern "C" fn reset_instruction(_user_data: *mut c_void) {}

pub fn criterion_benchmark(c: &mut Criterion) {
    let cpu = m68000_mc68000_new_no_reset();
    let mut memory = Memory {
        ram: vec![0u16; 0x1000].into_boxed_slice(),
    };
    let memory_ptr = &raw mut memory;
    let mut memory_callbacks = m68000_callbacks_t {
        get_byte,
        get_word,
        get_long,

        set_byte,
        set_word,
        set_long,

        reset_instruction,

        user_data: memory_ptr as *mut c_void,
    };
    // No need to use iter_batched because this base state behaves the same after each execution.

    unsafe {
        c.bench_function("m68000_mc68000_peek_next_word", |b| b.iter(|| {
            black_box(m68000_mc68000_peek_next_word(black_box(cpu), black_box(&raw mut memory_callbacks)));
        }));

        c.bench_function("m68000_mc68000_get_next_word", |b| b.iter(|| {
            black_box(m68000_mc68000_get_next_word(black_box(cpu), black_box(&raw mut memory_callbacks)));
        }));

        c.bench_function("m68000_mc68000_get_next_long", |b| b.iter(|| {
            black_box(m68000_mc68000_get_next_long(black_box(cpu), black_box(&raw mut memory_callbacks)));
        }));

        m68000_mc68000_delete(cpu);
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
