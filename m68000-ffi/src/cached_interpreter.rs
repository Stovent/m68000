// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Cached interpreter structs and exports.



#[macro_export]
macro_rules! cinterface_cached_interpreter {
    ($cpu:ident, $cpu_details:ty) => {
        paste! {
            /// Allocates a new core and returns the pointer to it.
            ///
            /// The created core has a [Reset vector](m68000::exception::Vector::ResetSspPc) pushed, so that the first call to an
            /// interpreter method will first fetch the reset vectors, then will execute the first instruction.
            ///
            /// It is not managed by Rust, so you have to delete it after usage with `m68000_*_delete`.
            #[no_mangle]
            pub extern "C" fn [<m68000_ $cpu _new_cached_interpreter>]() -> *mut M68000<$cpu_details> {
                Box::into_raw(Box::new(M68000::<$cpu_details>::new()))
            }

            /// `m68000_*_new` but without the initial reset vector, so you can initialize the core as you want.
            #[no_mangle]
            pub extern "C" fn [<m68000_ $cpu _new_no_reset_cached_interpreter>]() -> *mut M68000<$cpu_details> {
                Box::into_raw(Box::new(M68000::<$cpu_details>::new_no_reset()))
            }

            /// Frees the memory of the given core.
            #[no_mangle]
            pub unsafe extern "C" fn [<m68000_ $cpu _delete_cached_interpreter>](m68000: *mut M68000<$cpu_details>) {
                unsafe {
                    std::mem::drop(Box::from_raw(m68000));
                }
            }
        }
    };
}
