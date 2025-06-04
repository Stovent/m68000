# m68000 C interface

By enabling the `ffi` feature, the following structs and enums are made `repr(C)`:
- AddressingMode
- BriefExtensionWord
- Direction
- Instruction
- Operands
- Registers
- Size
- StatusRegister
- Vector

The crate `m68000-ffi` in the repo is a collection of structures and functions that allows using m68000's
interpreter and disassembler in other languages through a C interface.

## Build the C interface

First, build the two static libraries by building the project using the correct target toolchain.
To change the build toolchain, add `+<toolchain name>`.
```sh
cargo build --lib --release --features="ffi"
# or
cargo +nightly-x86_64-pc-windows-gnu build --lib --release --features="ffi"
```

Then, add the `include/` folder in your header directories search path, and include the given header files `m68000/m68000.h` and `m68000/m68000-ffi.h` in your project.

If you want to generate the header files from scratch, install [cargo-expand](https://github.com/dtolnay/cargo-expand) with the following command `cargo install cargo-expand`, then generate the header files using the `generate_headers.ps1` script on Windows or `generate_headers.sh` on Linux.

## Use the C interface

You need to link to the two generated libraries in your project:
- `libm68000` which contains the Rust core of the library.
- `libm68000_ffi` which contains the interface code between Rust and C.

The `m68000.h` header contains the m68000 library data structures.
The `m68000-ffi.h` header is the one to include in your C files and it contains the structs and functions declarations of the interface.

Because the main Rust library is generic over a certain CPU type trait, and this trait is only usable in Rust code, the C interface exports both of the pre-defined CPU types.

Each function name starts with the library name `m68000_`, then the CPU type `mc68000_` or `scc68070_` and finally the action performed by the function. Make sure you use the correct core with the correct functions.

The complete documentation for the functions and structures can be found in the `m68000-ffi/lib.rs` file.
See the C example below for a basic start.

Include the generated header file in your project, and define your memory access callback functions. These functions will be passed to the core through a m68000_callbacks_t struct.

The returned values are in a m68000_memory_result_t struct. Set `m68000_memory_result_t.exception` to 0 and set `m68000_memory_result_t.data` to the value to be returned on success. Set `m68000_memory_result_t.exception` to 2 (Access Error vector) if an Access Error occurs.

## C example

```c
#include "m68000-ffi.h"

#include <stdint.h>
#include <stdlib.h>

#define MEMSIZE (1 << 20) // 1 MB.

m68000_memory_result_t getByte(uint32_t addr, void* user_data)
{
    const uint8_t* memory = user_data;
    if(addr < MEMSIZE)
        return (m68000_memory_result_t) {
            .data = memory[addr],
            .exception = 0,
        };

    // If out of range, return an Access (bus) error.
    return (m68000_memory_result_t) {
        .data = 0,
        .exception = 2,
    };
}

m68000_memory_result_t getWord(uint32_t addr, void* user_data)
{
    const uint8_t* memory = user_data;
    if(addr < MEMSIZE)
        return (m68000_memory_result_t) {
            .data = (uint16_t)memory[addr] << 8
                | (uint16_t)memory[addr + 1],
            .exception = 0,
        };

    // If out of range, return an Access (bus) error.
    return (m68000_memory_result_t) {
        .data = 0,
        .exception = 2,
    };
}

m68000_memory_result_t getLong(uint32_t addr, void* user_data)
{
    const uint8_t* memory = user_data;
    if(addr < MEMSIZE)
        return (m68000_memory_result_t) {
            .data = (uint32_t)memory[addr] << 24
                | (uint32_t)memory[addr + 1] << 16
                | (uint32_t)memory[addr + 2] << 8
                | memory[addr + 3],
            .exception = 0,
        };

    // If out of range, return an Access (bus) error.
    return (m68000_memory_result_t) {
        .data = 0,
        .exception = 2,
    };
}

m68000_memory_result_t setByte(uint32_t addr, uint8_t data, void* user_data)
{
    uint8_t* memory = user_data;
    m68000_memory_result_t res = {
        .data = 0,
        .exception = 0,
    };

    if(addr < MEMSIZE)
        memory[addr] = data;
    else
        res.exception = 2;

    return res;
}

m68000_memory_result_t setWord(uint32_t addr, uint16_t data, void* user_data)
{
    uint8_t* memory = user_data;
    m68000_memory_result_t res = {
        .data = 0,
        .exception = 0,
    };

    if(addr < MEMSIZE)
    {
        memory[addr] = data >> 8;
        memory[addr + 1] = data;
    }
    else
        res.exception = 2;

    return res;
}

m68000_memory_result_t setLong(uint32_t addr, uint32_t data, void* user_data)
{
    uint8_t* memory = user_data;
    m68000_memory_result_t res = {
        .data = 0,
        .exception = 0,
    };

    if(addr < MEMSIZE)
    {
        memory[addr] = data >> 24;
        memory[addr + 1] = data >> 16;
        memory[addr + 2] = data >> 8;
        memory[addr + 3] = data;
    }
    else
        res.exception = 2;

    return res;
}

void reset(void* user_data) {}

int main()
{
    uint8_t* memory = malloc(MEMSIZE);
    // Check if malloc is successful, then load your program in memory here.
    // Next create the memory callback structure:
    m68000_callbacks_t callbacks = {
        .get_byte = getByte,
        .get_word = getWord,
        .get_long = getLong,
        .set_byte = setByte,
        .set_word = setWord,
        .set_long = setLong,
        .reset_instruction = reset,
        .user_data = memory,
    };

    m68000_mc68000_t* core = m68000_mc68000_new(); // Create a new core.
    // Now execute instructions as you want.
    m68000_mc68000_interpreter(core, &callbacks);

    // end of the program.
    m68000_mc68000_delete(core);
    free(memory);
    return 0;
}
```
