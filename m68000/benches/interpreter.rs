//! Benchmark for the interpreter methods.
//!
//! Make sure the result of the benchmarked function is used,
//! whether by sending it to black_box, or to return it from the closure.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};

use m68000::M68000;
use m68000::cpu_details::Mc68000;

pub fn criterion_benchmark(c: &mut Criterion) {
    let bra = 0x6000_u16;
    let mut code = [
        0x0000, 0x1000, // Initial SSP
        0x0000, 0x0008, // Initial PC
        bra, 0xFFFE, // Bra.s
    ];
    let mut cpu = M68000::<Mc68000>::new();
    cpu.interpreter(code.as_mut_slice()); // Fetch the reset vectors on the first call.
    // The benchmarks executes the same instruction so no need for dedicated input management.

    c.bench_function("interpreter", |b| b.iter(|| {
        black_box(cpu.interpreter(black_box(code.as_mut_slice())));
    }));

    c.bench_function("interpreter_exception", |b| b.iter(|| {
        black_box(cpu.interpreter_exception(black_box(code.as_mut_slice())));
    }));

    // TODO: is the below code the more correct way to benchmark?
//     let setup = || {
//         let bra = 0x6000_u16;
//         let mut code = [
//             0x0000, 0x1000, // Initial SSP
//             0x0000, 0x0008, // Initial PC
//             bra, 0xFFFE, // Bra.s
//         ];
//         let mut cpu = M68000::<Scc68070>::new();
//         cpu.interpreter(code.as_mut_slice()); // Fetch the reset vectors on the first call.
//         // The benchmarks executes the same instruction so no need for dedicated input management.
//         (cpu, code)
//     };
//
//     c.bench_function("interpreter", |b|
//         b.iter_batched_ref(setup, |(cpu, code)| {
//             cpu.interpreter(black_box(code.as_mut_slice()));
//         }, criterion::BatchSize::SmallInput)
//     );
//
//     c.bench_function("interpreter_exception", |b|
//         b.iter_batched_ref(setup, |(cpu, code)| {
//             cpu.interpreter_exception(black_box(code.as_mut_slice()));
//         }, criterion::BatchSize::SmallInput)
//     );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
