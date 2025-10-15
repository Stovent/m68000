// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Benchmark for the interpreter methods.
//!
//! Make sure the result of the benchmarked function is used,
//! whether by sending it to black_box, or to return it from the closure.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};

use m68000::cached_interpreter_flat_block::CachedInterpreterFlatBlock;
use m68000::cached_interpreter_trie_block::CachedInterpreterTrieBlock;
use m68000::M68000;
use m68000::cpu_details::Mc68000;

pub fn criterion_benchmark(c: &mut Criterion) {
    let bra = 0x6000_u16;
    let mut code = [
        0x0000, 0x1000, // Initial SSP
        0x0000, 0x0008, // Initial PC
        0x4E71, 0x4E71, // NOPs
        0x4E71, 0x4E71,
        0x4E71, 0x4E71,
        0x4E71, 0x4E71,
        0x4E71, 0x4E71,
        0x4E71, 0x4E71,
        0x4E71, 0x4E71,
        bra, 0xFFE2, // Bra.s
    ];
    let mut cpu = M68000::<Mc68000>::new();
    cpu.interpreter(code.as_mut_slice()); // Fetch the reset vectors on the first call.
    // The benchmarks executes the same instruction so no need for dedicated input management.

    c.bench_function("interpreter NOPs BRA", |b| b.iter(|| {
        for _ in 0..15 { // NOPs + BRA
            black_box(cpu.interpreter(black_box(code.as_mut_slice())));
        }
    }));

    c.bench_function("interpreter_exception NOPs BRA", |b| b.iter(|| {
        for _ in 0..15 { // NOPs + BRA
            black_box(cpu.interpreter_exception(black_box(code.as_mut_slice())));
        }
    }));

//     c.bench_function("cached_interpreter_1 NOPs BRA", |b| b.iter(|| {
//         for _ in 0..15 { // NOPs + BRA
//             black_box(cpu.cached_interpreter_1(black_box(code.as_mut_slice())));
//         }
//     }));
//
//     c.bench_function("cached_interpreter_exception_1 NOPs BRA", |b| b.iter(|| {
//         for _ in 0..15 { // NOPs + BRA
//             black_box(cpu.cached_interpreter_exception_1(black_box(code.as_mut_slice())));
//         }
//     }));

    /// Physical bus on a 68000 is 24 bits, minus 1 for even alignment.
    const CACHE_BITS: usize = 5;
    const CACHE_SIZE: usize = 1 << CACHE_BITS;
    const CACHE_MASK: usize = CACHE_SIZE - 1;

    let mut cached_interpreter_flat_block =
        CachedInterpreterFlatBlock::new(cpu, CACHE_SIZE, 1, CACHE_MASK);

    c.bench_function("cached_interpreter_flat_block NOPs BRA", |b| b.iter(|| {
        black_box(cached_interpreter_flat_block.cached_interpreter(black_box(code.as_mut_slice())));
    }));

    c.bench_function("cached_interpreter_exception_flat_block NOPs BRA", |b| b.iter(|| {
        black_box(cached_interpreter_flat_block.cached_interpreter_exception(black_box(code.as_mut_slice())));
    }));

    let mut cached_interpreter_trie_block = CachedInterpreterTrieBlock::new(cached_interpreter_flat_block.m68000);

    c.bench_function("cached_interpreter_trie_block NOPs BRA", |b| b.iter(|| {
        black_box(cached_interpreter_trie_block.cached_interpreter(black_box(code.as_mut_slice())));
    }));

    c.bench_function("cached_interpreter_exception_trie_block NOPs BRA", |b| b.iter(|| {
        black_box(cached_interpreter_trie_block.cached_interpreter_exception(black_box(code.as_mut_slice())));
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
