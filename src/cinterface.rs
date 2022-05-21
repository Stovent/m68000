use crate::{M68000, Registers};
use crate::memory_access::{MemoryAccess, GetResult, SetResult};

use std::ffi::CString;
use std::os::raw::c_char;

/// Return value of the `cycle_until_exception`, `loop_until_exception_stop` and `interpreter_exception` methods.
#[repr(C)]
pub struct ExceptionResult {
    /// The number of cycles executed.
    pub cycles: usize,
    /// 0 if no exception occured, the vector number that occured otherwise.
    pub exception: u8,
}

#[repr(C)]
pub struct GetSetResult {
    /// Set to the value to be returned. Only the low order bytes are read depending on the size. Unused with SetResult.
    pub data: u32,
    /// Set to 0 if read successfully, set to the exception vector otherwise (Access or Address error).
    pub exception: u8,
}

#[repr(C)]
pub struct M68000Callbacks {
    pub get_byte: extern "C" fn(u32) -> GetSetResult,
    pub get_word: extern "C" fn(u32) -> GetSetResult,
    pub get_long: extern "C" fn(u32) -> GetSetResult,

    pub set_byte: extern "C" fn(u32, u8) -> GetSetResult,
    pub set_word: extern "C" fn(u32, u16) -> GetSetResult,
    pub set_long: extern "C" fn(u32, u32) -> GetSetResult,

    pub reset_instruction: extern "C" fn(),
    pub disassembler: extern "C" fn(u32, *const c_char),
}

impl MemoryAccess for M68000Callbacks {
    fn get_byte(&mut self, addr: u32) -> GetResult<u8> {
        let res = (self.get_byte)(addr);
        if res.exception == 0 {
            Ok(res.data as u8)
        } else {
            Err(res.exception)
        }
    }

    fn get_word(&mut self, addr: u32) -> GetResult<u16> {
        let res = (self.get_word)(addr);
        if res.exception == 0 {
            Ok(res.data as u16)
        } else {
            Err(res.exception)
        }

    }
    fn get_long(&mut self, addr: u32) -> GetResult<u32> {
        let res = (self.get_long)(addr);
        if res.exception == 0 {
            Ok(res.data)
        } else {
            Err(res.exception)
        }
    }

    fn set_byte(&mut self, addr: u32, value: u8) -> SetResult {
        let res = (self.set_byte)(addr, value);
        if res.exception == 0 {
            Ok(())
        } else {
            Err(res.exception)
        }
    }

    fn set_word(&mut self, addr: u32, value: u16) -> SetResult {
        let res = (self.set_word)(addr, value);
        if res.exception == 0 {
            Ok(())
        } else {
            Err(res.exception)
        }
    }

    fn set_long(&mut self, addr: u32, value: u32) -> SetResult {
        let res = (self.set_long)(addr, value);
        if res.exception == 0 {
            Ok(())
        } else {
            Err(res.exception)
        }
    }

    fn reset_instruction(&mut self) {
        (self.reset_instruction)()
    }

    fn disassembler(&mut self, pc: u32, inst_string: String) {
        let cs = CString::new(inst_string).expect("New CString for disassembler failed");
        (self.disassembler)(pc, cs.as_ptr());
    }
}

#[no_mangle]
pub extern "C" fn m68000_new() -> *mut M68000 {
    Box::into_raw(Box::new(M68000::new()))
}

#[no_mangle]
pub extern "C" fn m68000_delete(m68000: *mut M68000) {
    unsafe {
        Box::from_raw(m68000);
    }
}

#[no_mangle]
pub extern "C" fn m68000_cycle(m68000: *mut M68000, memory: *mut M68000Callbacks, cycles: usize) -> usize {
    unsafe {
        (*m68000).cycle(&mut *memory, cycles)
    }
}

#[no_mangle]
pub extern "C" fn m68000_cycle_until_exception(m68000: *mut M68000, memory: *mut M68000Callbacks, cycles: usize) -> ExceptionResult {
    unsafe {
        let (cycles, vector) = (*m68000).cycle_until_exception(&mut *memory, cycles);
        ExceptionResult { cycles, exception: vector.unwrap_or(0) }
    }
}

#[no_mangle]
pub extern "C" fn m68000_loop_until_exception_stop(m68000: *mut M68000, memory: *mut M68000Callbacks) -> ExceptionResult {
    unsafe {
        let (cycles, vector) = (*m68000).loop_until_exception_stop(&mut *memory);
        ExceptionResult { cycles, exception: vector.unwrap_or(0) }
    }
}

#[no_mangle]
pub extern "C" fn m68000_interpreter(m68000: *mut M68000, memory: *mut M68000Callbacks) -> usize {
    unsafe {
        (*m68000).interpreter(&mut *memory)
    }
}

#[no_mangle]
pub extern "C" fn m68000_interpreter_exception(m68000: *mut M68000, memory: *mut M68000Callbacks) -> ExceptionResult {
    unsafe {
        let (cycles, vector) = (*m68000).interpreter_exception(&mut *memory);
        ExceptionResult { cycles, exception: vector.unwrap_or(0) }
    }
}

#[no_mangle]
pub extern "C" fn m68000_exception(m68000: *mut M68000, vector: u8) {
    unsafe {
        (*m68000).exception(vector)
    }
}

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

#[no_mangle]
pub extern "C" fn m68000_get_registers(m68000: *const M68000) -> Registers {
    unsafe {
        (*m68000).regs
    }
}

#[no_mangle]
pub extern "C" fn m68000_set_registers(m68000: *mut M68000, regs: Registers) {
    unsafe {
        (*m68000).regs = regs;
    }
}

#[no_mangle]
pub extern "C" fn m68000_enable_disassembler(m68000: *mut M68000, enabled: bool) {
    unsafe {
        (*m68000).disassemble = enabled;
    }
}
