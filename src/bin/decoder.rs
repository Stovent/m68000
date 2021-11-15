//! Application dedicated to generate the decoder.rs file of m68000.
//!
//! The goal is to make a decoding look-up table as a const member, so it has to be generated before.
//!
//! Because I don't know if macros and templates could do the job for me, I instead do it with a dedicated program.

use m68000::isa::Isa;

use std::fs::File;
use std::io::Write;
use std::str;

const FILE_BEGIN: &[u8] = b"//! Instruction decoding module.

use crate::isa::{Isa, Isa::*};

/// Look up the Isa of the given opcode.
///
/// This is a look-up table. Use the raw opcode as the index in the array.
///
/// # Example
///
/// ```rs
/// let opcode = memory.get_word(instruction_address); // get the opcode
/// let isa = DECODER[opcode as usize];
/// ```
pub const DECODER: [Isa; 65536] = [";

const FILE_END: &[u8] = b"
];
";

fn main() {
    let mut file = File::create("decoder.rs").expect("Unable to create file decoder.rs");
    let mut opcodes = [Isa::Unknown; 65536];

    generate_isa(&mut opcodes);

    match file.write(FILE_BEGIN) {
        Err(e) => panic!("Failed to write: {}", e),
        _ => (),
    }

    for i in 0..65536 {
        let str = if i % 16 == 0 {
            format!("\n    {:7?}, ", opcodes[i])
        } else {
            format!("{:7?}, ", opcodes[i])
        };

        match file.write(str.as_bytes()) {
            Err(e) => panic!("Failed to write index {}: {}", i, e),
            _ => (),
        }
    }

    match file.write(FILE_END) {
        Err(e) => panic!("Failed to write: {}", e),
        _ => (),
    }
}

