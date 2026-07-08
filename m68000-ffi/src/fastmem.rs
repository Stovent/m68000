// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Software fastmem implementation.
//!
//! https://wheremyfoodat.github.io/software-fastmem/

use core::hint::likely;

use crate::{m68000_callbacks_t, MemoryAccess};

/// Fast memory access with a software paging.
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct m68000_fastmem_t {
    /// Array of pointers to the pages.
    pub page_read: *const *const u8,
    /// Array of pointers to the pages.
    pub page_write: *const *mut u8,
    /// Shift applied to the address for indexing the first pointer.
    ///
    /// Can be different than [Self::offset_mask].
    pub page_shift: u32,
    /// Mask applied to the address for indexing the second pointer.
    pub offset_mask: u32,
    /// User callbacks for when the address is slow memory.
    pub slow_memory: m68000_callbacks_t,
}

impl m68000_fastmem_t {
    #[inline(always)]
    const fn get_read_page(&self, addr: u32) -> *const u8 {
        let page_index = (addr >> self.page_shift) as usize;
        // SAFETY: it is the user's responsibility.
        unsafe { *self.page_read.wrapping_add(page_index) }
    }

    #[inline(always)]
    const fn get_write_page(&self, addr: u32) -> *mut u8 {
        let page_index = (addr >> self.page_shift) as usize;
        // SAFETY: it is the user's responsibility.
        unsafe { *self.page_write.wrapping_add(page_index) }
    }

    #[inline(always)]
    const fn read_u16(&self, addr: u32, page: *const u8) -> u16 {
        let offset = (addr & self.offset_mask) as usize;
        let ptr = page.wrapping_add(offset).cast::<u16>();
        unsafe { u16::from_be(ptr.read_unaligned()) }
    }

    #[inline(always)]
    const fn write_u16(&self, addr: u32, value: u16, page: *mut u8) {
        let offset = (addr & self.offset_mask) as usize;
        let ptr = page.wrapping_add(offset).cast::<u16>();
        unsafe { ptr.write_unaligned(value.to_be()); }
    }
}

impl MemoryAccess for m68000_fastmem_t {
    fn get_byte(&mut self, addr: u32) -> Option<u8> {
        let page = self.get_read_page(addr);

        if likely(!page.is_null()) {
            let offset = (addr & self.offset_mask) as usize;
            unsafe { Some(*page.wrapping_add(offset)) }
        } else {
            self.slow_memory.get_byte(addr)
        }
    }

    // Addresses are even so it can't cross page boundaries.
    fn get_word(&mut self, addr: u32) -> Option<u16> {
        let page = self.get_read_page(addr);

        if likely(!page.is_null()) {
            Some(self.read_u16(addr, page))
        } else {
            self.slow_memory.get_word(addr)
        }
    }

    fn get_long(&mut self, addr: u32) -> Option<u32> {
        let page = self.get_read_page(addr);
        let addr_2 = addr + 2;
        let page_2 = self.get_read_page(addr_2);

        if likely(!page.is_null() && !page_2.is_null()) {
            if likely(page == page_2) { // Same page.
                let offset = (addr & self.offset_mask) as usize;
                let ptr = page.wrapping_add(offset).cast::<u32>();
                unsafe { Some(u32::from_be(ptr.read_unaligned())) }
            } else { // consecutive pages.
                let high = self.read_u16(addr, page) as u32;
                let low = self.read_u16(addr_2, page_2) as u32;

                Some((high << 16) | low)
            }
        } else {
            self.slow_memory.get_long(addr)
        }
    }

    fn set_byte(&mut self, addr: u32, value: u8) -> Option<()> {
        let page = self.get_write_page(addr);

        if likely(!page.is_null()) {
            let offset = (addr & self.offset_mask) as usize;
            unsafe { *page.wrapping_add(offset) = value; }
            Some(())
        } else {
            self.slow_memory.set_byte(addr, value)
        }
    }

    // Addresses are even so it can't cross page boundaries.
    fn set_word(&mut self, addr: u32, value: u16) -> Option<()> {
        let page = self.get_write_page(addr);

        if likely(!page.is_null()) {
            self.write_u16(addr, value, page);
            Some(())
        } else {
            self.slow_memory.set_word(addr, value)
        }
    }

    fn set_long(&mut self, addr: u32, value: u32) -> Option<()> {
        let page = self.get_write_page(addr);
        let addr_2 = addr + 2;
        let page_2 = self.get_write_page(addr_2);

        if likely(!page.is_null() && !page_2.is_null()) {
            if likely(page == page_2) { // Same page.
                let offset = (addr & self.offset_mask) as usize;
                let ptr = page.wrapping_add(offset).cast::<u32>();
                unsafe { ptr.write_unaligned(value.to_be()); }
            } else { // consecutive pages.
                self.write_u16(addr, (value >> 16) as u16, page);
                self.write_u16(addr_2, value as u16, page_2);
            }
            Some(())
        } else {
            self.slow_memory.set_long(addr, value)
        }
    }

