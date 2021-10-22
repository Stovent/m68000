use super::{M68000, MemoryAccess, SR_UPPER_MASK, CCR_MASK};
use super::decoder::DECODER;
use super::instruction::Instruction;
use super::operands::{Direction, Size};
use super::status_register::StatusRegister;
use super::utils::bits;

impl<M: MemoryAccess> M68000<M> {
    pub fn interpreter(&mut self) {
        let pc = self.pc;
        let opcode = self.get_next_word();
        let isa = DECODER[opcode as usize];
        let entry = &Self::ISA_ENTRY[isa as usize];

        let mut iter = self.memory.iter(self.pc);
        let (operands, len) = (entry.decode)(opcode, &mut iter);
        self.pc += len as u32;

        let mut instruction = Instruction {
            opcode,
            pc,
            operands,
        };

        #[cfg(debug_assertions)]
        println!("{:#X} {}", pc, (entry.disassemble)(&mut instruction));

        (entry.execute)(self, &mut instruction);
    }

    pub(super) fn unknown_instruction(&mut self, _: &mut Instruction) -> usize {
        // TODO: trap
        0
    }

    pub(super) fn abcd(&mut self, inst: &mut Instruction) -> usize {
        0
    }

    pub(super) fn add(&mut self, inst: &mut Instruction) -> usize {
        0
    }

    pub(super) fn adda(&mut self, inst: &mut Instruction) -> usize {
        0
    }

    pub(super) fn addi(&mut self, inst: &mut Instruction) -> usize {
        let (size, ea, imm) = inst.operands.size_effective_address_immediate();

        if size.byte() {
            let data = self.get_byte(ea) as i8;
            let (res, v) = data.overflowing_add(imm as i8);
            let (_, c) = data.carrying_add(imm as i8, false);
            self.set_byte(ea, res as u8);

            self.sr.x = c;
            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        } else if size.word() {
            let data = self.get_word(ea) as i16;
            let (res, v) = data.overflowing_add(imm as i16);
            let (_, c) = data.carrying_add(imm as i16, false);
            self.set_word(ea, res as u16);

            self.sr.x = c;
            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        } else {
            let data = self.get_long(ea) as i32;
            let (res, v) = data.overflowing_add(imm as i32);
            let (_, c) = data.carrying_add(imm as i32, false);
            self.set_long(ea, res as u32);

            self.sr.x = c;
            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        }

        1
    }

    pub(super) fn addq(&mut self, inst: &mut Instruction) -> usize {
        0
    }

    pub(super) fn addx(&mut self, inst: &mut Instruction) -> usize {
        0
    }

    pub(super) fn and(&mut self, inst: &mut Instruction) -> usize {
        0
    }

    pub(super) fn andi(&mut self, inst: &mut Instruction) -> usize {
        let (size, ea, imm) = inst.operands.size_effective_address_immediate();

        if size.byte() {
            let data = self.get_byte(ea) & imm as u8;
            self.set_byte(ea, data);

            self.sr.n = data & 0x80 != 0;
            self.sr.z = data == 0;
        } else if size.word() {
            let data = self.get_word(ea) & imm as u16;
            self.set_word(ea, data);

            self.sr.n = data & 0x8000 != 0;
            self.sr.z = data == 0;
        } else {
            let data = self.get_long(ea) & imm;
            self.set_long(ea, data);

            self.sr.n = data & 0x8000_0000 != 0;
            self.sr.z = data == 0;
        }

        self.sr.v = false;
        self.sr.c = false;

        1
    }

    pub(super) fn andiccr(&mut self, inst: &mut Instruction) -> usize {
        let imm = inst.operands.immediate();

        self.sr &= SR_UPPER_MASK | imm;

        1
    }

    pub(super) fn andisr(&mut self, inst: &mut Instruction) -> usize {
        if self.sr.s {
            let imm = inst.operands.immediate();
            self.sr &= imm;
            1
        } else {
            // TODO: trap
            0
        }
    }

