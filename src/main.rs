//! This program is a test function that runs a M68000 test ROM.

use m68000::{Config, M68000, StackFrame};
use m68000::memory_access::{MemoryAccess, MemoryIter};

use std::fs::File;
use std::io::Read;

const MB: usize = 1024 * 1024 * 5;

struct Memory68070 {
    pub memory_swap: usize,
    pub ram: Box<[u8]>,
}

impl MemoryAccess for Memory68070 {
    fn iter(&mut self, addr: u32) -> MemoryIter {
        MemoryIter {
            cpu: self,
            next_addr: addr,
        }
    }

    fn get_byte(&mut self, addr: u32) -> u8 {
        if addr == 0x80002013 {
            0b0000_1110
        } else if addr < 0x50_0000 {
            self.ram[addr as usize]
        } else {
            0
        }
    }

    fn get_word(&mut self, addr: u32) -> u16 {
        if self.memory_swap < 4 {
            self.memory_swap += 1;
            (self.get_byte(addr + 0x40_0000) as u16) << 8 | self.get_byte(addr + 0x40_0001) as u16
        } else {
            (self.get_byte(addr) as u16) << 8 | self.get_byte(addr + 1) as u16
        }
    }

    fn get_long(&mut self, addr: u32) -> u32 {
        (self.get_word(addr) as u32) << 16 | self.get_word(addr + 2) as u32
    }

    fn set_byte(&mut self, addr: u32, value: u8) {
        if addr == 0x8000_2019 {
            print!("{}", value as char);
        } else if addr < 0x50_0000 {
            self.ram[addr as usize] = value;
        }
    }

    fn set_word(&mut self, addr: u32, value: u16) {
        self.set_byte(addr, (value >> 8) as u8);
        self.set_byte(addr + 1, value as u8);
    }

    fn set_long(&mut self, addr: u32, value: u32) {
        self.set_word(addr, (value >> 16) as u16);
        self.set_word(addr + 2, value as u16);
    }

    fn reset(&mut self) {}
}

fn main()
{
    let mut ram = Memory68070 {
        memory_swap: 0,
        ram: vec![0; MB].into_boxed_slice(),
    };
    let mut bios_file = File::open("cpudiag40.rom").expect("no cpudiag40.rom");
    match bios_file.read(&mut ram.ram[0x40_0000..]) {
        Ok(i) => println!("Successfully read {} bytes from cpudiag40.rom", i),
        Err(e) => panic!("Failed to read from cpudiag40.rom: {}", e),
    }

    let config = Config { stack: StackFrame::Stack68070 };
    let mut cpu = M68000::new(ram, config);

    // Execute 1 000 000 000 instructions
    for _ in 0..1_000_000_000 {
        cpu.interpreter();
    }
    // Check that the CPU loops at the correct end point
    // assert_eq!(cpu.pc, 0x0)
}
