// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The C interface of m68000, to use it in other languages.
//!
//! The functions and structures defined here should not be used in a rust program.
//!
//! To use it, first allocate a new core with `m68000_*_new` or `m68000_*_new_no_reset`. When done, delete it with `m68000_*_delete`.
//!
//! ## Memory callback
//!
//! You need to provide the memory access structure to the core when executing instructions.
//! Create a new [m68000_callbacks_t] structure, and assign the correct function callback as its members.
//!
//! Each callback returns a [m68000_memory_result_t], which indicates if the memory access is successful or not.
//! If successful, set the `exception` member to 0 and set the `data` member to the value to be returned if read. it is not used on write.
//! If the address is out of range, set `exception` to 2 (Access Error).
//!
//! ## Interpreter functions
//!
//! There are several functions to execute instructions, see their individual documentation for more information:
//! - `m68000_*_interpreter` is the basic one. It tries to execute the next instruction, and returns the number of cycles the instruction tool to be executed.
//! if an exception occurs, it is added to the pending exceptions and will be processed on the next call to an interpreter function.
//! - `m68000_*_interpreter_exception` is like above, but if an exception occurs, it is returned instead of being processed.
//! To process the returned exception, call `m68000_*_exception` with the vector returned.
//! - `m68000_*_cycle` which runs the CPU for **at least** the given number of cycles.
//! - `m68000_*_cycle_until_exception` which runs the CPU until either an exception occurs or **at least** the given number of cycles have been executed.
//! - `m68000_*_loop_until_exception_stop` which runs the CPU indefinitely, until an exception or a STOP instruction occurs.
//! - `m68000_*_disassembler_interpreter` which behaves like `m68000_*_interpreter` and returns the disassembled string of the instruction executed.
//! - `m68000_*_disassembler_interpreter_exception` which behaves like `m68000_*_interpreter_exception` and returns the disassembled string of the instruction executed.
//!
//! ## Exceptions processing
//!
//! To request the core to process an exception, call `m68000_*_exception` with the vector number of the exception to process.
//!
//! ## Accessing the registers
//!
//! There are 3 functions to read and write to the core's registers:
//! - `m68000_*_registers` returns a mutable (non-const) pointer to the [Registers](crate::Registers).
//! The location of the registers does not change during execution, so you can store the pointer for as long as the core lives.
//! - `m68000_*_get_registers` returns a copy of the registers. Writing to it does not modify the core's registers.
//! - `m68000_*_set_registers` sets the core's registers to the value of the given [Registers] structure.
//!
//! ## C example
//!
//! The code below is a minimalist example showing a single function callback. See the README.md file for a complete example.
//!
//! ```c
//! #include "m68000.h"
//!
//! #include <stdint.h>
//! #include <stdlib.h>
//!
//! #define MEMSIZE (1 << 20) // 1 MB.
//!
//! m68000_memory_result_t getByte(uint32_t addr, void* user_data)
//! {
//!     const uint8_t* memory = user_data;
//!     if(addr < MEMSIZE)
//!         return (m68000_memory_result_t) {
//!             .data = memory[addr],
//!             .exception = 0,
//!         };
//!
//!     // If out of range, return an Access (bus) error.
//!     return (m68000_memory_result_t) {
//!         .data = 0,
//!         .exception = 2,
//!     };
//! }
//!
//! // Implement the rest of the callback function.
//!
//! int main()
//! {
//!     uint8_t* memory = malloc(MEMSIZE);
//!     // Check if malloc is successful, then load your program in memory here.
//!     // Next create the memory callback structure:
//!     m68000_callbacks_t callbacks = {
//!         .get_byte = getByte,
//!         // Assign the rest of the members.
//!         .user_data = memory,
//!     };
//!
//!     M68000* core = m68000_new(); // Create a new core.
//!
//!     // Now execute instructions as you want.
//!     m68000_interpreter(core, &callbacks);
//!
//!     // end of the program.
//!     m68000_delete(core);
//!     free(memory);
//!     return 0;
//! }
//! ```

pub mod mc68000;
pub mod scc68070;

use m68000::memory_access::MemoryAccess;

use std::ffi::{c_void, CString};
use std::os::raw::c_char;

/// Return value of the `m68000_*_cycle_until_exception`, `m68000_*_loop_until_exception_stop` and
/// `m68000_*_interpreter_exception` functions.
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct m68000_exception_result_t {
    /// The number of cycles executed.
    pub cycles: usize,
    /// 0 if no exception occured, the vector number that occured otherwise.
    pub exception: u8,
}

