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

# License

m68000 is distributed under the terms of the LGPL-3.0 or any later version. Refer to the LICENSE and LICENSE.LESSER files for more information.
