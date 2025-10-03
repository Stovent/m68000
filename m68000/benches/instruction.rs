// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Benchmark for instruction decoding.
//!
//! Make sure the result of the benchmarked function is used,
//! whether by sending it to black_box, or to return it from the closure.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};

use m68000::instruction::Instruction;
use m68000::memory_access::MemoryAccess;

pub fn criterion_benchmark(c: &mut Criterion) {
    let illegal = 0x4AFC_u16;
    let mut code = [illegal];

    c.bench_function("instruction decoding", |b| b.iter(|| {
        let mut memory_iter = code.iter_u16(0).unwrap();
        black_box(Instruction::from_memory(black_box(&mut memory_iter)).unwrap());
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
