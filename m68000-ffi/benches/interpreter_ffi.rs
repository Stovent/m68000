//! Benchmark for the cpudiag test ROM with the FFI interface.
//!
//! Make sure the result of the benchmarked function is used,
//! whether by sending it to black_box, or to return it from the closure.

use core::ffi::c_void;
use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};

use m68000_ffi::{m68000_callbacks_t, m68000_memory_result_t};
use m68000_ffi::mc68000::{m68000_mc68000_delete, m68000_mc68000_interpreter, m68000_mc68000_interpreter_exception, m68000_mc68000_new};

type CodeArray = [u16; 6];

extern "C" fn get_byte(_addr: u32, _user_data: *mut c_void) -> m68000_memory_result_t {
    m68000_memory_result_t { data: 0, exception: 2 }
}

extern "C" fn get_word(addr: u32, user_data: *mut c_void) -> m68000_memory_result_t {
    let memory = user_data as *mut CodeArray;
    let address = addr as usize >> 1;

    unsafe {
        if address < (*memory).len() {
            m68000_memory_result_t {
                data: (*memory)[address] as u32,
                exception: 0,
            }
        } else {
            m68000_memory_result_t { data: 0, exception: 2 }
        }
    }
}

extern "C" fn get_long(addr: u32, user_data: *mut c_void) -> m68000_memory_result_t {
    let memory = user_data as *mut CodeArray;
    let address = addr as usize >> 1;

    unsafe {
        if address < (*memory).len() {
            m68000_memory_result_t {
                data: ((*memory)[address] as u32) << 16 | (*memory)[address + 1] as u32,
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
    let cpu = m68000_mc68000_new();
    let bra = 0x6000_u16;
    let mut code: CodeArray = [
        0x0000, 0x1000, // Initial SSP
        0x0000, 0x0008, // Initial PC
        bra, 0xFFFE, // bra.s
    ];
    let memory_ptr = &raw mut code as *mut c_void;
    let mut memory_callbacks = m68000_callbacks_t {
        get_byte,
        get_word,
        get_long,

        set_byte,
        set_word,
        set_long,

        reset_instruction,

        user_data: memory_ptr,
    };

    unsafe {
        m68000_mc68000_interpreter(cpu, &raw mut memory_callbacks); // Fetch the reset vectors on the first call.
        // The benchmarks executes the same instruction so no need for dedicated input management.

        c.bench_function("m68000_mc68000_interpreter", |b| b.iter(|| {
            black_box(m68000_mc68000_interpreter(black_box(cpu), black_box(&raw mut memory_callbacks)));
        }));

        c.bench_function("m68000_mc68000_interpreter_exception", |b| b.iter(|| {
            black_box(m68000_mc68000_interpreter_exception(black_box(cpu), black_box(&raw mut memory_callbacks)));
        }));

        m68000_mc68000_delete(cpu);
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