fn generate_isa(opcodes: &mut [Isa; 65536]) {
    generate_opcodes(opcodes, "1100aaa10000bccc", &[&V0_7, &V0_1, &V0_7], Isa::Abcd);

    generate_opcodes(opcodes, "1101aaabbbcccddd", &[&V0_7, &V0_2, &V0__2_6, &V0_7], Isa::Add);
    generate_opcodes(opcodes, "1101aaabbb001ddd", &[&V0_7, &V1_2, &V0_7], Isa::Add);
    generate_opcodes(opcodes, "1101aaabbb111ddd", &[&V0_7, &V0_2, &V0_4], Isa::Add);
    generate_opcodes(opcodes, "1101aaabbbcccddd", &[&V0_7, &V4_6, &V2_6, &V0_7], Isa::Add);
    generate_opcodes(opcodes, "1101aaabbb111ddd", &[&V0_7, &V4_6, &V0_1], Isa::Add);

    generate_opcodes(opcodes, "1101aaab11cccddd", &[&V0_7, &V0_1, &V0_6, &V0_7], Isa::Adda);
    generate_opcodes(opcodes, "1101aaab11111ddd", &[&V0_7, &V0_1, &V0_4], Isa::Adda);

    generate_opcodes(opcodes, "00000110aabbbccc", &[&V0_2, &V0__2_6, &V0_7], Isa::Addi);
    generate_opcodes(opcodes, "00000110aa111ccc", &[&V0_2, &V0_1], Isa::Addi);

    generate_opcodes(opcodes, "0101aaa0bbcccddd", &[&V0_7, &V0_2, &V0__2_6, &V0_7], Isa::Addq);
    generate_opcodes(opcodes, "0101aaa0bb111ddd", &[&V0_7, &V0_2, &V0_1], Isa::Addq);
    generate_opcodes(opcodes, "0101aaa0bb001ddd", &[&V0_7, &V1_2, &V0_7], Isa::Addq);

    generate_opcodes(opcodes, "1101aaa1bb00cddd", &[&V0_7, &V0_2, &V0_1, &V0_7], Isa::Addx);

    generate_opcodes(opcodes, "1100aaa0bbcccddd", &[&V0_7, &V0_2, &V0__2_6, &V0_7], Isa::And);
    generate_opcodes(opcodes, "1100aaa0bb111ddd", &[&V0_7, &V0_2, &V0_4], Isa::And);
    generate_opcodes(opcodes, "1100aaa1bbcccddd", &[&V0_7, &V0_2, &V2_6, &V0_7], Isa::And);
    generate_opcodes(opcodes, "1100aaa1bb111ddd", &[&V0_7, &V0_2, &V0_1], Isa::And);

    generate_opcodes(opcodes, "00000010aabbbccc", &[&V0_2, &V0__2_6, &V0_7], Isa::Andi);
    generate_opcodes(opcodes, "00000010aa111ccc", &[&V0_2, &V0_1], Isa::Andi);

    opcodes[0x023C] = Isa::Andiccr;

    opcodes[0x027C] = Isa::Andisr;

    generate_opcodes(opcodes, "1110000a11bbbccc", &[&V0_1, &V2_6, &V0_7], Isa::Asm);
    generate_opcodes(opcodes, "1110000a11111ccc", &[&V0_1, &V0_1], Isa::Asm);

    generate_opcodes(opcodes, "1110aaabccd00eee", &[&V0_7, &V0_1, &V0_2, &V0_1, &V0_7], Isa::Asr);

    generate_opcodes(opcodes, "0110aaaabbbbbbbb", &[&V2_15, &VBYTE], Isa::Bcc);

    generate_opcodes(opcodes, "0000aaa101bbbccc", &[&V0_7, &V0__2_6, &V0_7], Isa::Bchg);
    generate_opcodes(opcodes, "0000aaa101111ccc", &[&V0_7, &V0_1], Isa::Bchg);
    generate_opcodes(opcodes, "0000100001aaabbb", &[&V0__2_6, &V0_7], Isa::Bchg);
    opcodes[0x0878] = Isa::Bchg;
    opcodes[0x0879] = Isa::Bchg;

    generate_opcodes(opcodes, "0000aaa110bbbccc", &[&V0_7, &V0__2_6, &V0_7], Isa::Bclr);
    generate_opcodes(opcodes, "0000aaa110111ccc", &[&V0_7, &V0_1], Isa::Bclr);
    generate_opcodes(opcodes, "0000100010aaabbb", &[&V0__2_6, &V0_7], Isa::Bclr);
    opcodes[0x08B8] = Isa::Bclr;
    opcodes[0x08B9] = Isa::Bclr;

    generate_opcodes(opcodes, "01100000aaaaaaaa", &[&VBYTE], Isa::Bra);

    generate_opcodes(opcodes, "0000aaa111bbbccc", &[&V0_7, &V0__2_6, &V0_7], Isa::Bset);
    generate_opcodes(opcodes, "0000aaa111111ccc", &[&V0_7, &V0_1], Isa::Bset);
    generate_opcodes(opcodes, "0000100011aaabbb", &[&V0__2_6, &V0_7], Isa::Bset);
    opcodes[0x08F8] = Isa::Bset;
    opcodes[0x08F9] = Isa::Bset;

    generate_opcodes(opcodes, "01100001aaaaaaaa", &[&VBYTE], Isa::Bsr);

    generate_opcodes(opcodes, "0000aaa100bbbccc", &[&V0_7, &V0__2_6, &V0_7], Isa::Btst);
    generate_opcodes(opcodes, "0000aaa100111ccc", &[&V0_7, &V0_4], Isa::Btst);
    generate_opcodes(opcodes, "0000100000aaabbb", &[&V0__2_6, &V0_7], Isa::Btst);
    opcodes[0x0838] = Isa::Btst;
    opcodes[0x0839] = Isa::Btst;
    opcodes[0x083A] = Isa::Btst;
    opcodes[0x083B] = Isa::Btst;

    generate_opcodes(opcodes, "0100aaa110bbbccc", &[&V0_7, &V0__2_6, &V0_7], Isa::Chk);
    generate_opcodes(opcodes, "0100aaa110111ccc", &[&V0_7, &V0_4], Isa::Chk);

    generate_opcodes(opcodes, "01000010aabbbccc", &[&V0_2, &V0__2_6, &V0_7], Isa::Clr);
    generate_opcodes(opcodes, "01000010aa111ccc", &[&V0_2, &V0_1], Isa::Clr);

    generate_opcodes(opcodes, "1011aaa000cccddd", &[&V0_7, &V0__2_6, &V0_7], Isa::Cmp);
    generate_opcodes(opcodes, "1011aaa000111ddd", &[&V0_7, &V0_4], Isa::Cmp);
    generate_opcodes(opcodes, "1011aaa0bbcccddd", &[&V0_7, &V1_2, &V0_6, &V0_7], Isa::Cmp);
    generate_opcodes(opcodes, "1011aaa0bb111ddd", &[&V0_7, &V1_2, &V0_4], Isa::Cmp);

    generate_opcodes(opcodes, "1011aaab11cccddd", &[&V0_7, &V0_1, &V0_6, &V0_7], Isa::Cmpa);
    generate_opcodes(opcodes, "1011aaab11111ddd", &[&V0_7, &V0_1, &V0_4], Isa::Cmpa);

    generate_opcodes(opcodes, "00001100aabbbccc", &[&V0_2, &V0__2_6, &V0_7], Isa::Cmpi);
    generate_opcodes(opcodes, "00001100aa111ccc", &[&V0_2, &V0_1], Isa::Cmpi);

    generate_opcodes(opcodes, "1011aaa1bb001ccc", &[&V0_7, &V0_2, &V0_7], Isa::Cmpm);

    generate_opcodes(opcodes, "0101aaaa11001bbb", &[&V0_15, &V0_7], Isa::Dbcc);

    generate_opcodes(opcodes, "1000aaa111bbbccc", &[&V0_7, &V0__2_6, &V0_7], Isa::Divs);
    generate_opcodes(opcodes, "1000aaa111111ccc", &[&V0_7, &V0_4], Isa::Divs);

    generate_opcodes(opcodes, "1000aaa011bbbccc", &[&V0_7, &V0__2_6, &V0_7], Isa::Divu);
    generate_opcodes(opcodes, "1000aaa011111ccc", &[&V0_7, &V0_4], Isa::Divu);

    generate_opcodes(opcodes, "1011aaa1bbcccddd", &[&V0_7, &V0_2, &V0__2_6, &V0_7], Isa::Eor);
    generate_opcodes(opcodes, "1011aaa1bb111ddd", &[&V0_7, &V0_2, &V0_1], Isa::Eor);

    generate_opcodes(opcodes, "00001010aabbbccc", &[&V0_2, &V0__2_6, &V0_7], Isa::Eori);
    generate_opcodes(opcodes, "00001010aa111ccc", &[&V0_2, &V0_1], Isa::Eori);

    opcodes[0x0A3C] = Isa::Eoriccr;

    opcodes[0x0A7C] = Isa::Eorisr;

    generate_opcodes(opcodes, "1100aaa1bbbbbccc", &[&V0_7, &V8_9__17, &V0_7], Isa::Exg);

    generate_opcodes(opcodes, "0100100aaa000bbb", &[&V2_3, &V0_7], Isa::Ext);

    opcodes[0x4AFC] = Isa::Illegal;

    generate_opcodes(opcodes, "0100111011aaabbb", &[&V2__5_6, &V0_7], Isa::Jmp);
    generate_opcodes(opcodes, "0100111011111bbb", &[&V0_3], Isa::Jmp);

    generate_opcodes(opcodes, "0100111010aaabbb", &[&V2__5_6, &V0_7], Isa::Jsr);
    generate_opcodes(opcodes, "0100111010111bbb", &[&V0_3], Isa::Jsr);

    generate_opcodes(opcodes, "0100aaa111bbbccc", &[&V0_7, &V2__5_6, &V0_7], Isa::Lea);
    generate_opcodes(opcodes, "0100aaa111111ccc", &[&V0_7, &V0_3], Isa::Lea);

    generate_opcodes(opcodes, "0100111001010aaa", &[&V0_7], Isa::Link);

    generate_opcodes(opcodes, "1110001a11bbbccc", &[&V0_1, &V2_6, &V0_7], Isa::Lsm);
    generate_opcodes(opcodes, "1110001a11111ccc", &[&V0_1, &V0_1], Isa::Lsm);

    generate_opcodes(opcodes, "1110aaabccd01eee", &[&V0_7, &V0_1, &V0_2, &V0_1, &V0_7], Isa::Lsr);

    generate_opcodes(opcodes, "00aabbbcccdddeee", &[&V1_3, &V0_7, &V0__2_6, &V0__2_6, &V0_7], Isa::Move);
    generate_opcodes(opcodes, "00aabbb111dddeee", &[&V1_3, &V0_1, &V0__2_6, &V0_7], Isa::Move);
    generate_opcodes(opcodes, "00aabbbccc111eee", &[&V1_3, &V0_7, &V0__2_6, &V0_4], Isa::Move);
    generate_opcodes(opcodes, "00aabbb111111eee", &[&V1_3, &V0_1, &V0_4], Isa::Move);
    generate_opcodes(opcodes, "00aabbbccc001eee", &[&V2_3, &V0_7, &V0__2_6, &V0_7], Isa::Move);
    generate_opcodes(opcodes, "00aabbb111001eee", &[&V2_3, &V0_1, &V0_7], Isa::Move);

    generate_opcodes(opcodes, "001abbb001cccddd", &[&V0_1, &V0_7, &V0_6, &V0_7], Isa::Movea);
    generate_opcodes(opcodes, "001abbb001111ddd", &[&V0_1, &V0_7, &V0_4], Isa::Movea);

    generate_opcodes(opcodes, "0100010011aaabbb", &[&V0__2_6, &V0_7], Isa::Moveccr);
    generate_opcodes(opcodes, "0100010011111bbb", &[&V0_4], Isa::Moveccr);

    generate_opcodes(opcodes, "0100000011aaabbb", &[&V0__2_6, &V0_7], Isa::Movefsr);
    opcodes[0x40F8] = Isa::Movefsr;
    opcodes[0x40F9] = Isa::Movefsr;

    generate_opcodes(opcodes, "0100011011aaabbb", &[&V0__2_6, &V0_7], Isa::Movesr);
    generate_opcodes(opcodes, "0100011011111bbb", &[&V0_4], Isa::Movesr);

    generate_opcodes(opcodes, "010011100110abbb", &[&V0_1, &V0_7], Isa::Moveusp);

    generate_opcodes(opcodes, "010010001bcccddd", &[&V0_1, &V2_6, &V0_7], Isa::Movem);
    generate_opcodes(opcodes, "010010001b111ddd", &[&V0_1, &V0_1], Isa::Movem);
    generate_opcodes(opcodes, "010011001bcccddd", &[&V0_1, &V2_6, &V0_7], Isa::Movem);
    generate_opcodes(opcodes, "010011001b111ddd", &[&V0_1, &V0_3], Isa::Movem);

    generate_opcodes(opcodes, "0000aaabbb001ccc", &[&V0_7, &V4_7, &V0_7], Isa::Movep);

    generate_opcodes(opcodes, "0111aaa0bbbbbbbb", &[&V0_7, &VBYTE], Isa::Moveq);

    generate_opcodes(opcodes, "1100aaa111bbbccc", &[&V0_7, &V0__2_6, &V0_7], Isa::Muls);
    generate_opcodes(opcodes, "1100aaa111111ccc", &[&V0_7, &V0_4], Isa::Muls);

    generate_opcodes(opcodes, "1100aaa011bbbccc", &[&V0_7, &V0__2_6, &V0_7], Isa::Mulu);
    generate_opcodes(opcodes, "1100aaa011111ccc", &[&V0_7, &V0_4], Isa::Mulu);

    generate_opcodes(opcodes, "0100100000aaabbb", &[&V0__2_6, &V0_7], Isa::Nbcd);
    opcodes[0x4838] = Isa::Nbcd;
    opcodes[0x4839] = Isa::Nbcd;

    generate_opcodes(opcodes, "01000100aabbbccc", &[&V0_2, &V0__2_6, &V0_7], Isa::Neg);
    generate_opcodes(opcodes, "01000100aa111ccc", &[&V0_2, &V0_1], Isa::Neg);

    generate_opcodes(opcodes, "01000000aabbbccc", &[&V0_2, &V0__2_6, &V0_7], Isa::Negx);
    generate_opcodes(opcodes, "01000000aa111ccc", &[&V0_2, &V0_1], Isa::Negx);

    opcodes[0x4E71] = Isa::Nop;

    generate_opcodes(opcodes, "01000110aabbbccc", &[&V0_2, &V0__2_6, &V0_7], Isa::Not);
    generate_opcodes(opcodes, "01000110aa111ccc", &[&V0_2, &V0_1], Isa::Not);

    generate_opcodes(opcodes, "1000aaa0bbcccddd", &[&V0_7, &V0_2, &V0__2_6, &V0_7], Isa::Or);
    generate_opcodes(opcodes, "1000aaa0bb111ddd", &[&V0_7, &V0_2, &V0_4], Isa::Or);
    generate_opcodes(opcodes, "1000aaa1bbcccddd", &[&V0_7, &V0_2, &V2_6, &V0_7], Isa::Or);
    generate_opcodes(opcodes, "1000aaa1bb111ddd", &[&V0_7, &V0_2, &V0_1], Isa::Or);

    generate_opcodes(opcodes, "00000000aabbbccc", &[&V0_2, &V0__2_6, &V0_7], Isa::Ori);
    generate_opcodes(opcodes, "00000000aa111ccc", &[&V0_2, &V0_1], Isa::Ori);

    opcodes[0x003C] = Isa::Oriccr;

    opcodes[0x007C] = Isa::Orisr;

    generate_opcodes(opcodes, "0100100001aaabbb", &[&V2__5_6, &V0_7], Isa::Pea);
    generate_opcodes(opcodes, "0100100001111bbb", &[&V0_3], Isa::Pea);

    opcodes[0x4E70] = Isa::Reset;

    generate_opcodes(opcodes, "1110011a11bbbccc", &[&V0_1, &V2_6, &V0_7], Isa::Rom);
    generate_opcodes(opcodes, "1110011a11111ccc", &[&V0_1, &V0_1 ], Isa::Rom);

    generate_opcodes(opcodes, "1110aaabccd11eee", &[&V0_7, &V0_1, &V0_2, &V0_1, &V0_7], Isa::Ror);

    generate_opcodes(opcodes, "1110010a11bbbccc", &[&V0_1, &V2_6, &V0_7], Isa::Roxm);
    generate_opcodes(opcodes, "1110010a11111ccc", &[&V0_1, &V0_1 ], Isa::Roxm);

    generate_opcodes(opcodes, "1110aaabccd10eee", &[&V0_7, &V0_1, &V0_2, &V0_1, &V0_7], Isa::Roxr);

    opcodes[0x4E73] = Isa::Rte;

    opcodes[0x4E77] = Isa::Rtr;

    opcodes[0x4E75] = Isa::Rts;

    generate_opcodes(opcodes, "1000aaa10000bccc", &[&V0_7, &V0_1, &V0_7], Isa::Sbcd);

    generate_opcodes(opcodes, "0101aaaa11bbbccc", &[&V0_15, &V0__2_6, &V0_7], Isa::Scc);
    generate_opcodes(opcodes, "0101aaaa11111ccc", &[&V0_15, &V0_1], Isa::Scc);

    opcodes[0x4E72] = Isa::Stop;

    generate_opcodes(opcodes, "1001aaabbbcccddd", &[&V0_7, &V0_2, &V0__2_6, &V0_7], Isa::Sub);
    generate_opcodes(opcodes, "1001aaabbb001ddd", &[&V0_7, &V1_2, &V0_7], Isa::Sub);
    generate_opcodes(opcodes, "1001aaabbb111ddd", &[&V0_7, &V0_2, &V0_4 ], Isa::Sub);
    generate_opcodes(opcodes, "1001aaabbbcccddd", &[&V0_7, &V4_6, &V2_6, &V0_7], Isa::Sub);
    generate_opcodes(opcodes, "1001aaabbb111ddd", &[&V0_7, &V4_6, &V0_1 ], Isa::Sub);

    generate_opcodes(opcodes, "1001aaab11cccddd", &[&V0_7, &V0_1, &V0_6, &V0_7], Isa::Suba);
    generate_opcodes(opcodes, "1001aaab11111ddd", &[&V0_7, &V0_1, &V0_4 ], Isa::Suba);

    generate_opcodes(opcodes, "00000100aabbbccc", &[&V0_2, &V0__2_6, &V0_7], Isa::Subi);
    generate_opcodes(opcodes, "00000100aa111ccc", &[&V0_2, &V0_1], Isa::Subi);

    generate_opcodes(opcodes, "0101aaa1bbcccddd", &[&V0_7, &V0_2, &V0__2_6, &V0_7], Isa::Subq);
    generate_opcodes(opcodes, "0101aaa1bb111ddd", &[&V0_7, &V0_2, &V0_1], Isa::Subq);
    generate_opcodes(opcodes, "0101aaa1bb001ddd", &[&V0_7, &V1_2, &V0_7], Isa::Subq);

    generate_opcodes(opcodes, "1001aaa1bb00cddd", &[&V0_7, &V0_2, &V0_1, &V0_7], Isa::Subx);

    generate_opcodes(opcodes, "0100100001000aaa", &[&V0_7], Isa::Swap);

    generate_opcodes(opcodes, "0100101011aaabbb", &[&V0__2_6, &V0_7], Isa::Tas);
    opcodes[0x4AF8] = Isa::Tas;
    opcodes[0x4AF9] = Isa::Tas;

    generate_opcodes(opcodes, "010011100100aaaa", &[&V0_15], Isa::Trap);

    opcodes[0x4E76] = Isa::Trapv;

    generate_opcodes(opcodes, "01001010aabbbccc", &[&V0_2, &V0__2_6, &V0_7], Isa::Tst);
    generate_opcodes(opcodes, "01001010aa111ccc", &[&V0_2, &V0_1], Isa::Tst);

    generate_opcodes(opcodes, "0100111001011aaa", &[&V0_7], Isa::Unlk);
}