/// Return type of the memory callback functions.
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct m68000_memory_result_t {
    /// Set to the value to be returned. Only the low order bytes are read depending on the size. Unused with SetResult.
    pub data: u32,
    /// Set to 0 if read successfully, set to 2 (Access Error) otherwise (Address errors are automatically detected by the library).
    ///
    /// If used as the return value of `m68000_*_peek_next_word`, this field contains the exception vector that occured when trying to read the next word.
    pub exception: u8,
}

/// Memory callbacks sent to the interpreter methods.
///
/// The void* argument passed on each callback is the [user_data](Self::user_data) member,
/// and its usage is let to the user of this library. For example, this can be used to allow the usage of C++ objects,
/// where [user_data](Self::user_data) has the value of the `this` pointer of the object.
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct m68000_callbacks_t {
    pub get_byte: extern "C" fn(addr: u32, user_data: *mut c_void) -> m68000_memory_result_t,
    pub get_word: extern "C" fn(addr: u32, user_data: *mut c_void) -> m68000_memory_result_t,
    pub get_long: extern "C" fn(addr: u32, user_data: *mut c_void) -> m68000_memory_result_t,

    pub set_byte: extern "C" fn(addr: u32, data: u8, user_data: *mut c_void) -> m68000_memory_result_t,
    pub set_word: extern "C" fn(addr: u32, data: u16, user_data: *mut c_void) -> m68000_memory_result_t,
    pub set_long: extern "C" fn(addr: u32, data: u32, user_data: *mut c_void) -> m68000_memory_result_t,

    pub reset_instruction: extern "C" fn(user_data: *mut c_void),

    pub user_data: *mut c_void,
}

impl MemoryAccess for m68000_callbacks_t {
    fn get_byte(&mut self, addr: u32) -> Option<u8> {
        let res = (self.get_byte)(addr, self.user_data);
        if res.exception == 0 {
            Some(res.data as u8)
        } else {
            None
        }
    }

    fn get_word(&mut self, addr: u32) -> Option<u16> {
        let res = (self.get_word)(addr, self.user_data);
        if res.exception == 0 {
            Some(res.data as u16)
        } else {
            None
        }

    }

    fn get_long(&mut self, addr: u32) -> Option<u32> {
        let res = (self.get_long)(addr, self.user_data);
        if res.exception == 0 {
            Some(res.data)
        } else {
            None
        }
    }

    fn set_byte(&mut self, addr: u32, value: u8) -> Option<()> {
        let res = (self.set_byte)(addr, value, self.user_data);
        if res.exception == 0 {
            Some(())
        } else {
            None
        }
    }

    fn set_word(&mut self, addr: u32, value: u16) -> Option<()> {
        let res = (self.set_word)(addr, value, self.user_data);
        if res.exception == 0 {
            Some(())
        } else {
            None
        }
    }

    fn set_long(&mut self, addr: u32, value: u32) -> Option<()> {
        let res = (self.set_long)(addr, value, self.user_data);
        if res.exception == 0 {
            Some(())
        } else {
            None
        }
    }

    fn reset_instruction(&mut self) {
        (self.reset_instruction)(self.user_data)
    }
}

