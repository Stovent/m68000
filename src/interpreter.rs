use super::{M68000, MemoryAccess, SR_UPPER_MASK, CCR_MASK, SR_MASK};
use super::decoder::DECODER;
use super::instruction::Instruction;
use super::operands::{Direction, Size};
use super::status_register::StatusRegister;

impl<M: MemoryAccess> M68000<M> {
    pub fn interpreter(&mut self) {
        let pc = self.pc;
        let opcode = self.get_next_word();
        let isa = DECODER[opcode as usize];
        let entry = &Self::ISA_ENTRY[isa as usize];

        let mut iter = self.memory.iter(self.pc);
        let (operands, len) = (entry.decode)(opcode, &mut iter);
        self.pc += len as u32;

        let instruction = Instruction {
            opcode,
            pc,
            operands,
        };

        #[cfg(debug_assertions)]
        println!("{:#X} {}", pc, (entry.disassemble)(&instruction));

        (entry.execute)(self, &instruction);
    }

    pub(super) fn unknown_instruction(&mut self, _: &Instruction) -> usize {
        // TODO: trap
        0
    }

    pub(super) fn abcd(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn add(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn adda(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn addi(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn addq(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn addx(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn and(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn andi(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn andiccr(&mut self, inst: &Instruction) -> usize {
        let imm = inst.operands.immediate();

        self.sr &= SR_UPPER_MASK | imm;

        1
    }

    pub(super) fn andisr(&mut self, inst: &Instruction) -> usize {
        if self.sr.s {
            let imm = inst.operands.immediate();
            self.sr &= imm;
            1
        } else {
            // TODO: trap
            0
        }
    }

    pub(super) fn asm(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn asr(&mut self, inst: &Instruction) -> usize {
        let (rot, dir, size, mode, reg) = inst.operands.rotation_direction_size_mode_register();

        self.sr.v = false;
        self.sr.c = false;

        let shift_count = if mode == 1 {
            (self.d[rot as usize] % 64) as u8
        } else {
            if rot == 0 { 8 } else { rot }
        };

        let (mut data, mask) = if size == Size::Byte {
            (self.d[reg as usize] & 0x0000_00FF, 0x80u32)
        } else if size == Size::Word {
            (self.d[reg as usize] & 0x0000_FFFF, 0x8000)
        } else {
            (self.d[reg as usize], 0x8000_0000)
        };

        if dir == Direction::Left {
            for _ in 0..shift_count {
                let sign = data & mask;
                data <<= 1;
                self.sr.x = sign != 0;
                self.sr.c = sign != 0;
                if sign ^ data & mask != 0 {
                    self.sr.v = true;
                }
            }
        } else {
            let sign = data & mask;
            for _ in 0..shift_count {
                let bit = data & 1;
                data >>= 1;
                data |= sign;
                self.sr.x = bit != 0;
                self.sr.c = bit != 0;
            }
        }

        self.sr.n = data & mask != 0;

        if size == Size::Byte {
            self.d_byte(reg, data as u8);
            self.sr.z = data & 0x0000_00FF == 0;
        } else if size == Size::Word {
            self.d_word(reg, data as u16);
            self.sr.z = data & 0x0000_FFFF == 0;
        } else {
            self.d[reg as usize] = data;
            self.sr.z = data == 0;
        }

        1
    }

    pub(super) fn bcc(&mut self, inst: &Instruction) -> usize {
        let (condition, displacement) = inst.operands.condition_displacement();

        if StatusRegister::CONDITIONS[condition as usize](self.sr) {
            self.pc = inst.pc + 2 + displacement as u32;
        }

        1
    }

    pub(super) fn bchg(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn bclr(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn bra(&mut self, inst: &Instruction) -> usize {
        let disp = inst.operands.displacement();

        self.pc = inst.pc + 2 + disp as u32;

        1
    }

    pub(super) fn bset(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn bsr(&mut self, inst: &Instruction) -> usize {
        let disp = inst.operands.displacement();

        self.push_long(self.pc);
        self.pc += disp as u32;

        1
    }

    pub(super) fn btst(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn chk(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn clr(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn cmp(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn cmpa(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn cmpi(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn cmpm(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn dbcc(&mut self, inst: &Instruction) -> usize {
        let (cc, reg, disp) = inst.operands.condition_register_disp();

        if !StatusRegister::CONDITIONS[cc as usize](self.sr) {
            let counter = self.d[reg as usize] as i16 - 1;
            self.d_word(reg, counter as u16);

            if counter != -1 {
                self.pc = inst.pc + 2 + disp as u32;
            }
        }

        1
    }

    pub(super) fn divs(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn divu(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn eor(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn eori(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn eoriccr(&mut self, inst: &Instruction) -> usize {
        let imm = inst.operands.immediate();

        self.sr ^= imm;

        1
    }

    pub(super) fn eorisr(&mut self, inst: &Instruction) -> usize {
        if self.sr.s {
            let imm = inst.operands.immediate();
            self.sr ^= imm;
            1
        } else {
            // TODO: trap
            0
        }
    }

    pub(super) fn exg(&mut self, inst: &Instruction) -> usize {
        let (rx, mode, ry) = inst.operands.register_opmode_register();

        if mode == 0b01000 {
            self.d.swap(rx as usize, ry as usize);
        } else if mode == 0b01001 {
            // TODO: change to std::mem::swap when new borrow checker is available
            let y = self.a(ry);
            *self.a_mut(ry) = self.a(rx);
            *self.a_mut(rx) = y;
        } else {
            let y = self.a(ry);
            *self.a_mut(ry) = self.d[rx as usize];
            self.d[rx as usize] = y;
        }

        1
    }

    pub(super) fn ext(&mut self, inst: &Instruction) -> usize {
        let (mode, reg) = inst.operands.opmode_register();

        if mode == 0b010 {
            let d = self.d[reg as usize] as i8 as u16;
            self.d_word(reg, d);
        } else {
            self.d[reg as usize] = self.d[reg as usize] as i16 as u32;
        }

        self.sr.n = self.d[reg as usize] & 0x8000_0000 != 0;
        self.sr.z = self.d[reg as usize] == 0;
        self.sr.v = false;
        self.sr.c = false;

        1
    }

    pub(super) fn illegal(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn jmp(&mut self, inst: &Instruction) -> usize {
        let ea = inst.operands.effective_address();

        self.pc = self.get_effective_address(ea, inst.pc + 2).unwrap();

        1
    }

    pub(super) fn jsr(&mut self, inst: &Instruction) -> usize {
        let ea = inst.operands.effective_address();

        self.push_long(self.pc);
        self.pc = self.get_effective_address(ea, inst.pc + 2).unwrap();

        1
    }

    pub(super) fn lea(&mut self, inst: &Instruction) -> usize {
        let (reg, ea) = inst.operands.register_effective_address();

        *self.a_mut(reg) = self.get_effective_address(ea, inst.pc + 2).unwrap();

        1
    }

    pub(super) fn link(&mut self, inst: &Instruction) -> usize {
        let (reg, disp) = inst.operands.register_disp();

        self.push_long(self.a(reg));
        *self.a_mut(reg) = self.sp();
        *self.sp_mut() += disp as u32;

        1
    }

    pub(super) fn lsm(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn lsr(&mut self, inst: &Instruction) -> usize {
        let (rot, dir, size, mode, reg) = inst.operands.rotation_direction_size_mode_register();

        self.sr.v = false;
        self.sr.c = false;

        let shift_count = if mode == 1 {
            (self.d[rot as usize] % 64) as u8
        } else {
            if rot == 0 { 8 } else { rot }
        };

        let (mut data, mask) = if size == Size::Byte {
            (self.d[reg as usize] & 0x0000_00FF, 0x80u32)
        } else if size == Size::Word {
            (self.d[reg as usize] & 0x0000_FFFF, 0x8000)
        } else {
            (self.d[reg as usize], 0x8000_0000)
        };

        if dir == Direction::Left {
            for _ in 0..shift_count {
                let sign = data & mask;
                data <<= 1;
                self.sr.x = sign != 0;
                self.sr.c = sign != 0;
                if sign ^ data & mask != 0 {
                    self.sr.v = true;
                }
            }
        } else {
            for _ in 0..shift_count {
                let bit = data & 1;
                data >>= 1;
                self.sr.x = bit != 0;
                self.sr.c = bit != 0;
            }
        }

        self.sr.n = data & mask != 0;

        if size == Size::Byte {
            self.d_byte(reg, data as u8);
            self.sr.z = data & 0x0000_00FF == 0;
        } else if size == Size::Word {
            self.d_word(reg, data as u16);
            self.sr.z = data & 0x0000_FFFF == 0;
        } else {
            self.d[reg as usize] = data;
            self.sr.z = data == 0;
        }

        1
    }

    pub(super) fn r#move(&mut self, inst: &Instruction) -> usize {
        let (size, dst, src) = inst.operands.size_effective_address_effective_address();

        if size.byte() {
            let d = self.get_byte(src, inst.pc + 2);
            self.set_byte(dst, inst.pc + 2, d);
        } else if size.word() {
            let d = self.get_word(src, inst.pc + 2);
            self.set_word(dst, inst.pc + 2, d);
        } else {
            let d = self.get_long(src, inst.pc + 2);
            self.set_long(dst, inst.pc + 2, d);
        }

        1
    }

    pub(super) fn movea(&mut self, inst: &Instruction) -> usize {
        let (size, reg, ea) = inst.operands.size_register_effective_address();

        *self.a_mut(reg) = if size.word() {
            self.get_word(ea, inst.pc + 2) as i16 as u32
        } else {
            self.get_long(ea, inst.pc + 2)
        };

        1
    }

    pub(super) fn moveccr(&mut self, inst: &Instruction) -> usize {
        let ea = inst.operands.effective_address();

        let ccr = self.get_word(ea, inst.pc + 2);
        self.sr.set_ccr(ccr);

        1
    }

    pub(super) fn movefsr(&mut self, inst: &Instruction) -> usize {
        let ea = inst.operands.effective_address();

        self.set_word(ea, inst.pc + 2, self.sr.into());

        1
    }

    pub(super) fn movesr(&mut self, inst: &Instruction) -> usize {
        if self.sr.s {
            let ea = inst.operands.effective_address();
            let sr = self.get_word(ea, inst.pc + 2);
            self.sr = sr.into();
            1
        } else {
            // TODO: trap
            0
        }
    }

    pub(super) fn moveusp(&mut self, inst: &Instruction) -> usize {
        if self.sr.s {
            let (d, reg) = inst.operands.direction_register();
            if d == Direction::UspToRegister {
                *self.a_mut(reg) = self.usp;
            } else {
                self.usp = self.a(reg);
            }
            1
        } else {
            // TODO: trap
            0
        }
    }

    pub(super) fn movem(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn movep(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn moveq(&mut self, inst: &Instruction) -> usize {
        let (reg, data) = inst.operands.register_data();

        self.d[reg as usize] = data as u32;

        1
    }

    pub(super) fn muls(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn mulu(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn nbcd(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn neg(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn negx(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn nop(&mut self, _: &Instruction) -> usize {
        1
    }

    pub(super) fn not(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn or(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn ori(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn oriccr(&mut self, inst: &Instruction) -> usize {
        let imm = inst.operands.immediate();

        self.sr |= imm;

        1
    }

    pub(super) fn orisr(&mut self, inst: &Instruction) -> usize {
        if self.sr.s {
            let imm = inst.operands.immediate();
            self.sr |= imm;
            1
        } else {
            // TODO: trap
            0
        }
    }

    pub(super) fn pea(&mut self, inst: &Instruction) -> usize {
        let ea = inst.operands.effective_address();

        let addr = self.get_effective_address(ea, inst.pc + 2).unwrap();
        self.push_long(addr);

        1
    }

    pub(super) fn reset(&mut self, _: &Instruction) -> usize {
        if self.sr.s {
            self.memory.reset();
            1
        } else {
            // TODO: trap
            0
        }
    }

    pub(super) fn rom(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn ror(&mut self, inst: &Instruction) -> usize {
        let (rot, dir, size, mode, reg) = inst.operands.rotation_direction_size_mode_register();

        self.sr.v = false;
        self.sr.c = false;

        let shift_count = if mode == 1 {
            (self.d[rot as usize] % 64) as u8
        } else {
            if rot == 0 { 8 } else { rot }
        };

        let (mut data, mask) = if size == Size::Byte {
            (self.d[reg as usize] & 0x0000_00FF, 0x80u32)
        } else if size == Size::Word {
            (self.d[reg as usize] & 0x0000_FFFF, 0x8000)
        } else {
            (self.d[reg as usize], 0x8000_0000)
        };

        if dir == Direction::Left {
            for _ in 0..shift_count {
                let sign = data & mask;
                data <<= 1;
                if sign != 0 {
                    data |= 1;
                }
                self.sr.c = sign != 0;
            }
        } else {
            for _ in 0..shift_count {
                let bit = data & 1;
                data >>= 1;
                if bit != 0 {
                    data |= mask;
                }
                self.sr.c = bit != 0;
            }
        }

        self.sr.n = data & mask != 0;

        if size == Size::Byte {
            self.d_byte(reg, data as u8);
            self.sr.z = data & 0x0000_00FF == 0;
        } else if size == Size::Word {
            self.d_word(reg, data as u16);
            self.sr.z = data & 0x0000_FFFF == 0;
        } else {
            self.d[reg as usize] = data;
            self.sr.z = data == 0;
        }

        1
    }

    pub(super) fn roxm(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn roxr(&mut self, inst: &Instruction) -> usize {
        let (rot, dir, size, mode, reg) = inst.operands.rotation_direction_size_mode_register();

        self.sr.v = false;
        self.sr.c = self.sr.x;

        let shift_count = if mode == 1 {
            (self.d[rot as usize] % 64) as u8
        } else {
            if rot == 0 { 8 } else { rot }
        };

        let (mut data, mask) = if size == Size::Byte {
            (self.d[reg as usize] & 0x0000_00FF, 0x80u32)
        } else if size == Size::Word {
            (self.d[reg as usize] & 0x0000_FFFF, 0x8000)
        } else {
            (self.d[reg as usize], 0x8000_0000)
        };

        if dir == Direction::Left {
            for _ in 0..shift_count {
                let sign = data & mask;
                data <<= 1;
                data |= self.sr.x as u32;
                self.sr.x = sign != 0;
                self.sr.c = sign != 0;
            }
        } else {
            for _ in 0..shift_count {
                let bit = data & 1;
                data >>= 1;
                if self.sr.x {
                    data |= mask;
                }
                self.sr.x = bit != 0;
                self.sr.c = bit != 0;
            }
        }

        self.sr.n = data & mask != 0;

        if size == Size::Byte {
            self.d_byte(reg, data as u8);
            self.sr.z = data & 0x0000_00FF == 0;
        } else if size == Size::Word {
            self.d_word(reg, data as u16);
            self.sr.z = data & 0x0000_FFFF == 0;
        } else {
            self.d[reg as usize] = data;
            self.sr.z = data == 0;
        }

        1
    }

    pub(super) fn rte(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn rtr(&mut self, _: &Instruction) -> usize {
        let ccr = self.pop_word();
        self.sr &= SR_UPPER_MASK;
        self.sr |= ccr & CCR_MASK;
        self.pc = self.pop_long();

        0
    }

    pub(super) fn rts(&mut self, _: &Instruction) -> usize {
        self.pc = self.pop_long();

        1
    }

    pub(super) fn sbcd(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn scc(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn stop(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn sub(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn suba(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn subi(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn subq(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn subx(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn swap(&mut self, inst: &Instruction) -> usize {
        let reg = inst.operands.register();

        let high = self.d[reg as usize] >> 16;
        self.d[reg as usize] <<= 16;
        self.d[reg as usize] |= high;

        self.sr.n = self.d[reg as usize] & 0x8000_0000 != 0;
        self.sr.z = self.d[reg as usize] == 0;
        self.sr.v = false;
        self.sr.c = false;

        1
    }

    pub(super) fn tas(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn trap(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn trapv(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn tst(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn unlk(&mut self, inst: &Instruction) -> usize {
        let reg = inst.operands.register();

        *self.sp_mut() = self.a(reg);
        *self.a_mut(reg) = self.pop_long();

        1
    }
}