    pub(super) fn asm(&mut self, inst: &mut Instruction) -> usize {
        0
    }

    pub(super) fn asr(&mut self, inst: &mut Instruction) -> usize {
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

    pub(super) fn bcc(&mut self, inst: &mut Instruction) -> usize {
        let (condition, displacement) = inst.operands.condition_displacement();

        if StatusRegister::CONDITIONS[condition as usize](self.sr) {
            self.pc = inst.pc + 2 + displacement as u32;
        }

        1
    }

    pub(super) fn bchg(&mut self, inst: &mut Instruction) -> usize {
        let (ea, mut count) = inst.operands.effective_address_count();

        if bits(inst.opcode, 8, 8) != 0 {
            count = self.d[count as usize] as u8;
        }

        if ea.mode.drd() {
            count %= 32;
            self.sr.z = self.d[ea.reg as usize] & 1 << count == 0;
            self.d[ea.reg as usize] ^= 1 << count;
        } else {
            count %= 8;
            let mut data = self.get_byte(ea);
            self.sr.z = data & 1 << count == 0;
            data ^= 1 << count;
            self.set_byte(ea, data);
        }

        1
    }

    pub(super) fn bclr(&mut self, inst: &mut Instruction) -> usize {
        let (ea, mut count) = inst.operands.effective_address_count();

        if bits(inst.opcode, 8, 8) != 0 {
            count = self.d[count as usize] as u8;
        }

        if ea.mode.drd() {
            count %= 32;
            self.sr.z = self.d[ea.reg as usize] & 1 << count == 0;
            self.d[ea.reg as usize] &= !(1 << count);
        } else {
            count %= 8;
            let mut data = self.get_byte(ea);
            self.sr.z = data & 1 << count == 0;
            data &= !(1 << count);
            self.set_byte(ea, data);
        }

        1
    }

    pub(super) fn bra(&mut self, inst: &mut Instruction) -> usize {
        let disp = inst.operands.displacement();

        self.pc = inst.pc + 2 + disp as u32;

        1
    }

    pub(super) fn bset(&mut self, inst: &mut Instruction) -> usize {
        let (ea, mut count) = inst.operands.effective_address_count();

        if bits(inst.opcode, 8, 8) != 0 {
            count = self.d[count as usize] as u8;
        }

        if ea.mode.drd() {
            count %= 32;
            self.sr.z = self.d[ea.reg as usize] & 1 << count == 0;
            self.d[ea.reg as usize] |= 1 << count;
        } else {
            count %= 8;
            let mut data = self.get_byte(ea);
            self.sr.z = data & 1 << count == 0;
            data |= 1 << count;
            self.set_byte(ea, data);
        }

        1
    }

    pub(super) fn bsr(&mut self, inst: &mut Instruction) -> usize {
        let disp = inst.operands.displacement();

        self.push_long(self.pc);
        self.pc = inst.pc + 2 + disp as u32;

        1
    }

    pub(super) fn btst(&mut self, inst: &mut Instruction) -> usize {
        let (ea, mut count) = inst.operands.effective_address_count();

        if bits(inst.opcode, 8, 8) != 0 {
            count = self.d[count as usize] as u8;
        }

        if ea.mode.drd() {
            count %= 32;
            self.sr.z = self.d[ea.reg as usize] & 1 << count == 0;
        } else {
            count %= 8;
            let data = self.get_byte(ea);
            self.sr.z = data & 1 << count == 0;
        }

        1
    }

    pub(super) fn chk(&mut self, inst: &mut Instruction) -> usize {
        0
    }

    pub(super) fn clr(&mut self, inst: &mut Instruction) -> usize {
        0
    }

    pub(super) fn cmp(&mut self, inst: &mut Instruction) -> usize {
        let (reg, _, size, ea) = inst.operands.register_direction_size_effective_address();

        if size.byte() {
            let src = self.get_byte(ea) as i8;
            let dst = self.d[reg as usize] as i8;

            let (res, v) = dst.overflowing_sub(src);
            let (_, c) = dst.borrowing_sub(src, false);

            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        } else if size.word() {
            let src = self.get_word(ea) as i16;
            let dst = self.d[reg as usize] as i16;

            let (res, v) = dst.overflowing_sub(src);
            let (_, c) = dst.borrowing_sub(src, false);

            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        } else {
            let src = self.get_long(ea) as i32;
            let dst = self.d[reg as usize] as i32;

            let (res, v) = dst.overflowing_sub(src);
            let (_, c) = dst.borrowing_sub(src, false);

            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        }

        1
    }

    pub(super) fn cmpa(&mut self, inst: &mut Instruction) -> usize {
        let (reg, size, ea) = inst.operands.register_size_effective_address();

        let src = if size.word() {
            self.get_word(ea) as i16 as i32
        } else {
            self.get_long(ea) as i32
        };

        let (res, v) = (self.a(reg) as i32).overflowing_sub(src);
        let (_, c) = (self.a(reg) as i32).borrowing_sub(src, false);

        self.sr.n = res < 0;
        self.sr.z = res == 0;
        self.sr.v = v;
        self.sr.c = c;

        1
    }

    pub(super) fn cmpi(&mut self, inst: &mut Instruction) -> usize {
        let (size, ea, imm) = inst.operands.size_effective_address_immediate();

        if size.byte() {
            let data = self.get_byte(ea) as i8;
            let (res, v) = data.overflowing_sub(imm as i8);
            let (_, c) = data.borrowing_sub(imm as i8, false);

            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        } else if size.word() {
            let data = self.get_word(ea) as i16;
            let (res, v) = data.overflowing_sub(imm as i16);
            let (_, c) = data.borrowing_sub(imm as i16, false);

            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        } else {
            let data = self.get_long(ea) as i32;
            let (res, v) = data.overflowing_sub(imm as i32);
            let (_, c) = data.borrowing_sub(imm as i32, false);

            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        }

        1
    }

    pub(super) fn cmpm(&mut self, inst: &mut Instruction) -> usize {
        let (ax, size, ay) = inst.operands.register_size_register();

        if size.byte() {
            let ay = self.ariwpo(ay, size);
            let ax = self.ariwpo(ax, size);
            let src = self.memory.get_byte(ay) as i8;
            let dst = self.memory.get_byte(ax) as i8;

            let (res, v) = dst.overflowing_sub(src);
            let (_, c) = dst.borrowing_sub(src, false);

            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        } else if size.word() {
            let ay = self.ariwpo(ay, size);
            let ax = self.ariwpo(ax, size);
            let src = self.memory.get_word(ay) as i16;
            let dst = self.memory.get_word(ax) as i16;

            let (res, v) = dst.overflowing_sub(src);
            let (_, c) = dst.borrowing_sub(src, false);

            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        } else {
            let ay = self.ariwpo(ay, size);
            let ax = self.ariwpo(ax, size);
            let src = self.memory.get_long(ay) as i32;
            let dst = self.memory.get_long(ax) as i32;

            let (res, v) = dst.overflowing_sub(src);
            let (_, c) = dst.borrowing_sub(src, false);

            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        }

        1
    }

    pub(super) fn dbcc(&mut self, inst: &mut Instruction) -> usize {
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

    pub(super) fn divs(&mut self, inst: &mut Instruction) -> usize {
        0
    }

    pub(super) fn divu(&mut self, inst: &mut Instruction) -> usize {
        0
    }

    pub(super) fn eor(&mut self, inst: &mut Instruction) -> usize {
        0
    }

    pub(super) fn eori(&mut self, inst: &mut Instruction) -> usize {
        let (size, ea, imm) = inst.operands.size_effective_address_immediate();

        if size.byte() {
            let data = self.get_byte(ea) ^ imm as u8;
            self.set_byte(ea, data);

            self.sr.n = data & 0x80 != 0;
            self.sr.z = data == 0;
        } else if size.word() {
            let data = self.get_word(ea) ^ imm as u16;
            self.set_word(ea, data);

            self.sr.n = data & 0x8000 != 0;
            self.sr.z = data == 0;
        } else {
            let data = self.get_long(ea) ^ imm;
            self.set_long(ea, data);

            self.sr.n = data & 0x8000_0000 != 0;
            self.sr.z = data == 0;
        }

        self.sr.v = false;
        self.sr.c = false;

        1
    }

    pub(super) fn eoriccr(&mut self, inst: &mut Instruction) -> usize {
        let imm = inst.operands.immediate();

        self.sr ^= imm;

        1
    }

    pub(super) fn eorisr(&mut self, inst: &mut Instruction) -> usize {
        if self.sr.s {
            let imm = inst.operands.immediate();
            self.sr ^= imm;
            1
        } else {
            // TODO: trap
            0
        }
    }

    pub(super) fn exg(&mut self, inst: &mut Instruction) -> usize {
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

    pub(super) fn ext(&mut self, inst: &mut Instruction) -> usize {
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

    pub(super) fn illegal(&mut self, inst: &mut Instruction) -> usize {
        0
    }

    pub(super) fn jmp(&mut self, inst: &mut Instruction) -> usize {
        let ea = inst.operands.effective_address();

        self.pc = self.get_effective_address(ea).unwrap();

        1
    }

    pub(super) fn jsr(&mut self, inst: &mut Instruction) -> usize {
        let ea = inst.operands.effective_address();

        self.push_long(self.pc);
        self.pc = self.get_effective_address(ea).unwrap();

        1
    }

    pub(super) fn lea(&mut self, inst: &mut Instruction) -> usize {
        let (reg, ea) = inst.operands.register_effective_address();

        *self.a_mut(reg) = self.get_effective_address(ea).unwrap();

        1
    }

    pub(super) fn link(&mut self, inst: &mut Instruction) -> usize {
        let (reg, disp) = inst.operands.register_disp();

        self.push_long(self.a(reg));
        *self.a_mut(reg) = self.sp();
        *self.sp_mut() += disp as u32;

        1
    }

    pub(super) fn lsm(&mut self, inst: &mut Instruction) -> usize {
        0
    }

    pub(super) fn lsr(&mut self, inst: &mut Instruction) -> usize {
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

    pub(super) fn r#move(&mut self, inst: &mut Instruction) -> usize {
        let (size, dst, src) = inst.operands.size_effective_address_effective_address();

        if size.byte() {
            let d = self.get_byte(src);
            self.set_byte(dst, d);
            self.sr.n = d & 0x80 != 0;
            self.sr.z = d == 0;
        } else if size.word() {
            let d = self.get_word(src);
            self.set_word(dst, d);
            self.sr.n = d & 0x8000 != 0;
            self.sr.z = d == 0;
        } else {
            let d = self.get_long(src);
            self.set_long(dst, d);
            self.sr.n = d & 0x8000_0000 != 0;
            self.sr.z = d == 0;
        }

        self.sr.v = false;
        self.sr.c = false;

        1
    }

    pub(super) fn movea(&mut self, inst: &mut Instruction) -> usize {
        let (size, reg, ea) = inst.operands.size_register_effective_address();

        *self.a_mut(reg) = if size.word() {
            self.get_word(ea) as i16 as u32
        } else {
            self.get_long(ea)
        };

        1
    }

    pub(super) fn moveccr(&mut self, inst: &mut Instruction) -> usize {
        let ea = inst.operands.effective_address();

        let ccr = self.get_word(ea);
        self.sr.set_ccr(ccr);

        1
    }

    pub(super) fn movefsr(&mut self, inst: &mut Instruction) -> usize {
        let ea = inst.operands.effective_address();

        self.set_word(ea, self.sr.into());

        1
    }

    pub(super) fn movesr(&mut self, inst: &mut Instruction) -> usize {
        if self.sr.s {
            let ea = inst.operands.effective_address();
            let sr = self.get_word(ea);
            self.sr = sr.into();
            1
        } else {
            // TODO: trap
            0
        }
    }

    pub(super) fn moveusp(&mut self, inst: &mut Instruction) -> usize {
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

    pub(super) fn movem(&mut self, inst: &mut Instruction) -> usize {
        0
    }

    pub(super) fn movep(&mut self, inst: &mut Instruction) -> usize {
        let (data, dir, size, addr, disp) = inst.operands.register_direction_size_register_disp();

        let mut shift = if size.word() { 8 } else { 24 };
        let mut addr = self.a(addr) + disp as u32;

        if dir == Direction::RegisterToMemory {
            while shift >= 0 {
                let d = (self.d[data as usize] >> shift) as u8;
                self.memory.set_byte(addr, d);
                shift -= 8;
                addr += 2;
            }
        } else {
            if size.word() { self.d[data as usize] &= 0xFFFF_0000 } else { self.d[data as usize] = 0 }

            while shift >= 0 {
                let d = self.memory.get_byte(addr) as u32;
                self.d[data as usize] |= d << shift;
                shift -= 8;
                addr += 2;
            }
        }

        1
    }

    pub(super) fn moveq(&mut self, inst: &mut Instruction) -> usize {
        let (reg, data) = inst.operands.register_data();

        self.d[reg as usize] = data as u32;

        self.sr.n = data <  0;
        self.sr.z = data == 0;
        self.sr.v = false;
        self.sr.c = false;

        1
    }

    pub(super) fn muls(&mut self, inst: &mut Instruction) -> usize {
        0
    }

    pub(super) fn mulu(&mut self, inst: &mut Instruction) -> usize {
        0
    }

    pub(super) fn nbcd(&mut self, inst: &mut Instruction) -> usize {
        0
    }

    pub(super) fn neg(&mut self, inst: &mut Instruction) -> usize {
        0
    }

    pub(super) fn negx(&mut self, inst: &mut Instruction) -> usize {
        0
    }

    pub(super) fn nop(&mut self, _: &mut Instruction) -> usize {
        1
    }

    pub(super) fn not(&mut self, inst: &mut Instruction) -> usize {
        0
    }

    pub(super) fn or(&mut self, inst: &mut Instruction) -> usize {
        0
    }

    pub(super) fn ori(&mut self, inst: &mut Instruction) -> usize {
        let (size, ea, imm) = inst.operands.size_effective_address_immediate();

        if size.byte() {
            let data = self.get_byte(ea) | imm as u8;
            self.set_byte(ea, data);

            self.sr.n = data & 0x80 != 0;
            self.sr.z = data == 0;
        } else if size.word() {
            let data = self.get_word(ea) | imm as u16;
            self.set_word(ea, data);

            self.sr.n = data & 0x8000 != 0;
            self.sr.z = data == 0;
        } else {
            let data = self.get_long(ea) | imm;
            self.set_long(ea, data);

            self.sr.n = data & 0x8000_0000 != 0;
            self.sr.z = data == 0;
        }

        self.sr.v = false;
        self.sr.c = false;

        1
    }

    pub(super) fn oriccr(&mut self, inst: &mut Instruction) -> usize {
        let imm = inst.operands.immediate();

        self.sr |= imm;

        1
    }

    pub(super) fn orisr(&mut self, inst: &mut Instruction) -> usize {
        if self.sr.s {
            let imm = inst.operands.immediate();
            self.sr |= imm;
            1
        } else {
            // TODO: trap
            0
        }
    }

    pub(super) fn pea(&mut self, inst: &mut Instruction) -> usize {
        let ea = inst.operands.effective_address();

        let addr = self.get_effective_address(ea).unwrap();
        self.push_long(addr);

        1
    }

    pub(super) fn reset(&mut self, _: &mut Instruction) -> usize {
        if self.sr.s {
            self.memory.reset();
            1
        } else {
            // TODO: trap
            0
        }
    }

    pub(super) fn rom(&mut self, inst: &mut Instruction) -> usize {
        0
    }

    pub(super) fn ror(&mut self, inst: &mut Instruction) -> usize {
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

    pub(super) fn roxm(&mut self, inst: &mut Instruction) -> usize {
        0
    }

    pub(super) fn roxr(&mut self, inst: &mut Instruction) -> usize {
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

    pub(super) fn rte(&mut self, inst: &mut Instruction) -> usize {
        0
    }

    pub(super) fn rtr(&mut self, _: &mut Instruction) -> usize {
        let ccr = self.pop_word();
        self.sr &= SR_UPPER_MASK;
        self.sr |= ccr & CCR_MASK;
        self.pc = self.pop_long();

        0
    }

    pub(super) fn rts(&mut self, _: &mut Instruction) -> usize {
        self.pc = self.pop_long();

        1
    }

    pub(super) fn sbcd(&mut self, inst: &mut Instruction) -> usize {
        0
    }

    pub(super) fn scc(&mut self, inst: &mut Instruction) -> usize {
        0
    }

    pub(super) fn stop(&mut self, inst: &mut Instruction) -> usize {
        0
    }

    pub(super) fn sub(&mut self, inst: &mut Instruction) -> usize {
        0
    }

    pub(super) fn suba(&mut self, inst: &mut Instruction) -> usize {
        0
    }

    pub(super) fn subi(&mut self, inst: &mut Instruction) -> usize {
        let (size, ea, imm) = inst.operands.size_effective_address_immediate();

        if size.byte() {
            let data = self.get_byte(ea) as i8;
            let (res, v) = data.overflowing_sub(imm as i8);
            let (_, c) = data.borrowing_sub(imm as i8, false);
            self.set_byte(ea, res as u8);

            self.sr.x = c;
            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        } else if size.word() {
            let data = self.get_word(ea) as i16;
            let (res, v) = data.overflowing_sub(imm as i16);
            let (_, c) = data.borrowing_sub(imm as i16, false);
            self.set_word(ea, res as u16);

            self.sr.x = c;
            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        } else {
            let data = self.get_long(ea) as i32;
            let (res, v) = data.overflowing_sub(imm as i32);
            let (_, c) = data.borrowing_sub(imm as i32, false);
            self.set_long(ea, res as u32);

            self.sr.x = c;
            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        }

        1
    }

    pub(super) fn subq(&mut self, inst: &mut Instruction) -> usize {
        0
    }

    pub(super) fn subx(&mut self, inst: &mut Instruction) -> usize {
        0
    }

    pub(super) fn swap(&mut self, inst: &mut Instruction) -> usize {
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

    pub(super) fn tas(&mut self, inst: &mut Instruction) -> usize {
        let ea = inst.operands.effective_address();

        let mut data = self.get_byte(ea);

        self.sr.n = data & 0x80 != 0;
        self.sr.z = data == 0;
        self.sr.v = false;
        self.sr.c = false;

        data |= 0x80;
        self.set_byte(ea, data);

        1
    }

    pub(super) fn trap(&mut self, inst: &mut Instruction) -> usize {
        0
    }

    pub(super) fn trapv(&mut self, inst: &mut Instruction) -> usize {
        0
    }

    pub(super) fn tst(&mut self, inst: &mut Instruction) -> usize {
        let (size, ea) = inst.operands.size_effective_address();

        if size.byte() {
            let data = self.get_byte(ea);
            self.sr.n = data & 0x80 != 0;
            self.sr.z = data == 0;
        } else if size.word() {
            let data = self.get_word(ea);
            self.sr.n = data & 0x8000 != 0;
            self.sr.z = data == 0;
        } else {
            let data = self.get_long(ea);
            self.sr.n = data & 0x8000_0000 != 0;
            self.sr.z = data == 0;
        }

        self.sr.v = false;
        self.sr.c = false;

        1
    }

    pub(super) fn unlk(&mut self, inst: &mut Instruction) -> usize {
        let reg = inst.operands.register();

        *self.sp_mut() = self.a(reg);
        *self.a_mut(reg) = self.pop_long();

        1
    }
}
