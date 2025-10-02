//! Benchmark for the cpudiag test ROM with the FFI interface.
//!
//! Make sure the result of the benchmarked function is used,
//! whether by sending it to black_box, or to return it from the closure.

use core::ffi::c_void;
use std::hint::black_box;
use std::pin::Pin;

use criterion::{criterion_group, criterion_main, Criterion};

use m68000::cpu_details::Mc68000;
use m68000::M68000;
use m68000_ffi::{m68000_callbacks_t, m68000_memory_result_t};
use m68000_ffi::mc68000::{m68000_mc68000_delete, m68000_mc68000_interpreter, m68000_mc68000_interpreter_exception, m68000_mc68000_new};

const RAM_SIZE: usize = 0x50_0000;

struct Memory68070 {
    pub memory_swap: usize,
    pub ram: Box<[u8]>,
}

extern "C" fn get_byte(addr: u32, user_data: *mut c_void) -> m68000_memory_result_t {
    let memory = user_data as *mut Memory68070;

    unsafe {
        if (addr as usize) < (&*memory).ram.len() {
            m68000_memory_result_t { data: (*memory).ram[addr as usize] as u32, exception: 0 }
        } else if addr >= 0x8000_2011 && addr <= 0x8000_201B {
            if addr == 0x8000_2013 {
                m68000_memory_result_t { data: 0b0000_1110, exception: 0 }
            } else {
                m68000_memory_result_t { data: 0, exception: 0 }
            }
        } else {
            m68000_memory_result_t { data: 0, exception: 2 }
        }
    }
}

extern "C" fn get_word(addr: u32, user_data: *mut c_void) -> m68000_memory_result_t {
    let memory = user_data as *mut Memory68070;

    unsafe {
        let address = if (*memory).memory_swap < 4 {
            (*memory).memory_swap += 1;
            addr + 0x40_0000
        } else {
            addr
        } as usize;

        if address < (&*memory).ram.len() - 1 {
            m68000_memory_result_t {
                data: u16::from_be_bytes((&mut *memory).ram[address..address + 2].try_into().unwrap()) as u32,
                exception: 0,
            }
        } else {
            m68000_memory_result_t { data: 0, exception: 2 }
        }
    }
}

extern "C" fn get_long(addr: u32, user_data: *mut c_void) -> m68000_memory_result_t {
    let memory = user_data as *mut Memory68070;

    unsafe {
        let address = if (*memory).memory_swap < 4 {
            (*memory).memory_swap += 2;
            addr + 0x40_0000
        } else {
            addr
        } as usize;

        if address < (&*memory).ram.len() - 3 {
            m68000_memory_result_t {
                data: u32::from_be_bytes((&mut *memory).ram[address..address + 4].try_into().unwrap()),
                exception: 0,
            }
        } else {
            m68000_memory_result_t { data: 0, exception: 2 }
        }
    }
}

extern "C" fn set_byte(addr: u32, data: u8, user_data: *mut c_void) -> m68000_memory_result_t {
    let memory = user_data as *mut Memory68070;

    unsafe {
        if (addr as usize) < (&mut *memory).ram.len() {
            (*memory).ram[addr as usize] = data;
            m68000_memory_result_t { data: 0, exception: 0 }
        } else if addr >= 0x8000_2011 && addr <= 0x8000_2019 {
            m68000_memory_result_t { data: 0, exception: 0 }
        } else {
            m68000_memory_result_t { data: 0, exception: 2 }
        }
    }
}

extern "C" fn set_word(addr: u32, data: u16, user_data: *mut c_void) -> m68000_memory_result_t {
    let memory = user_data as *mut Memory68070;

    unsafe {
        if (addr as usize) < (&mut *memory).ram.len() - 1 {
            (*memory).ram[addr as usize] = (data >> 8) as u8;
            (*memory).ram[addr as usize + 1] = data as u8;
            m68000_memory_result_t { data: 0, exception: 0 }
        } else if addr >= 0x8000_2011 && addr <= 0x8000_2019 {
            m68000_memory_result_t { data: 0, exception: 0 }
        } else {
            m68000_memory_result_t { data: 0, exception: 2 }
        }
    }
}

extern "C" fn set_long(addr: u32, data: u32, user_data: *mut c_void) -> m68000_memory_result_t {
    let memory = user_data as *mut Memory68070;

    unsafe {
        if (addr as usize) < (&mut *memory).ram.len() - 3 {
            (*memory).ram[addr as usize] = (data >> 24) as u8;
            (*memory).ram[addr as usize + 1] = (data >> 16) as u8;
            (*memory).ram[addr as usize + 2] = (data >> 8) as u8;
            (*memory).ram[addr as usize + 3] = data as u8;
            m68000_memory_result_t { data: 0, exception: 0 }
        } else if addr >= 0x8000_2011 && addr <= 0x8000_2019 {
            m68000_memory_result_t { data: 0, exception: 0 }
        } else {
            m68000_memory_result_t { data: 0, exception: 2 }
        }
    }
}

extern "C" fn reset_instruction(_user_data: *mut c_void) {}

struct Scc68070 {
    cpu: *mut M68000<Mc68000>,
    /// Pinned to make sure it never outlives the pointer in callbacks.
    _memory: Pin<Box<Memory68070>>,
    /// This has a pointer to _memory.
    callbacks: m68000_callbacks_t,
}

impl Drop for Scc68070 {
    fn drop(&mut self) {
        unsafe {
            m68000_mc68000_delete(self.cpu);
        }
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let test_rom = std::fs::read("cpudiag40.rom").expect("no cpudiag40.rom");
    let setup = || {
        let mut memory = Box::pin(Memory68070 {
            memory_swap: 0,
            ram: vec![0u8; RAM_SIZE].into_boxed_slice(),
        });

        // Load the program in memory.
        let begin = 0x40_0000;
        let end = begin + test_rom.len();
        memory.ram[begin..end].copy_from_slice(&test_rom);

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

        Scc68070 {
            cpu: m68000_mc68000_new(),
            _memory: memory,
            callbacks: memory_callbacks,
        }
    };

    c.bench_function("cpudiag m68000_mc68000_interpreter", |b|
        b.iter_batched_ref(setup, |scc68070| {
            unsafe {
                for _ in 0..10_000_000 {
                    black_box(m68000_mc68000_interpreter(scc68070.cpu, &raw mut scc68070.callbacks));
                }
            }
        }, criterion::BatchSize::SmallInput)
    );

    c.bench_function("cpudiag m68000_mc68000_interpreter_exception", |b|
        b.iter_batched_ref(setup, |scc68070| {
            unsafe {
                for _ in 0..10_000_000 {
                    black_box(m68000_mc68000_interpreter_exception(scc68070.cpu, &raw mut scc68070.callbacks));
                }
            }
        }, criterion::BatchSize::SmallInput)
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
