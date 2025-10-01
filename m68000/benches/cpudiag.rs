//! Benchmark for the cpudiag test ROM.
//!
//! Make sure the result of the benchmarked function is used,
//! whether by sending it to black_box, or to return it from the closure.

use core::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};

use m68000::{M68000, MemoryAccess};
use m68000::cpu_details::Scc68070;

struct McuScc68070 {
    pub cpu: M68000<Scc68070>,
    pub memory: Memory68070,
}

struct Memory68070 {
    pub memory_swap: usize,
    pub ram: Box<[u8]>,
}

impl MemoryAccess for Memory68070 {
    fn get_byte(&mut self, addr: u32) -> Option<u8> {
        if addr >= 0x8000_2011 && addr <= 0x8000_201B {
            if addr == 0x8000_2013 {
                Some(0b0000_1110)
            } else {
                Some(0)
            }
        } else if (addr as usize) < self.ram.len() {
            Some(self.ram[addr as usize])
        } else {
            None
        }
    }

    fn get_word(&mut self, addr: u32) -> Option<u16> {
        if self.memory_swap < 4 {
            self.memory_swap += 1;
            Some((self.get_byte(addr + 0x40_0000)? as u16) << 8 | self.get_byte(addr + 0x40_0001)? as u16)
        } else {
            Some((self.get_byte(addr)? as u16) << 8 | self.get_byte(addr + 1)? as u16)
        }
    }

    fn set_byte(&mut self, addr: u32, value: u8) -> Option<()> {
        if addr >= 0x8000_2011 && addr <= 0x8000_2019 {
            Some(())
        } else if (addr as usize) < self.ram.len() {
            self.ram[addr as usize] = value;
            Some(())
        } else {
            None
        }
    }

    fn set_word(&mut self, addr: u32, value: u16) -> Option<()> {
        self.set_byte(addr, (value >> 8) as u8)?;
        self.set_byte(addr + 1, value as u8)
    }

    fn reset_instruction(&mut self) {}
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let test_rom = std::fs::read("cpudiag40.rom").expect("no cpudiag40.rom");
    let setup = || {
        let mut memory = Memory68070 {
            memory_swap: 0,
            ram: vec![0; 0x50_0000].into_boxed_slice(),
        };

        // Load the program in memory.
        let begin = 0x40_0000;
        let end = begin + test_rom.len();
        memory.ram[begin..end].copy_from_slice(&test_rom);

        McuScc68070 {
            cpu: M68000::new(),
            memory,
        }
    };

    c.bench_function("cpudiag interpreter", |b|
        b.iter_batched(setup, |mut scc68070| {
            for _ in 0..10_000_000 {
                black_box(scc68070.cpu.interpreter(&mut scc68070.memory));
            }
        }, criterion::BatchSize::SmallInput)
    );

    c.bench_function("cpudiag interpreter_exception", |b|
        b.iter_batched(setup, |mut scc68070| {
            for _ in 0..10_000_000 {
                black_box(scc68070.cpu.interpreter_exception(&mut scc68070.memory));
            }
        }, criterion::BatchSize::SmallInput)
    );
}

/*
cpudiag interpreter     time:   [99.848 ms 100.24 ms 100.69 ms]
                        change: [−0.2439% +0.2826% +0.8291%] (p = 0.30 > 0.05)
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe

Benchmarking cpudiag interpreter_exception: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 11.7s, or reduce sample count to 40.
cpudiag interpreter_exception
                        time:   [114.88 ms 115.19 ms 115.53 ms]
                        change: [−0.1153% +0.1951% +0.5338%] (p = 0.27 > 0.05)
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild

cpudiag m68000_mc68000_interpreter
                        time:   [61.813 ms 61.964 ms 62.130 ms]
                        change: [−0.7927% −0.3231% +0.1099%] (p = 0.16 > 0.05)
                        No change in performance detected.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe

Benchmarking cpudiag m68000_mc68000_interpreter_exception: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 7.3s, or reduce sample count to 60.
cpudiag m68000_mc68000_interpreter_exception
                        time:   [72.364 ms 72.563 ms 72.784 ms]
                        change: [−1.5018% −0.5828% +0.1564%] (p = 0.19 > 0.05)
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
*/

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
