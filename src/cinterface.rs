use crate::{M68000, Registers};
use crate::memory_access::{MemoryAccess, GetResult, SetResult};

use std::ffi::{c_void, CString};
use std::os::raw::c_char;

/// Return value of the `cycle_until_exception`, `loop_until_exception_stop` and `interpreter_exception` functions.
#[repr(C)]
pub struct ExceptionResult {
    /// The number of cycles executed.
    pub cycles: usize,
    /// 0 if no exception occured, the vector number that occured otherwise.
    pub exception: u8,
}

/// Return type of the memory callback functions.
#[repr(C)]
pub struct GetSetResult {
    /// Set to the value to be returned. Only the low order bytes are read depending on the size. Unused with SetResult.
    pub data: u32,
    /// Set to 0 if read successfully, set to 2 (Access Error) otherwise (Address errors are automatically detected by the library).
    pub exception: u8,
}

/// Memory callbacks sent to the interpreter methods.
///
/// The void* argument passed on each callback is the `user_data` member, and its usage is let to the user of this library.
/// For example, this can be used to allow the usage of C++ objects, where `user_data` has the value of the `this` pointer of the object.
#[repr(C)]
pub struct M68000Callbacks {
    pub get_byte: extern "C" fn(u32, *mut c_void) -> GetSetResult,
    pub get_word: extern "C" fn(u32, *mut c_void) -> GetSetResult,
    pub get_long: extern "C" fn(u32, *mut c_void) -> GetSetResult,

    pub set_byte: extern "C" fn(u32, u8, *mut c_void) -> GetSetResult,
    pub set_word: extern "C" fn(u32, u16, *mut c_void) -> GetSetResult,
    pub set_long: extern "C" fn(u32, u32, *mut c_void) -> GetSetResult,

    pub reset_instruction: extern "C" fn(*mut c_void),

    pub user_data: *mut c_void,
}

impl MemoryAccess for M68000Callbacks {
    fn get_byte(&mut self, addr: u32) -> GetResult<u8> {
        let res = (self.get_byte)(addr, self.user_data);
        if res.exception == 0 {
            Ok(res.data as u8)
        } else {
            Err(res.exception)
        }
    }

    fn get_word(&mut self, addr: u32) -> GetResult<u16> {
        let res = (self.get_word)(addr, self.user_data);
        if res.exception == 0 {
            Ok(res.data as u16)
        } else {
            Err(res.exception)
        }

    }

    fn get_long(&mut self, addr: u32) -> GetResult<u32> {
        let res = (self.get_long)(addr, self.user_data);
        if res.exception == 0 {
            Ok(res.data)
        } else {
            Err(res.exception)
        }
    }

    fn set_byte(&mut self, addr: u32, value: u8) -> SetResult {
        let res = (self.set_byte)(addr, value, self.user_data);
        if res.exception == 0 {
            Ok(())
        } else {
            Err(res.exception)
        }
    }

    fn set_word(&mut self, addr: u32, value: u16) -> SetResult {
        let res = (self.set_word)(addr, value, self.user_data);
        if res.exception == 0 {
            Ok(())
        } else {
            Err(res.exception)
        }
    }

    fn set_long(&mut self, addr: u32, value: u32) -> SetResult {
        let res = (self.set_long)(addr, value, self.user_data);
        if res.exception == 0 {
            Ok(())
        } else {
            Err(res.exception)
        }
    }

    fn reset_instruction(&mut self) {
        (self.reset_instruction)(self.user_data)
    }
}

/// Allocates a new core and returns the pointer to it. Is is unmanaged by Rust, so you have to delete it after usage.
#[no_mangle]
pub extern "C" fn m68000_new() -> *mut M68000 {
    Box::into_raw(Box::new(M68000::new()))
}

