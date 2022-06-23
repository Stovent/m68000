//! This program is a test function that runs a M68000 test ROM.

use m68000::M68000;
use m68000::exception::Vector;
use m68000::memory_access::{GetResult, MemoryAccess, SetResult};

use std::fs::File;
use std::io::Read;

/// The microcontroller structure, with its CPU core and its internal peripherals memory.
struct Scc68070 {
    pub cpu: M68000,
    pub memory: Memory68070,
}

/// The peripheral memory of the SCC68070 microcontroller, implementing the MemoryAccess trait.
struct Memory68070 {
    pub memory_swap: usize,
    pub ram: Box<[u8]>,
}

impl MemoryAccess for Memory68070 {
    fn get_byte(&mut self, addr: u32) -> GetResult<u8> {
        if addr >= 0x8000_2011 && addr <= 0x8000_201B {
            if addr == 0x8000_2013 {
                Ok(0b0000_1110)
            } else {
                Ok(0)
            }
        } else if (addr as usize) < self.ram.len() {
            Ok(self.ram[addr as usize])
        } else {
            Err(Vector::AccessError as u8)
        }
    }

    fn get_word(&mut self, addr: u32) -> GetResult<u16> {
        if self.memory_swap < 4 {
            self.memory_swap += 1;
            Ok((self.get_byte(addr + 0x40_0000)? as u16) << 8 | self.get_byte(addr + 0x40_0001)? as u16)
        } else {
            Ok((self.get_byte(addr)? as u16) << 8 | self.get_byte(addr + 1)? as u16)
        }
    }

    fn set_byte(&mut self, addr: u32, value: u8) -> SetResult {
        if addr >= 0x8000_2011 && addr <= 0x8000_2019 {
            if addr == 0x8000_2019 {
                print!("{}", value as char);
            }
            Ok(())
        } else if (addr as usize) < self.ram.len() {
            self.ram[addr as usize] = value;
            Ok(())
        } else {
            Err(Vector::AccessError as u8)
        }
    }

    fn set_word(&mut self, addr: u32, value: u16) -> SetResult {
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
        Ok(i) => println!("Successfully read {} bytes from cpudiag40.rom", i),
        Err(e) => panic!("Failed to read from cpudiag40.rom: {}", e),
    }

    let mut scc68070 = Scc68070 {
        cpu: M68000::new_reset(&mut ram),
        memory: ram,
    };

    // Execute 1 000 000 000 instructions
    for _ in 0..1_000_000_000 {
        scc68070.cpu.interpreter(&mut scc68070.memory);
    }
}
