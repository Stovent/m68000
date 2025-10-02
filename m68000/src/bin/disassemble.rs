// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Little program to disassemble the given binary file for the specified range.
//!
//! Usage: `./disassembler.exe <input file> [-o <output file>] [-b <beginning pos>] [-e <ending position>]`

use m68000::decoder::DECODER;
use m68000::disassembler::DLUT;
use m68000::instruction::Instruction;
use m68000::memory_access::MemoryAccess;

use std::borrow::BorrowMut;
use std::fs::File;
use std::io::{Read, Write};

fn main() {
    let mut args = std::env::args();
    let exec = args.next().unwrap();
    if args.len() < 1 || args.len() > 7 {
        println!("Disassembles the instructions in the given input binary file, starting and ending at the given locations.");
        println!("Outputs the instructions in the given output file, or on the standard output if the output file is not supplied or cannot be opened.");
        println!("Usage: {exec} <input file> [-o <output file>] [-b <beginning pos>] [-e <ending position>]");
        std::process::exit(1);
    }

    let inname = args.next().unwrap();
    let mut infile = File::open(inname.clone()).unwrap_or_else(|e| panic!("Failed to open input file \"{inname}\": {e}"));

    let mut outname = String::new();
    let mut beg = 0;
    let mut end = usize::MAX;

    while let Some(arg) = args.next() {
        match &arg[..] {
            "-o" => outname = args.next().expect("Expected output filename with parameter -o"),
            "-b" => beg = args.next().expect("Expected beginning position with parameter -b").parse().expect("Expected number for beginning position"),
            "-e" => end = args.next().expect("Expected ending position with parameter -e").parse().expect("Expected number for ending position"),
            _ => panic!("Unknown parameter \"{arg}\""),
        }
    }

    let mut outfile = File::create(outname).ok();

    let filelen = infile.metadata().unwrap().len() as usize;
    if end > filelen {
        end = filelen
    };

    let mut data = Vec::new();
    infile.read_to_end(&mut data).unwrap();

    let mut i = beg;
    let mut iter = data.iter_u16(i as u32);
    while i < end {
        let inst = Instruction::from_memory(&mut iter).unwrap();

        let disassemble = DLUT[DECODER[inst.opcode as usize] as usize];
        if let Some(outfile) = outfile.borrow_mut() {
            writeln!(outfile, "{:#X} {}", i, disassemble(&inst)).unwrap();
        } else {
            println!("{:#X} {}", i, disassemble(&inst));
        }
        i = iter.next_addr as usize;
    }
}
