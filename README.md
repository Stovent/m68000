# m68000

m68000 is a Motorola 68000 assembler, disassembler and interpreter written in Rust.

This library emulates the common user and supervisor instructions of the M68k ISA. It is configurable at compile-time to behave like the given CPU type (see below), changing the instruction's execution times and exception handling.

# Supported CPUs

The CPU type is specified at compile-time as a feature. There must be one and only one feature specified.

There are no default features. If you don't specify any feature or specify more than one, a compile-time error is raised.

* MC68000 (feature "cpu-mc68000")
* SCC68070 (feature "cpu-scc68070")

# How to use

Include this library in your project and configure the CPU type by specifying the correct feature.

Since the memory map is application-dependant (especially for the SCC68070 microcontroller), it is the user's responsibility to define it by implementing the `MemoryAccess` trait on their memory structure, and passing it to the core on each instruction execution.

The `mais.rs` file is a usage example that implements the SCC68070 microcontroller.

# C interface

## Build the C interface

This library has a C interface to generate a static library and use it in your C/C++ projects.

To generate the static library, simply build the project using the correct target toolchain.
```sh
cargo build --release --lib --features=cpu-scc68070
```

Change the CPU type you want to use by changing the last parameter of the previous command. To change the build toolchain, add `+<toolchain name>`. For example, to build it for windows targetting the GCC compiler, type
```sh
cargo +stable-x86_64-pc-windows-gnu build --release --lib --features=cpu-scc68070
```

To generate the C header file, it is recommended to use [cbindgen](https://github.com/eqrion/cbindgen), and to use the `cbindgen.toml` file provided in this repo. In a terminal, type the following command to generate the header file:
```sh
bindgen.exe --config .\cbindgen.toml --crate m68000 --output m68000.h
```

You can change the name of the file by changing the last parameter of the previous command.

## Use the C interface

Include the generated header file in your project, and define your memory access callback functions. These functions will be passed to the core through a M68000Callbacks struct.

The returned values are in a GetSetResult struct. Set `GetSetResult.exception` to 0 and set `GetSetResult.data` to the value to be returned on success. Set `GetSetResult.exception` to 2 (Access Error vector) if an Access Error occurs.

```c
#include "m68000.h"

uint8_t memory[0x500000] = {0};

GetSetResult getByte(uint32_t addr)
{
    GetSetResult gs = {
        .data = 0,
        .exception = 0
    };

    if(addr >= 0x500000)
        gs.exception = 2; // If out of range, return an Access Error.
    else
        gs.data = memory[addr];

    return gs;
}

GetSetResult getWord(uint32_t addr)
{
    GetSetResult gs = {
        .data = 0,
        .exception = 0
    };

    if(addr >= 0x500000)
        gs.exception = 2; // If out of range, return an Access Error.
    else
    {
        gs.data = (uint16_t)memory[addr] << 8 | (uint16_t)memory[addr + 1];
    }

    return gs;
}

GetSetResult getLong(uint32_t addr)
{
    GetSetResult gs = {
        .data = 0,
        .exception = 0
    };

    if(addr >= 0x500000)
        gs.exception = 2; // If out of range, return an Access Error.
    else
    {
        gs.data = (uint32_t)memory[addr] << 24
                | (uint32_t)memory[addr + 1] << 16
                | (uint32_t)memory[addr + 2] << 8
                | (uint32_t)memory[addr + 3];
    }

    return gs;
}

GetSetResult setByte(uint32_t addr, uint8_t data)
{
    GetSetResult gs = {
        .data = 0, // Always 0 in Set functions.
        .exception = 0
    };

    if(addr >= 0x500000)
        gs.exception = 2; // If out of range, return an Access Error.
    else
        memory[addr] = data;

    return gs;
}

GetSetResult setWord(uint32_t addr, uint16_t data)
{
    GetSetResult gs = {
        .data = 0, // Always 0 in Set functions.
        .exception = 0
    };

    if(addr >= 0x500000)
        gs.exception = 2; // If out of range, return an Access Error.
    else
    {
        memory[addr] = data >> 8;
        memory[addr + 1] = data;
    }

    return gs;
}

GetSetResult setLong(uint32_t addr, uint32_t data)
{
    GetSetResult gs = {
        .data = 0, // Always 0 in Set functions.
        .exception = 0
    };

    if(addr >= 0x500000)
        gs.exception = 2; // If out of range, return an Access Error.
    else
    {
        memory[addr] = data >> 24;
        memory[addr + 1] = data >> 16;
        memory[addr + 2] = data >> 8;
        memory[addr + 3] = data;
    }

    return gs;
}

void reset() {}

void disassembler(uint32_t pc, const char* str) {}

int main()
{
    // Load your program in memory here.

    // Create the M68000Callbacks struct, and assign the callbacks to it.
    M68000Callbacks callbacks = {
        .get_byte = getByte,
        .get_word = getWord,
        .get_long = getLong,
        .set_byte = setByte,
        .set_word = setWord,
        .set_long = setLong,
        .reset_instruction = reset,
        .disassembler = disassembler,
    };

    // Create a new core.
    M68000* core = m68000_new();
    // Enable the disassembler if you want, and implement the disassembler callback.
//    m68000_enable_disassembler(core, true);

    // Run the core for the given number of cycles.
    m68000_cycle(core, &callbacks, 13001038281);
    // Or call interpreter to execute instructions one by one.
//    m68000_interpreter(core, &callbacks);

    // Free the allocated memory for the core.
    m68000_delete(core);
    return 0;
}
```

# License

m68000 is distributed under the terms of the LGPL-3.0 or any later version. Refer to the LICENSE and LICENSE.LESSER files for more information.