macro_rules! cinterface {
    ($cpu:ident, $cpu_details:ty) => {
        paste! {
            /// Allocates a new core and returns the pointer to it.
            ///
            /// The created core has a [Reset vector](m68000::exception::Vector::ResetSspPc) pushed, so that the first call to an
            /// interpreter method will first fetch the reset vectors, then will execute the first instruction.
            ///
            /// It is not managed by Rust, so you have to delete it after usage with `m68000_*_delete`.
            #[no_mangle]
            pub extern "C" fn [<m68000_ $cpu _new>]() -> *mut M68000<$cpu_details> {
                Box::into_raw(Box::new(M68000::<$cpu_details>::new()))
            }

            /// `m68000_*_new` but without the initial reset vector, so you can initialize the core as you want.
            #[no_mangle]
            pub extern "C" fn [<m68000_ $cpu _new_no_reset>]() -> *mut M68000<$cpu_details> {
                Box::into_raw(Box::new(M68000::<$cpu_details>::new_no_reset()))
            }

            /// Frees the memory of the given core.
            #[no_mangle]
            pub extern "C" fn [<m68000_ $cpu _delete>](m68000: *mut M68000<$cpu_details>) {
                unsafe {
                    std::mem::drop(Box::from_raw(m68000));
                }
            }

            /// Runs the CPU for `cycles` number of cycles.
            ///
            /// This function executes **at least** the given number of cycles.
            /// Returns the number of cycles actually executed.
            ///
            /// If you ask to execute 4 cycles but the next instruction takes 6 cycles to execute,
            /// it will be executed and the 2 extra cycles will be subtracted in the next call.
            #[no_mangle]
            pub extern "C" fn [<m68000_ $cpu _cycle>](m68000: *mut M68000<$cpu_details>, memory: *mut m68000_callbacks_t, cycles: usize) -> usize {
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
            pub extern "C" fn [<m68000_ $cpu _cycle_until_exception>](m68000: *mut M68000<$cpu_details>, memory: *mut m68000_callbacks_t, cycles: usize) -> m68000_exception_result_t {
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
            pub extern "C" fn [<m68000_ $cpu _loop_until_exception_stop>](m68000: *mut M68000<$cpu_details>, memory: *mut m68000_callbacks_t) -> m68000_exception_result_t {
                unsafe {
                    let (cycles, vector) = (*m68000).loop_until_exception_stop(&mut *memory);
                    m68000_exception_result_t { cycles, exception: vector.unwrap_or(0) }
                }
            }

            /// Executes the next instruction, returning the cycle count necessary to execute it.
            #[no_mangle]
            pub extern "C" fn [<m68000_ $cpu _interpreter>](m68000: *mut M68000<$cpu_details>, memory: *mut m68000_callbacks_t) -> usize {
                unsafe {
                    (*m68000).interpreter(&mut *memory)
                }
            }

            /// Executes the next instruction, returning the cycle count necessary to execute it,
            /// and the vector of the exception that occured during the execution if any.
            ///
            /// To process the returned exception, call `m68000_*_exception`.
            #[no_mangle]
            pub extern "C" fn [<m68000_ $cpu _interpreter_exception>](m68000: *mut M68000<$cpu_details>, memory: *mut m68000_callbacks_t) -> m68000_exception_result_t {
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
            pub extern "C" fn [<m68000_ $cpu _disassembler_interpreter>](m68000: *mut M68000<$cpu_details>, memory: *mut m68000_callbacks_t, str: *mut c_char, len: usize) -> usize {
                unsafe {
                    let (dis, cycles) = (*m68000).disassembler_interpreter(&mut *memory);

                    let cstring = CString::new(dis).expect("New CString for disassembler failed")
                        .into_bytes_with_nul();
                    let raw_cstring = std::mem::transmute::<*const u8, *const c_char>(cstring.as_ptr());

                    if cstring.len() <= len {
                        str.copy_from_nonoverlapping(raw_cstring, cstring.len());
                    } else {
                        str.copy_from_nonoverlapping(raw_cstring, len - 1);
                        *str.add(len - 1) = 0;
                    }

                    cycles
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
            pub extern "C" fn [<m68000_ $cpu _disassembler_interpreter_exception>](m68000: *mut M68000<$cpu_details>, memory: *mut m68000_callbacks_t, str: *mut c_char, len: usize) -> m68000_exception_result_t {
                unsafe {
                    let (dis, cycles, vector) = (*m68000).disassembler_interpreter_exception(&mut *memory);

                    let cstring = CString::new(dis).expect("New CString for disassembler failed")
                        .into_bytes_with_nul();
                    let raw_cstring = std::mem::transmute::<*const u8, *const c_char>(cstring.as_ptr());

                    if cstring.len() <= len {
                        str.copy_from_nonoverlapping(raw_cstring, cstring.len());
                    } else {
                        str.copy_from_nonoverlapping(raw_cstring, len - 1);
                        *str.add(len - 1) = 0;
                    }

                    m68000_exception_result_t { cycles, exception: vector.unwrap_or(0) }
                }
            }

            /// Requests the CPU to process the given exception vector.
            #[no_mangle]
            pub extern "C" fn [<m68000_ $cpu _exception>](m68000: *mut M68000<$cpu_details>, vector: Vector) {
                unsafe {
                    (*m68000).exception(Exception::from(vector))
                }
            }

            /// Returns the 16-bits word at the current PC value of the given core.
            #[no_mangle]
            pub extern "C" fn [<m68000_ $cpu _peek_next_word>](m68000: *mut M68000<$cpu_details>, memory: *mut m68000_callbacks_t) -> m68000_memory_result_t {
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

            /// Returns a mutable pointer to the registers of the given core.
            #[no_mangle]
            pub extern "C" fn [<m68000_ $cpu _registers>](m68000: *mut M68000<$cpu_details>) -> *mut Registers {
                unsafe {
                    &mut (*m68000).regs
                }
            }

            /// Returns a copy of the registers of the given core.
            #[no_mangle]
            pub extern "C" fn [<m68000_ $cpu _get_registers>](m68000: *const M68000<$cpu_details>) -> Registers {
                unsafe {
                    (*m68000).regs
                }
            }

            /// Sets the registers of the core to the given value.
            #[no_mangle]
            pub extern "C" fn [<m68000_ $cpu _set_registers>](m68000: *mut M68000<$cpu_details>, regs: Registers) {
                unsafe {
                    (*m68000).regs = regs;
                }
            }
        }
    };
}

use cinterface;
