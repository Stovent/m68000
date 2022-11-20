# m68000

m68000 is a Motorola 68000 interpreter, disassembler and assembler (code emitter) written in Rust.

This library emulates the common user and supervisor instructions of the M68k ISA. It is configurable to behave like the given CPU type (see below), changing the instruction's execution times and exception handling.

This library has been designed to be used in two different contexts:

- It can be used to emulate a whole CPU, and the user of this library only have to call the interpreter methods and exception when an interrupt or reset occurs. This is the typical use case for an emulator.
- It can also be used as a M68k user-mode interpreter to run a M68k program, but without the requirement of having an operating system compiled to binary M68k. In this case, the application runs the program until an exception occurs (TRAP for syscalls, zero divide, etc.), which can be handled in Rust code (or any other language using the C interface), so the application can implement the surrounding environment required by the M68k program in a high level language and not in M68k assembly.

# Supported CPUs

The CPU type is specified with a generic parameter on the main structure. The trait `CpuDetails` contains all the details of the emulated CPU:
- Instruction execution times
- Exception processing times
- Exception stack format

m68000 provides CPU details for the following CPUs:
* MC68000 (as described in the M68000 8-/16-/32-Bit Microprocessors User’s Manual, Ninth Edition)
* SCC68070 microcontroller

# How to use

m68000 requires a nightly compiler as it uses the `btree_drain_filter` and `bigint_helper_methods` features of the std.

To behave properly, overflow checks MUST be disabled for this crate by adding the following lines in your `Cargo.toml`:

```toml
[profile.dev.package.m68000]
overflow-checks = false
```

First, since the memory map is application-dependant, it is the user's responsibility to define it by implementing the `MemoryAccess` trait on their memory structure, and passing it to the core on each instruction execution.

Second, choose the CPU behavior by specifying the instance that implements the `CpuDetails` trait, whether it is your own or one of the provided ones in the crate.

The file `src/bin/scc68070.rs` is a usage example that implements the SCC68070 microcontroller.

## Basic Rust example

```rs
const MEM_SIZE: u32 = 65536;
struct Memory([u8; MEM_SIZE as usize]); // Define your memory management system.

impl MemoryAccess for Memory { // Implement the MemoryAccess trait.
    fn get_byte(&mut self, addr: u32) -> Option<u8> {
        if addr < MEM_SIZE {
            Some(self.0[addr as usize])
        } else {
            None
        }
    }

    // And so on...
}

fn main() {
    let mut memory = Memory([0; MEM_SIZE as usize]);
    // Load the program in memory here.
    let mut cpu: M68000<m68000::cpu_details::Mc68000> = M68000::new();

    // Execute instructions
    cpu.interpreter(&mut memory);
}
```

# FFI and C interface

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

This library has a C interface to generate a static library and use it in the language you want.

To generate the static library, simply build the project using the correct target toolchain.
To change the build toolchain, add `+<toolchain name>`.
```sh
cargo build --release --features="ffi"
cargo +nightly-x86_64-pc-windows-gnu build --release --features="ffi"
```

The two C headers are generated using [cbindgen](https://github.com/eqrion/cbindgen) and a bit modified to adapt to the situation. It is recommended to use the ones in the root directory of the repo.

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

Include the generated header file in your project, and define your memory access callback functions. These functions will be passed to the core through a M68000Callbacks struct.

The returned values are in a GetSetResult struct. Set `GetSetResult.exception` to 0 and set `GetSetResult.data` to the value to be returned on success. Set `GetSetResult.exception` to 2 (Access Error vector) if an Access Error occurs.

## C example

```c
#include "m68000-ffi.h"

#include <stdint.h>
#include <stdlib.h>

#define MEMSIZE (1 << 20) // 1 MB.

GetSetResult getByte(uint32_t addr, void* user_data)
{
    const uint8_t* memory = user_data;
    if(addr < MEMSIZE)
        return (GetSetResult){
            .data = memory[addr],
            .exception = 0,
        };

    // If out of range, return an Access (bus) error.
    return (GetSetResult){
        .data = 0,
        .exception = 2,
    };
}

GetSetResult getWord(uint32_t addr, void* user_data)
{
    const uint8_t* memory = user_data;
    if(addr < MEMSIZE)
        return (GetSetResult){
            .data = (uint16_t)memory[addr] << 8
                | (uint16_t)memory[addr + 1],
            .exception = 0,
        };

    // If out of range, return an Access (bus) error.
    return (GetSetResult){
        .data = 0,
        .exception = 2,
    };
}

GetSetResult getLong(uint32_t addr, void* user_data)
{
    const uint8_t* memory = user_data;
    if(addr < MEMSIZE)
        return (GetSetResult){
            .data = (uint32_t)memory[addr] << 24
                | (uint32_t)memory[addr + 1] << 16
                | (uint32_t)memory[addr + 2] << 8
                | memory[addr + 3],
            .exception = 0,
        };

    // If out of range, return an Access (bus) error.
    return (GetSetResult){
        .data = 0,
        .exception = 2,
    };
}

GetSetResult setByte(uint32_t addr, uint8_t data, void* user_data)
{
    uint8_t* memory = user_data;
    GetSetResult res = {
        .data = 0,
        .exception = 0,
    };

    if(addr < MEMSIZE)
        memory[addr] = data;
    else
        res.exception = 2;

    return res;
}

GetSetResult setWord(uint32_t addr, uint16_t data, void* user_data)
{
    uint8_t* memory = user_data;
    GetSetResult res = {
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

GetSetResult setLong(uint32_t addr, uint32_t data, void* user_data)
{
    uint8_t* memory = user_data;
    GetSetResult res = {
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
    M68000Callbacks callbacks = {
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

# License

m68000 is distributed under the terms of the LGPL-3.0 or any later version. Refer to the COPYING and COPYING.LESSER files for more information.
