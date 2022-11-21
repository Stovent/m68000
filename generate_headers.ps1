$lib_expanded = 'm68000/lib-expanded.rs'
$lib_header = 'm68000.h'
$ffi_expanded = 'm68000-ffi/lib-expanded.rs'
$ffi_header = 'm68000-ffi.h'

cargo +nightly expand --lib --manifest-path ./m68000/Cargo.toml --features="ffi" | Out-File -Encoding "UTF8" $lib_expanded
cargo +nightly expand --lib --manifest-path ./m68000-ffi/Cargo.toml | Out-File -Encoding "UTF8" $ffi_expanded

cbindgen.exe --config ./m68000-ffi/cbindgen.toml --output $lib_header $lib_expanded
cbindgen.exe --config ./m68000-ffi/cbindgen-ffi.toml --output $ffi_header $ffi_expanded

# Add the forward declaration of the M68000<...> structs manually as they are generic and not repr(C).
(Get-Content $lib_header -Raw) -replace "#include <stdint.h>", "#include <stdint.h>`r`n`r`ntypedef struct m68000_mc68000_s m68000_mc68000_t;`r`ntypedef struct m68000_scc68070_s m68000_scc68070_t;" | Out-File -encoding ASCII $lib_header

# Avoid duplication names in enumerations as C doesn't like it.
(Get-Content $lib_header -Raw) -replace "STOP`r`n.*\*/`r`n.*Immediate", "STOP`r`n     */`r`n    Immediate_" | Out-File -encoding ASCII $lib_header
(Get-Content $lib_header -Raw) -replace "TRAP`r`n.*\*/`r`n.*Vector", "TRAP`r`n     */`r`n    Vector_" | Out-File -encoding ASCII $lib_header

(Get-Content $ffi_header) -replace 'M68000<Mc68000>', 'm68000_mc68000_t' | Out-File -encoding ASCII $ffi_header
(Get-Content $ffi_header) -replace 'M68000<Scc68070>', 'm68000_scc68070_t' | Out-File -encoding ASCII $ffi_header

Remove-Item $lib_expanded
Remove-Item $ffi_expanded
