# m68000

m68000 is a Motorola 68000 interpreter written in Rust. Its goal is to facilitate the creation of any application that needs to emulate a 68000-based system.

# License

m68000 is distributed under the terms of the GPLv2 license. Refer to the [LICENSE](https://github.com/Stovent/m68000/blob/master/LICENSE) file.

# No std

The [no-std](https://github.com/Stovent/m68000/tree/no_std) branch contains a version of this crate that can be compiled without the standard library. The disassembler can be enabled with `m68000 = { features = "disassembler" }`, but it requires `String` from the std.

