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

See [CINTERFACE.md](https://github.com/Stovent/CeDImu/blob/master/CINTERFACE.md).

# License

m68000 is distributed under the terms of the [Mozilla Public License version 2.0](https://www.mozilla.org/MPL/2.0/). Refer to the LICENSE file for more information.
