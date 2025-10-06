// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Benchmark for the interpreter methods with the FFI interface and fastmem.
//!
//! Make sure the result of the benchmarked function is used,
//! whether by sending it to black_box, or to return it from the closure.

use core::ffi::c_void;
use core::pin::Pin;
use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};

use m68000_ffi::{m68000_callbacks_t, m68000_memory_result_t};
use m68000_ffi::fastmem::m68000_fastmem_t;
use m68000_ffi::mc68000::{m68000_mc68000_delete, m68000_mc68000_fastmem_get_next_long, m68000_mc68000_fastmem_get_next_word, m68000_mc68000_new_no_reset, m68000_mc68000_fastmem_peek_next_word};

extern "C" fn get_byte(_addr: u32, _user_data: *mut c_void) -> m68000_memory_result_t {
    m68000_memory_result_t { data: 0, exception: 2 }
}

extern "C" fn get_word(_addr: u32, _user_data: *mut c_void) -> m68000_memory_result_t {
    m68000_memory_result_t { data: 0, exception: 2 }
}

extern "C" fn get_long(_addr: u32, _user_data: *mut c_void) -> m68000_memory_result_t {
    m68000_memory_result_t { data: 0, exception: 2 }
}

extern "C" fn set_byte(_addr: u32, _data: u8, _user_data: *mut c_void) -> bool {
    false
}

extern "C" fn set_word(_addr: u32, _data: u16, _user_data: *mut c_void) -> bool {
    false
}

extern "C" fn set_long(_addr: u32, _data: u32, _user_data: *mut c_void) -> bool {
    false
}

extern "C" fn reset_instruction(_user_data: *mut c_void) {}

pub fn criterion_benchmark(c: &mut Criterion) {
    let cpu = m68000_mc68000_new_no_reset();

    let (mut fastmem, _memory) = make_fastmem();

    unsafe {
        c.bench_function("m68000_mc68000_fastmem_peek_next_word", |b| b.iter(|| {
            black_box(m68000_mc68000_fastmem_peek_next_word(black_box(cpu), black_box(&raw mut fastmem)));
        }));

        c.bench_function("m68000_mc68000_fastmem_get_next_word", |b| b.iter(|| {
            black_box(m68000_mc68000_fastmem_get_next_word(black_box(cpu), black_box(&raw mut fastmem)));
        }));

        c.bench_function("m68000_mc68000_fastmem_get_next_long", |b| b.iter(|| {
            black_box(m68000_mc68000_fastmem_get_next_long(black_box(cpu), black_box(&raw mut fastmem)));
        }));

        m68000_mc68000_delete(cpu);
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

struct Memory {
    _code: Pin<Box<[u8]>>,
    page_read: Pin<Box<[*const u8]>>,
    page_write: Pin<Box<[*mut u8]>>,
}

fn make_fastmem() -> (m68000_fastmem_t, Pin<Box<Memory>>) {
    const OFFSET_BITS: u32 = 24;
    const RAM_BANK_SIZE: usize = 1 << OFFSET_BITS;
    const OFFSET_MASK: u32 = RAM_BANK_SIZE as u32 - 1;
    const PAGE_BITS: u32 = 32 - OFFSET_BITS;
    const PAGE_COUNT: usize = 1 << PAGE_BITS;

    let code = Pin::new(vec![0; RAM_BANK_SIZE].into_boxed_slice());

    const CODE_PAGE: usize = 0;

    let page_read = Pin::new({
        let mut page = vec![core::ptr::null(); PAGE_COUNT].into_boxed_slice();
        page[CODE_PAGE] = code.as_ptr();
        page
    });

    let page_write = Pin::new(vec![core::ptr::null_mut(); PAGE_COUNT].into_boxed_slice());

    let mut memory = Box::pin(Memory {
        _code: code,
        page_read,
        page_write,
    });
    let memory_ptr = &raw mut *memory as *mut c_void;

    let memory_callbacks = m68000_callbacks_t {
        get_byte,
        get_word,
        get_long,

        set_byte,
        set_word,
        set_long,

        reset_instruction,

        user_data: memory_ptr,
    };

    (m68000_fastmem_t {
        page_read: memory.page_read.as_ptr(),
        page_write: memory.page_write.as_mut_ptr(),
        page_shift: OFFSET_BITS,
        offset_mask: OFFSET_MASK,
        slow_memory: memory_callbacks,
    }, memory)
}