/// Frees the memory of the given core.
#[no_mangle]
pub extern "C" fn m68000_delete(m68000: *mut M68000) {
    unsafe {
        Box::from_raw(m68000);
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
pub extern "C" fn m68000_cycle(m68000: *mut M68000, memory: *mut M68000Callbacks, cycles: usize) -> usize {
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
pub extern "C" fn m68000_cycle_until_exception(m68000: *mut M68000, memory: *mut M68000Callbacks, cycles: usize) -> ExceptionResult {
    unsafe {
        let (cycles, vector) = (*m68000).cycle_until_exception(&mut *memory, cycles);
        ExceptionResult { cycles, exception: vector.unwrap_or(0) }
    }
}

/// Runs indefinitely until an exception or STOP instruction occurs.
///
/// Returns the number of cycles executed and the exception that occured.
/// If exception is None, this means the CPU has executed a STOP instruction.
#[no_mangle]
pub extern "C" fn m68000_loop_until_exception_stop(m68000: *mut M68000, memory: *mut M68000Callbacks) -> ExceptionResult {
    unsafe {
        let (cycles, vector) = (*m68000).loop_until_exception_stop(&mut *memory);
        ExceptionResult { cycles, exception: vector.unwrap_or(0) }
    }
}

/// Executes the next instruction, returning the cycle count necessary to execute it.
#[no_mangle]
pub extern "C" fn m68000_interpreter(m68000: *mut M68000, memory: *mut M68000Callbacks) -> usize {
    unsafe {
        (*m68000).interpreter(&mut *memory)
    }
}

/// Executes the next instruction, returning the cycle count necessary to execute it,
/// and the vector of the exception that occured during the execution if any.
///
/// To process the returned exception, call [M68000::exception].
#[no_mangle]
pub extern "C" fn m68000_interpreter_exception(m68000: *mut M68000, memory: *mut M68000Callbacks) -> ExceptionResult {
    unsafe {
        let (cycles, vector) = (*m68000).interpreter_exception(&mut *memory);
        ExceptionResult { cycles, exception: vector.unwrap_or(0) }
    }
}

/// Executes and disassembles the next instruction, returning the disassembler string and the cycle count necessary to execute it.
///
/// `str` is a pointer to a C string buffer where the disassembled instruction will be written.
/// `len` is the maximum size of the buffer.
#[no_mangle]
pub extern "C" fn m68000_disassembler_interpreter(m68000: *mut M68000, memory: *mut M68000Callbacks, str: *mut c_char, len: usize) -> usize {
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
/// To process the returned exception, call [M68000::exception].
///
/// `str` is a pointer to a C string buffer where the disassembled instruction will be written.
/// `len` is the maximum size of the buffer.
#[no_mangle]
pub extern "C" fn m68000_disassembler_interpreter_exception(m68000: *mut M68000, memory: *mut M68000Callbacks, str: *mut c_char, len: usize) -> ExceptionResult {
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

        ExceptionResult { cycles, exception: vector.unwrap_or(0) }
    }
}

/// Requests the CPU to process the given exception vector.
#[no_mangle]
pub extern "C" fn m68000_exception(m68000: *mut M68000, vector: u8) {
    unsafe {
        (*m68000).exception(vector)
    }
}

/// Returns the 16-bits word at the current PC value of the given core.
#[no_mangle]
pub extern "C" fn m68000_peek_next_word(m68000: *mut M68000, memory: *mut M68000Callbacks) -> GetSetResult {
    unsafe {
        match (*m68000).peek_next_word(&mut *memory) {
            Ok(data) => GetSetResult {
                data: data as u32,
                exception: 0,
            },
            Err(vec) => GetSetResult {
                data: 0,
                exception: vec,
            },
        }
    }
}

/// Returns a mutable pointer to the registers of the given core.
#[no_mangle]
pub extern "C" fn m68000_registers(m68000: *mut M68000) -> *mut Registers {
    unsafe {
        &mut (*m68000).regs
    }
}

/// Returns a copy of the registers of the given core.
#[no_mangle]
pub extern "C" fn m68000_get_registers(m68000: *const M68000) -> Registers {
    unsafe {
        (*m68000).regs
    }
}

/// Sets the registers of the core to the given value.
#[no_mangle]
pub extern "C" fn m68000_set_registers(m68000: *mut M68000, regs: Registers) {
    unsafe {
        (*m68000).regs = regs;
    }
}
