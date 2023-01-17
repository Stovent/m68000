#!/bin/sh

lib_expanded='m68000/lib-expanded.rs'
lib_header='include/m68000/m68000.h'
ffi_expanded='m68000-ffi/lib-expanded.rs'
ffi_header='include/m68000/m68000-ffi.h'

cargo +nightly expand --lib --manifest-path ./m68000/Cargo.toml --features="ffi" > $lib_expanded
cargo +nightly expand --lib --manifest-path ./m68000-ffi/Cargo.toml > $ffi_expanded

cbindgen.exe --config ./m68000-ffi/cbindgen.toml --output $lib_header $lib_expanded
cbindgen.exe --config ./m68000-ffi/cbindgen-ffi.toml --output $ffi_header $ffi_expanded

# Add the forward declaration of the M68000<...> structs manually as they are generic and not repr(C).
sed -i 's/#include <stdint.h>/#include <stdint.h>\n\ntypedef struct m68000_mc68000_s m68000_mc68000_t;\ntypedef struct m68000_scc68070_s m68000_scc68070_t;/g' $lib_header
sed -i 's/Wrapping<uint32_t>/uint32_t/g' $lib_header

# Avoid duplication names in enumerations as C doesn't like it.
sed -i ':a;N;$!ba;s/STOP\r\n.* Immediate,/STOP\r\n     *\/\r\n    Immediate_,/g' $lib_header
sed -i ':a;N;$!ba;s/TRAP\r\n.*Vector/TRAP\r\n     *\/\r\n    Vector_/g' $lib_header

sed -i 's/M68000<Mc68000>/m68000_mc68000_t/g' $ffi_header
sed -i 's/M68000<Scc68070>/m68000_scc68070_t/g' $ffi_header

rm $ffi_expanded $lib_expanded
