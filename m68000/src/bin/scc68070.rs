// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! This is a minimal SCC68070 microcontroller emulation used to run test ROMs.
//! It also demonstrate how to use this library in Rust projects.

use m68000::M68000;
use m68000::memory_access::MemoryAccess;

use std::fs::File;
use std::io::Read;

/// The microcontroller structure, with its CPU core and its internal peripherals memory.
struct Scc68070 {
    pub cpu: M68000<m68000::cpu_details::Scc68070>,
    pub memory: Memory68070,
}

/// The peripheral memory of the SCC68070 microcontroller, implementing the MemoryAccess trait.
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
            if addr == 0x8000_2019 {
                print!("{}", value as char);
            }
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

fn main()
{
    let mut ram = Memory68070 {
        memory_swap: 0,
        ram: vec![0; 0x50_0000].into_boxed_slice(),
    };

    // Load the program in memory.
    let mut bios_file = File::open("cpudiag40.rom").expect("no cpudiag40.rom");
    match bios_file.read(&mut ram.ram[0x40_0000..]) {
        Ok(_) => (),
        Err(e) => panic!("Failed to read from cpudiag40.rom: {}", e),
    }

    let mut scc68070 = Scc68070 {
        cpu: M68000::new(),
        memory: ram,
    };

    let start = std::time::Instant::now();

    // Execute 1 000 000 000 instructions.
    for _ in 0..1_000_000_000 {
        scc68070.cpu.interpreter(&mut scc68070.memory);
        // let (dis, _) = scc68070.cpu.disassembler_interpreter(&mut scc68070.memory);
        // println!("{dis}");
    }

    let delay = start.elapsed();
    println!("{delay:?}");
}
