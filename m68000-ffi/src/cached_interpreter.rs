// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Cached interpreter structs and exports.

use m68000::cached_interpreter_flat_block::CachedInterpreterFlatBlock;
use m68000::cached_interpreter_single_block::CachedInterpreterSingleBlock;
use m68000::cached_interpreter_trie_block::CachedInterpreterTrieBlock;

/// Allocates a new flat block cached interpreter.
#[no_mangle]
pub extern "C" fn m68000_new_cached_interpreter_flat_block(begin: u32, end: u32) -> *mut CachedInterpreterFlatBlock {
    Box::into_raw(Box::new(CachedInterpreterFlatBlock::new(begin, end)))
}

/// Frees a flat block cached interpreter.
#[no_mangle]
pub unsafe extern "C" fn m68000_delete_cached_interpreter_flat_block(ptr: *mut CachedInterpreterFlatBlock) {
    std::mem::drop(Box::from_raw(ptr));
}

/// Allocates a new single block cached interpreter.
#[no_mangle]
pub extern "C" fn m68000_new_cached_interpreter_single_block(begin: u32, end: u32) -> *mut CachedInterpreterSingleBlock {
    Box::into_raw(Box::new(CachedInterpreterSingleBlock::new(begin, end)))
}

/// Frees a single block cached interpreter.
#[no_mangle]
pub unsafe extern "C" fn m68000_delete_cached_interpreter_single_block(ptr: *mut CachedInterpreterSingleBlock) {
    std::mem::drop(Box::from_raw(ptr));
}

/// Allocates a new trie block cached interpreter.
#[no_mangle]
pub extern "C" fn m68000_new_cached_interpreter_trie_block() -> *mut CachedInterpreterTrieBlock {
    Box::into_raw(Box::new(CachedInterpreterTrieBlock::new()))
}

/// Frees a trie block cached interpreter.
#[no_mangle]
pub unsafe extern "C" fn m68000_delete_cached_interpreter_trie_block(ptr: *mut CachedInterpreterTrieBlock) {
    std::mem::drop(Box::from_raw(ptr));
}

#[macro_export]
macro_rules! cinterface_cached_interpreter {
    ($cpu:ident, $cpu_details:ty, $cached_name:ident, $cached_type:ty) => {
        paste! {
            /// Executes the next instruction or block of instructions, returning the cycle count necessary to execute it.
            #[no_mangle]
            pub unsafe extern "C" fn [<m68000_ $cpu _ $cached_name _interpreter>](interpreter: *mut $cached_type, m68000: *mut M68000<$cpu_details>, memory: *mut m68000_callbacks_t) -> usize {
                unsafe {
                    (*interpreter).cached_interpreter(&mut *m68000, &mut *memory)
                }
            }

            /// Executes the next instruction or block of instructions, returning the cycle count necessary to execute it,
            /// and the vector of the exception that occured during the execution if any.
            ///
            /// To process the returned exception, call `m68000_*_exception`.
            #[no_mangle]
            pub unsafe extern "C" fn [<m68000_ $cpu _ $cached_name _interpreter_exception>](interpreter: *mut $cached_type, m68000: *mut M68000<$cpu_details>, memory: *mut m68000_callbacks_t) -> m68000_exception_result_t {
                unsafe {
                    let (cycles, vector) = (*interpreter).cached_interpreter_exception(&mut *m68000, &mut *memory);
                    m68000_exception_result_t { cycles, exception: vector.unwrap_or(0) }
                }
            }

            /// Executes the next instruction or block of instructions, returning the cycle count necessary to execute it.
            #[no_mangle]
            pub unsafe extern "C" fn [<m68000_ $cpu _fastmem_ $cached_name _interpreter>](interpreter: *mut $cached_type, m68000: *mut M68000<$cpu_details>, memory: *mut m68000_fastmem_t) -> usize {
                unsafe {
                    (*interpreter).cached_interpreter(&mut *m68000, &mut *memory)
                }
            }

            /// Executes the next instruction or block of instructions, returning the cycle count necessary to execute it,
            /// and the vector of the exception that occured during the execution if any.
            ///
            /// To process the returned exception, call `m68000_*_exception`.
            #[no_mangle]
            pub unsafe extern "C" fn [<m68000_ $cpu _fastmem_ $cached_name _interpreter_exception>](interpreter: *mut $cached_type, m68000: *mut M68000<$cpu_details>, memory: *mut m68000_fastmem_t) -> m68000_exception_result_t {
                unsafe {
                    let (cycles, vector) = (*interpreter).cached_interpreter_exception(&mut *m68000, &mut *memory);
                    m68000_exception_result_t { cycles, exception: vector.unwrap_or(0) }
                }
            }
        }
    };
}
