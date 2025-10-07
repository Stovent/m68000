//! FFI fastmem unit tests.

use core::ffi::c_void;
use core::pin::Pin;

use m68000::MemoryAccess;
use m68000_ffi::{m68000_callbacks_t, m68000_memory_result_t};
use m68000_ffi::fastmem::m68000_fastmem_t;

extern "C" fn get_byte(addr: u32, _user_data: *mut c_void) -> m68000_memory_result_t {
    match addr {
        0x8000_0000 => m68000_memory_result_t { data: 0x11, exception: 0 },
        _ => m68000_memory_result_t { data: 0, exception: 2 },
    }
}

extern "C" fn get_word(addr: u32, _user_data: *mut c_void) -> m68000_memory_result_t {
    match addr {
        0x8000_0000 => m68000_memory_result_t { data: 0x22, exception: 0 },
        _ => m68000_memory_result_t { data: 0, exception: 2 },
    }
}

extern "C" fn get_long(addr: u32, _user_data: *mut c_void) -> m68000_memory_result_t {
    match addr {
        0x8000_0000 => m68000_memory_result_t { data: 0x33, exception: 0 },
        _ => m68000_memory_result_t { data: 0, exception: 2 },
    }
}

extern "C" fn set_byte(_addr: u32, _data: u8, _user_data: *mut c_void) -> bool {
    true
}

extern "C" fn set_word(_addr: u32, _data: u16, _user_data: *mut c_void) -> bool {
    true
}

extern "C" fn set_long(_addr: u32, _data: u32, _user_data: *mut c_void) -> bool {
    false
}

extern "C" fn reset_instruction(_user_data: *mut c_void) {}

#[test]
fn fastmem() {
    let (mut fastmem, memory) = make_fastmem();

    // Test slow memory read.
    assert_eq!(fastmem.get_byte(0x8000_0000), Some(0x11));
    assert_eq!(fastmem.get_word(0x8000_0000), Some(0x22));
    assert_eq!(fastmem.get_long(0x8000_0000), Some(0x33));

    // Test slow memory write.
    assert_eq!(fastmem.set_byte(0x8000_0000, 0), Some(()));
    assert_eq!(fastmem.set_word(0x8000_0000, 0), Some(()));
    assert_eq!(fastmem.set_long(0x8000_0000, 0), None);

    // Test fastmem read.
    const SIZE: u32 = RAM_BANK_SIZE as u32;
    assert_eq!(fastmem.get_byte(SIZE - 2), Some(0xAA));
    assert_eq!(fastmem.get_word(SIZE - 2), Some(0xAA55));
    assert_eq!(fastmem.get_long(SIZE - 2), Some(0xAA5555AA));

    // Test fastmem write.
    assert_eq!(fastmem.set_byte(SIZE, 0), Some(())); // Goes in slowmem path.
    assert_eq!(fastmem.set_word(SIZE - 2, 0x500A), Some(()));
    assert_eq!(memory.ram1[RAM_BANK_SIZE - 2], 0x50);
    assert_eq!(memory.ram1[RAM_BANK_SIZE - 1], 0x0A);
    assert_eq!(fastmem.set_long(SIZE - 2, 0x500A), None);
}

struct Memory {
    ram1: Pin<Box<[u8]>>,
    _ram2: Pin<Box<[u8]>>,
    page_read: Pin<Box<[*const u8]>>,
    page_write: Pin<Box<[*mut u8]>>,
}

const OFFSET_BITS: u32 = 20;
const RAM_BANK_SIZE: usize = 1 << OFFSET_BITS;
const OFFSET_MASK: u32 = RAM_BANK_SIZE as u32 - 1;
const PAGE_BITS: u32 = 32 - OFFSET_BITS;
const PAGE_COUNT: usize = 1 << PAGE_BITS;

fn make_fastmem() -> (m68000_fastmem_t, Pin<Box<Memory>>) {
    let mut ram1 = Pin::new(vec![0; RAM_BANK_SIZE].into_boxed_slice());
    let mut ram2 = Pin::new(vec![0; RAM_BANK_SIZE].into_boxed_slice());
    // Setup cross page boundaries read.
    ram1[RAM_BANK_SIZE - 2] = 0xAA;
    ram1[RAM_BANK_SIZE - 1] = 0x55;
    ram2[0] = 0x55;
    ram2[1] = 0xAA;

    const RAM1_PAGE: usize = 0;
    const RAM2_PAGE: usize = RAM_BANK_SIZE >> OFFSET_BITS;

    let page_read = Pin::new({
        let mut page = vec![core::ptr::null(); PAGE_COUNT].into_boxed_slice();
        page[RAM1_PAGE] = ram1.as_ptr();
        page[RAM2_PAGE] = ram2.as_ptr();
        page
    });

    let page_write = Pin::new({
        let mut page = vec![core::ptr::null_mut(); PAGE_COUNT].into_boxed_slice();
        page[RAM1_PAGE] = ram1.as_mut_ptr();
        page
    });

    let mut memory = Box::pin(Memory {
        ram1,
        _ram2: ram2,
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