/// Generates opcodes from the given format and replaces all variables by the values in ``values``.
///
/// e.g. with format = "0101aabb" and values = [[0, 1], [1, 2]]
/// will generate the binary strings ``01010001``, ``01010010``, ``01010101``, ``01010110``.
///
/// Then the binary string is converted back to integer, and stores ``isa`` in ``opcodes``
/// at every index generated by the function.
fn generate_opcodes(opcodes: &mut [Isa; 65536], format: &str, values: &[&[u8]], isa: Isa) {
    if values.len() == 1 {
        let mut pos = 0;
        let mut len = 0;
        let mut i = 0usize;
        while i < 16 {
            let char = format.as_bytes()[i] as char;
            if char != '0' && char != '1' {
                pos = i;
                while i < 16 && format.as_bytes()[i] as char == char {
                    len += 1;
                    i += 1;
                }
            } else {
                i += 1;
            }
        }

        for value in values[0].iter() {
            let left = str::from_utf8(&format.as_bytes()[..pos]).unwrap();
            let bin_str = int_to_bin_string(*value as usize, len);
            let index = if pos + len < 16 {
                let right = str::from_utf8(&format.as_bytes()[pos + len..]).unwrap();
                format!("{}{}{}", left, bin_str, right)
            } else {
                format!("{}{}", left, bin_str)
            };
            let index = bin_string_to_int(&index);
            assert_eq!(opcodes[index], Isa::Unknown, "Overwriting an existing opcode: {:#X}", index);
            opcodes[index] = isa;
        }
    } else {
        let mut pos = 0;
        let mut len = 0;
        let mut i = 0usize;
        while i < 16 {
            let char = format.as_bytes()[i] as char;
            if char != '0' && char != '1' {
                pos = i;
                while format.as_bytes()[i] as char == char {
                    len += 1;
                    i += 1;
                }
                break;
            } else {
                i += 1;
            }
        }

        for value in values[0].iter() {
            let left = str::from_utf8(&format.as_bytes()[..pos]).unwrap();
            let bin_str = int_to_bin_string(*value as usize, len);
            let index = if pos + len < 16 {
                let right = str::from_utf8(&format.as_bytes()[pos + len..]).unwrap();
                format!("{}{}{}", left, bin_str, right)
            } else {
                format!("{}{}", left, bin_str)
            };
            generate_opcodes(opcodes, &index, &values[1..], isa);
        }
    }
}

