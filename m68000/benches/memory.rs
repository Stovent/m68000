// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Benchmark for the memory access FFI.
//!
//! Make sure the result of the benchmarked function is used,
//! whether by sending it to black_box, or to return it from the closure.
//!
//! This benchmark has a hack in the get_word/get_long functions to always return a value instead of a bus error.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};

use m68000::M68000;
use m68000::cpu_details::Scc68070;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut memory = vec![0u16; 0x1000].into_boxed_slice();
    let mut cpu = M68000::<Scc68070>::new();
    // No need to use iter_batched because this base state behaves the same after each execution.

    c.bench_function("peek_next_word", |b| b.iter(|| {
        black_box(cpu.peek_next_word(black_box(memory.as_mut())).unwrap_or(0));
    }));

    c.bench_function("get_next_word", |b| b.iter(|| {
        black_box(cpu.get_next_word(black_box(memory.as_mut())).unwrap_or(0));
    }));

    c.bench_function("get_next_long", |b| b.iter(|| {
        black_box(cpu.get_next_long(black_box(memory.as_mut())).unwrap_or(0));
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
