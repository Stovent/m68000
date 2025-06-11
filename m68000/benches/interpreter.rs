//! Benchmark for the interpreter.$

#![feature(test)]

extern crate test;
use test::Bencher;

use m68000::M68000;
use m68000::assembler as asm;
use m68000::cpu_details::Scc68070;

struct Memory {
    bytes: Vec<u16>,
}

// fn vec_u8_to_vec_u16(data: &[u16]) -> Vec<u8> {
//     let mut bytes = Vec::new();
//     for d in data {
//         bytes.push((d >> 8) as u8);
//         bytes.push(*d as u8);
//     }
//     bytes
// }

fn generate_bench_program() -> Memory {
    let mut bytes = vec![
        0, 0, // SSP
        0, 0x4, // PC
    ];

    let bra = asm::bra(-2); // infinite loop.
    bytes.extend_from_slice(&bra);

    Memory {
        // bytes: vec_u8_to_vec_u16(&bytes),
        bytes,
    }
}

#[bench]
fn bench_interpreter(b: &mut Bencher) {
    let mut cpu: M68000<Scc68070> = M68000::new();
    let mut memory = generate_bench_program();

    // let start = Instant::now();

    b.iter(|| { for _ in 0..1_000_000 {
            cpu.interpreter_unified::<false, [u16]>(memory.bytes.as_mut());
        }
    });

    // let finish = Instant::now();
    // let delta = finish - start;

    // println!("{}", delta.as_millis());
}
