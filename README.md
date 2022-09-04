# m68000

m68000 is a Motorola 68000 interpreter, disassembler and assembler (code emitter) written in Rust.

This library emulates the common user and supervisor instructions of the M68k ISA. It is configurable to behave like the given CPU type (see below), changing the instruction's execution times and exception handling.

This library has been designed to be used in two different contexts:

- It can be used to emulate a whole CPU, and the user of this library only have to call the interpreter methods and exception when an interrupt or reset occurs. This is the typical use case for an emulator.
- It can also be used as a M68k user-land interpreter to run an M68k program, but without the requirement of having an operating system compiled to binary M68k. In this case, the application runs the program until an exception occurs (TRAP for syscalls, zero divide, etc.) and treat the exception in Rust code (or any other language using the C interface), so the application can implement the surrounding environment required by the M68k program in a high level language and not in M68k assembly.

# Supported CPUs

The CPU type is specified with a generic parameter on the main structure. The trait `CpuDetails` contains all the details of the emulated CPU:
- Instruction execution times
- Exception processing times
- Exception stack format

m68000 provides CPU details for the following CPUs:
* MC68000 (as described in the M68000 8-/16-/32-Bit Microprocessors User’s Manual, Ninth Edition)
* SCC68070 microcontroller

# How to use

m68000 requires a nightly compiler as it uses the `btree_drain_filter` feature of the std.

First, since the memory map is application-dependant, it is the user's responsibility to define it by implementing the `MemoryAccess` trait on their memory structure, and passing it to the core on each instruction execution.

Second, choose the CPU behavior by specifying the instance that implements the `CpuDetails` trait, whether it is your own or one the provided ones.

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

# C interface

## Build the C interface

This library has a C interface to generate a static library and use it in the language you want.

To generate the static library, simply build the project using the correct target toolchain.
```sh
cargo build --release --lib --features=cpu-scc68070
```

Change the CPU type you want to use by changing the last parameter of the previous command. To change the build toolchain, add `+<toolchain name>`. For example, to build it for windows targetting the MinGW compiler, type
```sh
cargo +nightly-x86_64-pc-windows-gnu build --release --lib --features=cpu-scc68070
```

To generate the C header file, it is recommended to use [cbindgen](https://github.com/eqrion/cbindgen), and to use the `cbindgen.toml` file provided in this repo. In a terminal, type the following command to generate the header file:
```sh
bindgen.exe --config .\cbindgen.toml --crate m68000 --output m68000.h
```

You can change the name of the file by changing the last parameter of the previous command.

## Use the C interface

The complete documentation for the functions and structures can be found in the `cinterface.rs` module.
See the C example below for a basic start.

Include the generated header file in your project, and define your memory access callback functions. These functions will be passed to the core through a M68000Callbacks struct.

The returned values are in a GetSetResult struct. Set `GetSetResult.exception` to 0 and set `GetSetResult.data` to the value to be returned on success. Set `GetSetResult.exception` to 2 (Access Error vector) if an Access Error occurs.

## C example

```c
#include "m68000.h"

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

    M68000* core = m68000_new(); // Create a new core.
    // Now execute instructions as you want.
    m68000_interpreter(core, &callbacks);

    // end of the program.
    m68000_delete(core);
    free(memory);
    return 0;
}
```

# License

m68000 is distributed under the terms of the LGPL-3.0 or any later version. Refer to the COPYING and COPYING.LESSER files for more information.