    fn reset_instruction(&mut self) {
        self.slow_memory.reset_instruction()
    }
}

#[macro_export]
macro_rules! cinterface_fastmem {
    ($cpu:ident, $cpu_details:ty) => {
        paste! {
            /// Runs the CPU for `cycles` number of cycles.
            ///
            /// This function executes **at least** the given number of cycles.
            /// Returns the number of cycles actually executed.
            ///
            /// If you ask to execute 4 cycles but the next instruction takes 6 cycles to execute,
            /// it will be executed and the 2 extra cycles will be subtracted in the next call.
            #[no_mangle]
            pub unsafe extern "C" fn [<m68000_ $cpu _fastmem_cycle>](m68000: *mut M68000<$cpu_details>, memory: *mut m68000_fastmem_t, cycles: usize) -> usize {
                unsafe {
                    (*m68000).cycle(&mut *memory, cycles)
                }
            }

            /// Runs the CPU until either an exception occurs or `cycle` cycles have been executed.
            ///
            /// This function executes **at least** the given number of cycles.
            /// Returns the number of cycles actually executed, and the exception that occured if any.
            ///
            /// If you ask to execute 4 cycles but the next instruction takes 6 cycles to execute,
            /// it will be executed and the 2 extra cycles will be subtracted in the next call.
            #[no_mangle]
            pub unsafe extern "C" fn [<m68000_ $cpu _fastmem_cycle_until_exception>](m68000: *mut M68000<$cpu_details>, memory: *mut m68000_fastmem_t, cycles: usize) -> m68000_exception_result_t {
                unsafe {
                    let (cycles, vector) = (*m68000).cycle_until_exception(&mut *memory, cycles);
                    m68000_exception_result_t { cycles, exception: vector.unwrap_or(0) }
                }
            }

            /// Runs indefinitely until an exception or STOP instruction occurs.
            ///
            /// Returns the number of cycles executed and the exception that occured.
            /// If exception is None, this means the CPU has executed a STOP instruction.
            #[no_mangle]
            pub unsafe extern "C" fn [<m68000_ $cpu _fastmem_loop_until_exception_stop>](m68000: *mut M68000<$cpu_details>, memory: *mut m68000_fastmem_t) -> m68000_exception_result_t {
                unsafe {
                    let (cycles, vector) = (*m68000).loop_until_exception_stop(&mut *memory);
                    m68000_exception_result_t { cycles, exception: vector.unwrap_or(0) }
                }
            }

            /// Executes the next instruction, returning the cycle count necessary to execute it.
            #[no_mangle]
            pub unsafe extern "C" fn [<m68000_ $cpu _fastmem_interpreter>](m68000: *mut M68000<$cpu_details>, memory: *mut m68000_fastmem_t) -> usize {
                unsafe {
                    (*m68000).interpreter(&mut *memory)
                }
            }

            /// Executes the next instruction, returning the cycle count necessary to execute it.
            #[no_mangle]
            pub unsafe extern "C" fn [<m68000_ $cpu _fastmem_cached_interpreter_1>](m68000: *mut M68000<$cpu_details>, memory: *mut m68000_fastmem_t) -> usize {
                unsafe {
                    (*m68000).cached_interpreter_1(&mut *memory)
                }
            }

            /// Executes the next instruction, returning the cycle count necessary to execute it.
            #[no_mangle]
            pub unsafe extern "C" fn [<m68000_ $cpu _fastmem_cached_interpreter_2>](m68000: *mut M68000<$cpu_details>, memory: *mut m68000_fastmem_t) -> usize {
                unsafe {
                    (*m68000).cached_interpreter_2(&mut *memory)
                }
            }

            /// Executes the next instruction, returning the cycle count necessary to execute it,
            /// and the vector of the exception that occured during the execution if any.
            ///
            /// To process the returned exception, call `m68000_*_exception`.
            #[no_mangle]
            pub unsafe extern "C" fn [<m68000_ $cpu _fastmem_interpreter_exception>](m68000: *mut M68000<$cpu_details>, memory: *mut m68000_fastmem_t) -> m68000_exception_result_t {
                unsafe {
                    let (cycles, vector) = (*m68000).interpreter_exception(&mut *memory);
                    m68000_exception_result_t { cycles, exception: vector.unwrap_or(0) }
                }
            }

            /// Executes and disassembles the next instruction, returning the disassembler string and the cycle count necessary to execute it.
            ///
            /// `str` is a pointer to a C string buffer where the disassembled instruction will be written.
            /// `len` is the maximum size of the buffer, null-charactere included.
            #[no_mangle]
            pub unsafe extern "C" fn [<m68000_ $cpu _fastmem_disassembler_interpreter>](m68000: *mut M68000<$cpu_details>, memory: *mut m68000_fastmem_t, str: *mut c_char, len: usize) -> m68000_disassembler_result_t {
                unsafe {
                    let (pc, dis, cycles) = (*m68000).disassembler_interpreter(&mut *memory);

                    let cstring = CString::new(dis).expect("New CString for disassembler failed")
                        .into_bytes_with_nul();
                    let raw_cstring = std::mem::transmute::<*const u8, *const c_char>(cstring.as_ptr());

                    if cstring.len() <= len {
                        str.copy_from_nonoverlapping(raw_cstring, cstring.len());
                    } else {
                        str.copy_from_nonoverlapping(raw_cstring, len - 1);
                        *str.add(len - 1) = 0;
                    }

                    m68000_disassembler_result_t {
                        cycles,
                        pc,
                    }
                }
            }

            /// Executes and disassembles the next instruction, returning the disassembled string, the cycle count necessary to execute it,
            /// and the vector of the exception that occured during the execution if any.
            ///
            /// To process the returned exception, call `m68000_*_exception`.
            ///
            /// `str` is a pointer to a C string buffer where the disassembled instruction will be written.
            /// `len` is the maximum size of the buffer.
            #[no_mangle]
            pub unsafe extern "C" fn [<m68000_ $cpu _fastmem_disassembler_interpreter_exception>](m68000: *mut M68000<$cpu_details>, memory: *mut m68000_fastmem_t, str: *mut c_char, len: usize) -> m68000_disassembler_exception_result_t {
                unsafe {
                    let (pc, dis, cycles, vector) = (*m68000).disassembler_interpreter_exception(&mut *memory);

                    let cstring = CString::new(dis).expect("New CString for disassembler failed")
                        .into_bytes_with_nul();
                    let raw_cstring = std::mem::transmute::<*const u8, *const c_char>(cstring.as_ptr());

                    if cstring.len() <= len {
                        str.copy_from_nonoverlapping(raw_cstring, cstring.len());
                    } else {
                        str.copy_from_nonoverlapping(raw_cstring, len - 1);
                        *str.add(len - 1) = 0;
                    }

                    m68000_disassembler_exception_result_t {
                        cycles,
                        pc,
                        exception: vector.unwrap_or(0) ,
                    }
                }
            }

            /// Returns the 16-bits word at the current PC value of the given core and advances PC by 2.
            #[no_mangle]
            pub unsafe extern "C" fn [<m68000_ $cpu _fastmem_get_next_word>](m68000: *mut M68000<$cpu_details>, memory: *mut m68000_fastmem_t) -> m68000_memory_result_t {
                unsafe {
                    match (*m68000).get_next_word(&mut *memory) {
                        Ok(data) => m68000_memory_result_t {
                            data: data as u32,
                            exception: 0,
                        },
                        Err(vec) => m68000_memory_result_t {
                            data: 0,
                            exception: vec,
                        },
                    }
                }
            }

            /// Returns the 32-bits long at the current PC value of the given core and advances PC by 4.
            #[no_mangle]
            pub unsafe extern "C" fn [<m68000_ $cpu _fastmem_get_next_long>](m68000: *mut M68000<$cpu_details>, memory: *mut m68000_fastmem_t) -> m68000_memory_result_t {
                unsafe {
                    match (*m68000).get_next_long(&mut *memory) {
                        Ok(data) => m68000_memory_result_t {
                            data,
                            exception: 0,
                        },
                        Err(vec) => m68000_memory_result_t {
                            data: 0,
                            exception: vec,
                        },
                    }
                }
            }

            /// Returns the 16-bits word at the current PC value of the given core.
            #[no_mangle]
            pub unsafe extern "C" fn [<m68000_ $cpu _fastmem_peek_next_word>](m68000: *const M68000<$cpu_details>, memory: *mut m68000_fastmem_t) -> m68000_memory_result_t {
                unsafe {
                    match (*m68000).peek_next_word(&mut *memory) {
                        Ok(data) => m68000_memory_result_t {
                            data: data as u32,
                            exception: 0,
                        },
                        Err(vec) => m68000_memory_result_t {
                            data: 0,
                            exception: vec,
                        },
                    }
                }
            }
        }
    };
}
