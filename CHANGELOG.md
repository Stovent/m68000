# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- `ffi` feature to enable `repr(C)` on some structs and enums.

### Changed
- Change license from LGPL-3.0 to MPL-2.0.
- Separate the C interface from the main crate.
- CPU behavior is now controlled using a trait and generic member instead of features (breaking).
- Move the register access helper methods to the Registers struct.
- Status Register's default function returns a SR with value 0x2700 (breaking).
- Interrupt's exception processing sets the interrupt priority mask.
- Interrupt level 7 is non-maskable.
- Use wrapping types and methods so overflow checks can be enabled.
- Make MemoryIter generic over the underlying MemoryAccess trait object.

### Removed
- M68000 does not store the cycles count anymore (breaking).

### Fixed
- Fix ADDQ/SUBQ immediate data 0 not interpreted as 8.
- Fix ADDQ/SUBQ truncating address registers.
- Fix immediate Shift/Rotate count of 0 not disassembled as 8.

## [0.1.1] - 2022-08-28
### Fixed
- Fixed docs.rs documentation generation.

## [0.1.0] - 2022-08-28
- Initial release.

[Unreleased]: https://github.com/Stovent/m68000/compare/v0.1.0...master
[0.1.1]: https://github.com/Stovent/m68000/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/Stovent/m68000/releases/tag/v0.1.0
