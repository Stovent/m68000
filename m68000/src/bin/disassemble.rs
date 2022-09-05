//! Little program to disassemble the given binary file for the specified range.
//!
//! Usage: ./disassembler.exe <input file> [-o <output file>] [-b <beginning pos>] [-e <ending position>]

use m68000::decoder::DECODER;
use m68000::instruction::Instruction;
use m68000::isa::IsaEntry;
use m68000::memory_access::MemoryAccess;

use std::borrow::BorrowMut;
use std::fs::File;
use std::io::{Read, Write};

struct Memory {
    data: Vec<u8>,
}

impl MemoryAccess for Memory {
    fn get_byte(&mut self, addr: u32) -> Option<u8> {
        Some(self.data[addr as usize])
    }

    fn get_word(&mut self, addr: u32) -> Option<u16> {
        Some((self.data[addr as usize] as u16) << 8 | self.data[addr as usize + 1] as u16)
    }

    fn set_byte(&mut self, _: u32, _: u8) -> Option<()> {
        Some(())
    }

    fn set_word(&mut self, _: u32, _: u16) -> Option<()> {
        Some(())
    }

    fn reset_instruction(&mut self) {}
}

fn main() {
    let mut args = std::env::args();
    let exec = args.next().unwrap();
    if args.len() < 1 || args.len() > 7 {
        println!("Disassembles the instructions in the given input binary file, starting and ending at the given locations.");
        println!("Outputs the instructions in the given output file, or on the standard output if the output file is not supplied or cannot be opened.");
        println!("Usage: {} <input file> [-o <output file>] [-b <beginning pos>] [-e <ending position>]", exec);
        std::process::exit(1);
    }

    let inname = args.next().unwrap();
    let mut infile = File::open(inname.clone()).unwrap_or_else(|e| panic!("Failed to open input file \"{}\": {}", inname, e));

    let mut outname = String::new();
    let mut beg = 0;
    let mut end = usize::MAX;

    while let Some(arg) = args.next() {
        match &arg[..] {
            "-o" => outname = args.next().expect("Expected output filename with parameter -o"),
            "-b" => beg = args.next().expect("Expected beginning position with parameter -b").parse().expect("Expected number for beginning position"),
            "-e" => end = args.next().expect("Expected ending position with parameter -e").parse().expect("Expected number for ending position"),
            _ => panic!("Unknown parameter \"{}\"", arg),
        }
    }

    let mut outfile = if let Ok(f) = File::create(outname) {
        Some(f)
    } else {
        None
    };

    let filelen = infile.metadata().unwrap().len() as usize;
    if end > filelen {
        end = filelen
    };

    let mut memory = Memory {
        data: Vec::new(),
    };
    infile.read_to_end(&mut memory.data).unwrap();

    let mut i = beg;
    while i < end {
        let (inst, len) = Instruction::from_memory(&mut memory.iter_u16(i as u32)).unwrap();

        let disassemble = IsaEntry::ISA_ENTRY[DECODER[inst.opcode as usize] as usize].disassemble;
        if let Some(outfile) = outfile.borrow_mut() {
            writeln!(outfile, "{:#X} {}", i - 2, disassemble(&inst)).unwrap();
        } else {
            println!("{:#X} {}", i - 2, disassemble(&inst));
        }
        i += 2 + len;
    }
}
