// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Benchmark for the cpudiag test ROM with the FFI interface.
//!
//! Make sure the result of the benchmarked function is used,
//! whether by sending it to black_box, or to return it from the closure.

use core::ffi::c_void;
use std::pin::Pin;

use m68000::cpu_details;
use m68000::M68000;
use m68000_ffi::{m68000_callbacks_t, m68000_memory_result_t};
use m68000_ffi::scc68070::{m68000_scc68070_delete, m68000_scc68070_fastmem_interpreter, m68000_scc68070_new};
// use m68000_ffi::scc68070::m68000_scc68070_fastmem_disassembler_interpreter_exception;
use m68000_ffi::fastmem::m68000_fastmem_t;

extern "C" fn get_byte(addr: u32, _user_data: *mut c_void) -> m68000_memory_result_t {
    match addr {
        0x8000_2011..=0x8000_201B => {
            if addr == 0x8000_2013 {
                m68000_memory_result_t { data: 0b0000_1110, exception: 0 }
            } else {
                m68000_memory_result_t { data: 0, exception: 0 }
            }
        },
        _ => m68000_memory_result_t { data: 0, exception: 2 },
    }
}

extern "C" fn get_word(_addr: u32, _user_data: *mut c_void) -> m68000_memory_result_t {
    m68000_memory_result_t { data: 0, exception: 2 }
}

extern "C" fn get_long(_addr: u32, _user_data: *mut c_void) -> m68000_memory_result_t {
    m68000_memory_result_t { data: 0, exception: 2 }
}

extern "C" fn set_byte(addr: u32, data: u8, _user_data: *mut c_void) -> bool {
    match addr {
        0x8000_2011..=0x8000_2019 => {
            if addr == 0x8000_2019 {
                print!("{}", data as char);
            }
            true
        },
        _ => false,
    }
}

extern "C" fn set_word(addr: u32, _data: u16, _user_data: *mut c_void) -> bool {
    match addr {
        0x8000_2011..=0x8000_2019 => true,
        _ => false,
    }
}

extern "C" fn set_long(addr: u32, _data: u32, _user_data: *mut c_void) -> bool {
    match addr {
        0x8000_2011..=0x8000_2019 => true,
        _ => false,
    }
}

extern "C" fn reset_instruction(_user_data: *mut c_void) {}

struct Scc68070 {
    cpu: *mut M68000<cpu_details::Scc68070>,
    /// Pinned to make sure it never outlives the pointer in callbacks.
    _memory: Pin<Box<Memory>>,
    /// This has a pointer to _memory.
    fastmem: m68000_fastmem_t,
}

impl Drop for Scc68070 {
    fn drop(&mut self) {
        unsafe {
            m68000_scc68070_delete(self.cpu);
        }
    }
}

fn main() {
    let test_rom = std::fs::read("cpudiag40.rom").expect("no cpudiag40.rom");

    let (fastmem, memory) = make_fastmem(test_rom);
    let mut scc68070 = Scc68070 {
        cpu: m68000_scc68070_new(),
        fastmem,
        _memory: memory,
    };

    let start = std::time::Instant::now();

    for _ in 0..1_000_000_000 {
        unsafe {
            m68000_scc68070_fastmem_interpreter(scc68070.cpu, &raw mut scc68070.fastmem);
            // let mut str: [u8; 64] = [0; 64];
            // let res = m68000_scc68070_fastmem_disassembler_interpreter_exception(scc68070.cpu, &raw mut scc68070.fastmem, str.as_mut_ptr().cast(), str.len());
            // println!("{:X} {} {:?}", res.pc, res.exception, String::from_utf8_lossy(&str[..]));
        }
    }

    let delay = start.elapsed();
    println!("{delay:?}");
}

struct Memory {
    _ram1: Pin<Box<[u8]>>,
    _ram2: Pin<Box<[u8]>>,
    _bios: Pin<Box<[u8]>>,
    page_read: Pin<Box<[*const u8]>>,
    page_write: Pin<Box<[*mut u8]>>,
}

fn make_fastmem(bios_data: Vec<u8>) -> (m68000_fastmem_t, Pin<Box<Memory>>) {
    const OFFSET_BITS: u32 = 19;
    const RAM_BANK_SIZE: usize = 1 << OFFSET_BITS;
    const OFFSET_MASK: u32 = RAM_BANK_SIZE as u32 - 1;
    const PAGE_BITS: u32 = 32 - OFFSET_BITS;
    const PAGE_COUNT: usize = 1 << PAGE_BITS;

    let mut ram1 = Pin::new(vec![0; RAM_BANK_SIZE].into_boxed_slice());
    let mut ram2 = Pin::new(vec![0; RAM_BANK_SIZE].into_boxed_slice());
    let mut bios = Pin::new(vec![0; RAM_BANK_SIZE * 2].into_boxed_slice());
    bios[0..bios_data.len()].copy_from_slice(&bios_data);
    ram1[0..8].copy_from_slice(&bios[0..8]); // Copy reset vectors.

    const RAM1_PAGE: usize = 0;
    const RAM2_PAGE: usize = 0x20_0000 >> OFFSET_BITS;
    const BIOS_PAGE: usize = 0x40_0000 >> OFFSET_BITS;

    let page_read = Pin::new({
        let mut page = vec![core::ptr::null(); PAGE_COUNT].into_boxed_slice();
        page[RAM1_PAGE] = ram1.as_ptr();
        page[RAM2_PAGE] = ram2.as_ptr();
        page[BIOS_PAGE] = bios.as_ptr(); // BIOS takes two pages.
        page[BIOS_PAGE + 1] = bios.as_ptr().wrapping_add(RAM_BANK_SIZE);
        page
    });

    let page_write = Pin::new({
        let mut page = vec![core::ptr::null_mut(); PAGE_COUNT].into_boxed_slice();
        page[RAM1_PAGE] = ram1.as_mut_ptr();
        page[RAM2_PAGE] = ram2.as_mut_ptr();
        page
    });

    let mut memory = Box::pin(Memory {
        _ram1: ram1,
        _ram2: ram2,
        _bios: bios,
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