/// Converts a string of 0s ans 1s to its integer value.
pub fn bin_string_to_int(s: &str) -> usize {
    let mut result = 0usize;
    let mut mask = 1usize;
    let mut pos = s.len() as isize - 1;

    while pos >= 0 {
        if s.as_bytes()[pos as usize] == '1' as u8 {
            result |= mask;
        }
        pos -= 1;
        mask <<= 1;
    }

    result
}

/// Converts an integer to a binary string, taking only the ``size`` first bits starting at the lsb.
pub fn int_to_bin_string(val: usize, mut size: usize) -> String {
    let mut str = String::default();
    let mut mask = 1 << size - 1;

    while size > 0 {
        size -= 1;
        if val & mask != 0 {
            str.push('1');
        } else {
            str.push('0');
        }
        mask >>= 1;
    }

    str
}

const V0_1: [u8; 2] = [0, 1];
const V0_2: [u8; 3] = [0, 1, 2];
const V0__2_6: [u8; 6] = [0, 2, 3, 4, 5, 6];
const V0_3: [u8; 4] = [0, 1, 2, 3];
const V0_4: [u8; 5] = [0, 1, 2, 3, 4];
const V0_6: [u8; 7] = [0, 1, 2, 3, 4, 5, 6];
const V0_7: [u8; 8] = [0, 1, 2, 3, 4, 5, 6, 7];
const V0_15: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
const V1_2: [u8; 2] = [1, 2];
const V1_3: [u8; 3] = [1, 2, 3];
const V2_3: [u8; 2] = [2, 3];
const V2__5_6: [u8; 3] = [2, 5, 6];
const V2_6: [u8; 5] = [2, 3, 4, 5, 6];
const V2_15: [u8; 14] = [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
const V4_6: [u8; 3] = [4, 5, 6];
const V4_7: [u8; 4] = [4, 5, 6, 7];
const V8_9__17: [u8; 3] = [8, 9, 17];
const VBYTE: [u8; 256] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39,
40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79,
80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119,
120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154, 155, 156, 157, 158, 159,
160, 161, 162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172, 173, 174, 175, 176, 177, 178, 179, 180, 181, 182, 183, 184, 185, 186, 187, 188, 189, 190, 191, 192, 193, 194, 195, 196, 197, 198, 199,
200, 201, 202, 203, 204, 205, 206, 207, 208, 209, 210, 211, 212, 213, 214, 215, 216, 217, 218, 219, 220, 221, 222, 223, 224, 225, 226, 227, 228, 229, 230, 231, 232, 233, 234, 235, 236, 237, 238, 239,
240, 241, 242, 243, 244, 245, 246, 247, 248, 249, 250, 251, 252, 253, 254, 255];
