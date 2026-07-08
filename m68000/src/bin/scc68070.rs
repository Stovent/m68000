// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! This is a minimal SCC68070 microcontroller emulation used to run test ROMs.
//! It also demonstrate how to use this library in Rust projects.

use m68000::cached_interpreter_flat_block::CachedInterpreterFlatBlock;
use m68000::cached_interpreter_single_block::CachedInterpreterSingleBlock;
use m68000::cached_interpreter_trie_block::CachedInterpreterTrieBlock;
use m68000::M68000;
use m68000::memory_access::MemoryAccess;

use std::fs::File;
use std::io::Read;

type CpuScc68070 = m68000::cpu_details::Scc68070;

use core::hint::cold_path;

/// The microcontroller structure, with its CPU core and its internal peripherals memory.
struct Scc68070 {
    pub cpu: M68000<CpuScc68070>,
    pub memory: Memory68070,
}

/// The peripheral memory of the SCC68070 microcontroller, implementing the MemoryAccess trait.
struct Memory68070 {
    pub memory_swap: usize,
    pub ram: Box<[u8]>,
}

const ROM_BEGIN: u32 = 0x40_0000;
const ROM_END: u32 = 0x50_0000;

impl MemoryAccess for Memory68070 {
    fn get_byte(&mut self, addr: u32) -> Option<u8> {
        match addr {
            addr if (addr as usize) < self.ram.len() => {
                Some(self.ram[addr as usize])
            },
            0x8000_2011..=0x8000_201B => {
                if addr == 0x8000_2013 {
                    Some(0b0000_1110)
                } else {
                    Some(0)
                }
            },
            _ => {
                cold_path();
                None
            },
        }
    }

    fn get_word(&mut self, addr: u32) -> Option<u16> {
        let addr = if self.memory_swap < 4 {
            cold_path();
            self.memory_swap += 1;
            addr + ROM_BEGIN
        } else {
            addr
        } as usize;

        if addr < self.ram.len() { // addr is even so no need to subtract 1.
            Some(u16::from_be_bytes(self.ram[addr..addr + 2].try_into().unwrap()))
        } else {
            cold_path();
            None
        }
    }

    fn get_long(&mut self, addr: u32) -> Option<u32> {
        let addr = if self.memory_swap < 4 {
            cold_path();
            self.memory_swap += 2;
            addr + ROM_BEGIN
        } else {
            addr
        } as usize;

        if addr < self.ram.len() - 3 {
            Some(u32::from_be_bytes(self.ram[addr..addr + 4].try_into().unwrap()))
        } else {
            cold_path();
            None
        }
    }

    fn set_byte(&mut self, addr: u32, value: u8) -> Option<()> {
        match addr {
            addr if (addr as usize) < self.ram.len() => {
                self.ram[addr as usize] = value;
                Some(())
            },
            0x8000_2011..=0x8000_2019 => {
                cold_path();
                if addr == 0x8000_2019 {
                    print!("{}", value as char);
                }
                Some(())
            },
            _ => {
                cold_path();
                None
            },
        }
    }

    fn set_word(&mut self, addr: u32, value: u16) -> Option<()> {
        self.set_byte(addr, (value >> 8) as u8)?;
        self.set_byte(addr + 1, value as u8)
    }

    fn reset_instruction(&mut self) {}
}

fn main()
{
    let mut ram = Memory68070 {
        memory_swap: 0,
        ram: vec![0; 0x50_0000].into_boxed_slice(),
    };

    // Load the program in memory.
    let mut bios_file = File::open("cpudiag40.rom").expect("no cpudiag40.rom");
    match bios_file.read(&mut ram.ram[ROM_BEGIN as usize..]) {
        Ok(_) => (),
        Err(e) => panic!("Failed to read from cpudiag40.rom: {e}"),
    }

    let mut scc68070 = Scc68070 {
        cpu: M68000::new(),
        memory: ram,
    };
    // let mut cached_interpreter =
    //     CachedInterpreterFlatBlock::new(M68000::<CpuScc68070>::new(), 1 << 23, 1, 0xFF_FFFF);
    // let mut cached_interpreter = CachedInterpreterTrieBlock::new(M68000::<CpuScc68070>::new());
    let mut cached_interpreter = CachedInterpreterSingleBlock::new(M68000::<CpuScc68070>::new(), ROM_BEGIN, ROM_END);

    let start = std::time::Instant::now();

    // Execute 1 000 000 000 instructions.
    for _ in 0..1_000_000_000 {
        cached_interpreter.cached_interpreter(&mut scc68070.memory);
        // scc68070.cpu.cached_interpreter_1(&mut scc68070.memory);
        // scc68070.cpu.interpreter(&mut scc68070.memory);
        // let (dis, _) = scc68070.cpu.disassembler_interpreter(&mut scc68070.memory);
        // println!("{dis}");
    }

    let delay = start.elapsed();
    println!("{delay:?}");
}
