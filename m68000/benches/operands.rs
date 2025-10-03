// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Benchmark for the operand functions.
//!
//! Make sure the result of the benchmarked function is used,
//! whether by sending it to black_box, or to return it from the closure.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};

use m68000::instruction::{displacement, immediate, size_effective_address_immediate, vector, rotation_direction_size_mode_register};
use m68000::memory_access::MemoryAccess;

pub fn criterion_benchmark(c: &mut Criterion) {
    {
        let bra = 0x60FE_u16;
        let mut code_bra = [bra];
        c.bench_function("displacement byte", |b| b.iter(|| {
            let mut memory_iter = code_bra.iter_u16(2).unwrap();
            black_box(displacement(black_box(bra), &mut memory_iter));
        }));
    }

    {
        let bra = 0x6000_u16;
        let mut code_bra = [bra, 0xFFFE];
        c.bench_function("displacement word", |b| b.iter(|| {
            let mut memory_iter = code_bra.iter_u16(2).unwrap();
            black_box(displacement(black_box(bra), &mut memory_iter));
        }));
    }

    {
        let stop = 0x4E72_u16;
        let mut code_stop = [stop, 0x0700];
        c.bench_function("immediate", |b| b.iter(|| {
            let mut memory_iter = code_stop.iter_u16(2).unwrap();
            black_box(immediate(black_box(&mut memory_iter)));
        }));
    }

    {
        let andi = 0x02B9; // Long, absolute long EA.
        let mut code_andi = [andi, 0x0000, 0x0000, 0x0000, 0x0000];
        c.bench_function("size_effective_address_immediate", |b| b.iter(|| {
            let mut memory_iter = code_andi.iter_u16(2).unwrap();
            black_box(size_effective_address_immediate(black_box(andi), &mut memory_iter));
        }));
    }

    {
        let trap = 0x4E40_u16;
        c.bench_function("vector", |b| b.iter(|| {
            black_box(vector(black_box(trap)));
        }));
    }

    {
        let lsr = 0xE088_u16;
        c.bench_function("rotation_direction_size_mode_register", |b| b.iter(|| {
            black_box(rotation_direction_size_mode_register(black_box(lsr)));
        }));
    }
}

criterion_group!(operands, criterion_benchmark);
criterion_main!(operands);
