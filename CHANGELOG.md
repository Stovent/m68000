# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.3] - 2025-06-11
### Changed
- m68000 now builds on stable Rust instead of nightly.

## [0.2.2] - 2025-06-04
### Added
- Push the current opcode in SCC68070 long exception stack frame.

## [0.2.1] - 2023-08-28
### Fixed
- Fixed cargo metadata.

## [0.2.0] - 2023-08-28
### Added
- `ffi` feature to enable `repr(C)` on some structs and enums.
- Implement MemoryAccess trait for `[u8]`, `&[u8]`, `[u16]` and `&[u16]`.
- Line A and line F emulator exceptions.
- Added `with_sr` function for `Registers`.

### Changed
- Change license from LGPL-3.0 to MPL-2.0.
- Separate the C interface from the main crate.
- CPU behavior is now controlled using a trait and generic member instead of features (breaking).
- Move the register access helper methods to the Registers struct.
- Status Register's default function returns a SR with value 0x2700 (breaking).
- Use wrapping types and methods so overflow checks can be enabled.
- Make MemoryIter generic over the underlying MemoryAccess trait object.

### Removed
- M68000 does not store the cycles count anymore (breaking).

### Fixed
- Fix ADDQ/SUBQ immediate data 0 not interpreted as 8.
- Fix ADDQ/SUBQ truncating address registers.
- Fix immediate Shift/Rotate count of 0 not disassembled as 8.
- Fix ABCD/NBCD/SBCD.
- Fix DIVS/DIVU changing the destination even when an overflow occured.
- Interrupt's exception processing sets the interrupt priority mask.
- Interrupt level 7 is non-maskable.
- Privileged instructions can't trigger Trace exceptions.

## [0.1.1] - 2022-08-28
### Fixed
- Fixed docs.rs documentation generation.

## [0.1.0] - 2022-08-28
- Initial release.

# On each new version
- Move the elements in [Unreleased] in a new section for their version with the release date.
- Edit and add a new entry below for the new version.
- git tag
- cargo publish

[Unreleased]: https://github.com/Stovent/m68000/compare/v0.2.3...master
[0.2.3]: https://github.com/Stovent/m68000/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/Stovent/m68000/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/Stovent/m68000/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/Stovent/m68000/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/Stovent/m68000/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/Stovent/m68000/releases/tag/v0.1.0
