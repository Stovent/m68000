#![allow(overflowing_literals)]

use super::{M68000, MemoryAccess, StackFormat};
use super::decoder::DECODER;
use super::exception::Vector;
use super::instruction::{Direction, Instruction, Size};
use super::memory_access::MemoryIter;
use super::utils::{BigInt, bits};

const SR_UPPER_MASK: u16 = 0xA700;
const CCR_MASK: u16 = 0x001F;

impl<M: MemoryAccess> M68000<M> {
    /// Runs the CPU for `cycles` number of cycles.
    ///
    /// This function executes *at least* the given number of cycles.
    /// If you ask to execute 4 cycles but the next instruction takes 6 cycles to execute,
    /// it will be executed and the 2 extra cycles will be subtracted in the next call.
    pub fn execute_cycles(&mut self, cycles: usize) {
        while self.extra_cycles < cycles {
            self.extra_cycles += self.interpreter();
        }
        self.extra_cycles -= cycles;
    }

    /// Executes a single instruction, returning the cycle count necessary to execute it.
    pub fn interpreter(&mut self) -> usize {
        let mut cycle_count = 0;

        if let Some(vector) = self.exceptions.pop_front() {
            cycle_count += self.process_exception(vector);
        }

        if self.stop {
            return if cycle_count != 0 { cycle_count } else { 1 };
        }

        let pc = self.pc;
        let opcode = self.get_next_word();
        let isa = DECODER[opcode as usize];
        let entry = &Self::ISA_ENTRY[isa as usize];

        let mut iter = MemoryIter {
            memory: &mut self.memory,
            next_addr: self.pc,
        };
        let (operands, len) = (entry.decode)(opcode, &mut iter);
        self.pc += len as u32;

        let mut instruction = Instruction {
            opcode,
            pc,
            operands,
        };

        #[cfg(debug_assertions)]
        println!("{:#X} {}", pc, (entry.disassemble)(&mut instruction));

        cycle_count += (entry.execute)(self, &mut instruction);
        cycle_count
    }

    pub(super) fn unknown_instruction(&mut self, _: &mut Instruction) -> usize {
        self.exception(Vector::IllegalInstruction as u8);
        0
    }

    pub(super) fn abcd(&mut self, inst: &mut Instruction) -> usize {
        let (rx, _, mode, ry) = inst.operands.register_size_mode_register();

        let (src, dst) = if mode == Direction::MemoryToMemory {
            let src_addr = self.ariwpr(ry, Size::Byte);
            let dst_addr = self.ariwpr(rx, Size::Byte);
            (self.memory.get_byte(src_addr), self.memory.get_byte(dst_addr))
        } else {
            (self.d[ry as usize] as u8, self.d[rx as usize] as u8)
        };

        let low = (src & 0x0F) + (dst & 0x0F) + self.sr.x as u8;
        let high = (src >> 4 & 0x0F) + (dst >> 4 & 0x0F) + (low > 10) as u8;
        let res = (high << 4) | low;

        if mode == Direction::MemoryToMemory {
            self.memory.set_byte(self.a(rx), res);
        } else {
            self.d_byte(rx, res);
        }

        if res != 0 { self.sr.z = false; }
        self.sr.c = high > 10;
        self.sr.x = self.sr.c;

        1
    }

    pub(super) fn add(&mut self, inst: &mut Instruction) -> usize {
        let (reg, dir, size, ea) = inst.operands.register_direction_size_effective_address();

        if size.byte() {
            let (src, dst) = if dir == Direction::DstEa {
                (self.d[reg as usize] as u8, self.get_byte(ea))
            } else {
                (self.get_byte(ea), self.d[reg as usize] as u8)
            };

            let (res, v) = (src as i8).overflowing_add(dst as i8);
            let (_, c) = src.overflowing_add(dst);

            self.sr.x = c;
            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;

            if dir == Direction::DstEa {
                self.set_byte(ea, res as u8);
            } else {
                self.d_byte(reg, res as u8);
            }
        } else if size.word() {
            let (src, dst) = if dir == Direction::DstEa {
                (self.d[reg as usize] as u16, self.get_word(ea))
            } else {
                (self.get_word(ea), self.d[reg as usize] as u16)
            };

            let (res, v) = (src as i16).overflowing_add(dst as i16);
            let (_, c) = src.overflowing_add(dst);

            self.sr.x = c;
            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;

            if dir == Direction::DstEa {
                self.set_word(ea, res as u16);
            } else {
                self.d_word(reg, res as u16);
            }
        } else {
            let (src, dst) = if dir == Direction::DstEa {
                (self.d[reg as usize] as u32, self.get_long(ea))
            } else {
                (self.get_long(ea), self.d[reg as usize] as u32)
            };

            let (res, v) = (src as i32).overflowing_add(dst as i32);
            let (_, c) = src.overflowing_add(dst);

            self.sr.x = c;
            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;

            if dir == Direction::DstEa {
                self.set_long(ea, res as u32);
            } else {
                self.d[reg as usize] = res as u32;
            }
        }

        1
    }

    pub(super) fn adda(&mut self, inst: &mut Instruction) -> usize {
        let (reg, size, ea) = inst.operands.register_size_effective_address();

        let src = if size.word() {
            self.get_word(ea) as i16 as u32
        } else {
            self.get_long(ea)
        };

        *self.a_mut(reg) += src;

        1
    }

    pub(super) fn addi(&mut self, inst: &mut Instruction) -> usize {
        let (size, ea, imm) = inst.operands.size_effective_address_immediate();

        if size.byte() {
            let data = self.get_byte(ea);
            let (res, v) = (data as i8).overflowing_add(imm as i8);
            let (_, c) = data.overflowing_add(imm as u8);
            self.set_byte(ea, res as u8);

            self.sr.x = c;
            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        } else if size.word() {
            let data = self.get_word(ea);
            let (res, v) = (data as i16).overflowing_add(imm as i16);
            let (_, c) = data.overflowing_add(imm as u16);
            self.set_word(ea, res as u16);

            self.sr.x = c;
            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        } else {
            let data = self.get_long(ea);
            let (res, v) = (data as i32).overflowing_add(imm as i32);
            let (_, c) = data.overflowing_add(imm);
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
        let (imm, size, ea) = inst.operands.data_size_effective_address();

        if size.byte() {
            let data = self.get_byte(ea);
            let (res, v) = (data as i8).overflowing_add(imm as i8);
            let (_, c) = data.overflowing_add(imm);
            self.set_byte(ea, res as u8);

            self.sr.x = c;
            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        } else if size.word() {
            let data = self.get_word(ea);
            let (res, v) = (data as i16).overflowing_add(imm as i16);
            let (_, c) = data.overflowing_add(imm as u16);
            self.set_word(ea, res as u16);

            if !ea.mode.ard() {
                self.sr.x = c;
                self.sr.n = res < 0;
                self.sr.z = res == 0;
                self.sr.v = v;
                self.sr.c = c;
            }
        } else {
            let data = self.get_long(ea);
            let (res, v) = (data as i32).overflowing_add(imm as i32);
            let (_, c) = data.overflowing_add(imm as u32);
            self.set_long(ea, res as u32);

            if !ea.mode.ard() {
                self.sr.x = c;
                self.sr.n = res < 0;
                self.sr.z = res == 0;
                self.sr.v = v;
                self.sr.c = c;
            }
        }

        1
    }

    pub(super) fn addx(&mut self, inst: &mut Instruction) -> usize {
        let (rx, size, mode, ry) = inst.operands.register_size_mode_register();

        if size.byte() {
            let (src, dst) = if mode == Direction::MemoryToMemory {
                let src_addr = self.ariwpr(ry, size);
                let dst_addr = self.ariwpr(rx, size);
                (self.memory.get_byte(src_addr), self.memory.get_byte(dst_addr))
            } else {
                (self.d[ry as usize] as u8, self.d[rx as usize] as u8)
            };

            let (res, v) = (src as i8).extended_add(dst as i8, self.sr.x);
            let (_, c) = src.extended_add(dst, self.sr.x);

            self.sr.x = c;
            self.sr.n = res < 0;
            if res != 0 {
                self.sr.z = false;
            }
            self.sr.v = v;
            self.sr.c = c;

            if mode == Direction::MemoryToMemory {
                self.memory.set_byte(self.a(rx), res as u8);
            } else {
                self.d_byte(rx, res as u8);
            }
        } else if size.word() {
            let (src, dst) = if mode == Direction::MemoryToMemory {
                let src_addr = self.ariwpr(ry, size);
                let dst_addr = self.ariwpr(rx, size);
                (self.memory.get_word(src_addr), self.memory.get_word(dst_addr))
            } else {
                (self.d[ry as usize] as u16, self.d[rx as usize] as u16)
            };

            let (res, v) = (src as i16).extended_add(dst as i16, self.sr.x);
            let (_, c) = src.extended_add(dst, self.sr.x);

            self.sr.x = c;
            self.sr.n = res < 0;
            if res != 0 {
                self.sr.z = false;
            }
            self.sr.v = v;
            self.sr.c = c;

            if mode == Direction::MemoryToMemory {
                self.memory.set_word(self.a(rx), res as u16);
            } else {
                self.d_word(rx, res as u16);
            }
        } else {
            let (src, dst) = if mode == Direction::MemoryToMemory {
                let src_addr = self.ariwpr(ry, size);
                let dst_addr = self.ariwpr(rx, size);
                (self.memory.get_long(src_addr), self.memory.get_long(dst_addr))
            } else {
                (self.d[ry as usize], self.d[rx as usize])
            };

            let (res, v) = (src as i32).extended_add(dst as i32, self.sr.x);
            let (_, c) = src.extended_add(dst, self.sr.x);

            self.sr.x = c;
            self.sr.n = res < 0;
            if res != 0 {
                self.sr.z = false;
            }
            self.sr.v = v;
            self.sr.c = c;

            if mode == Direction::MemoryToMemory {
                self.memory.set_long(self.a(rx), res as u32);
            } else {
                self.d[rx as usize] = res as u32;
            }
        }

        1
    }

    pub(super) fn and(&mut self, inst: &mut Instruction) -> usize {
        let (reg, dir, size, ea) = inst.operands.register_direction_size_effective_address();

        if size.byte() {
            let src = self.d[reg as usize] as u8;
            let dst = self.get_byte(ea);

            let res = src & dst;

            self.sr.n = res & 0x80 != 0;
            self.sr.z = res == 0;

            if dir == Direction::DstEa {
                self.set_byte(ea, res);
            } else {
                self.d_byte(reg, res);
            }
        } else if size.word() {
            let src = self.d[reg as usize] as u16;
            let dst = self.get_word(ea);

            let res = src & dst;

            self.sr.n = res & 0x8000 != 0;
            self.sr.z = res == 0;

            if dir == Direction::DstEa {
                self.set_word(ea, res);
            } else {
                self.d_word(reg, res);
            }
        } else {
            let src = self.d[reg as usize];
            let dst = self.get_long(ea);

            let res = src & dst;

            self.sr.n = res & 0x8000_0000 != 0;
            self.sr.z = res == 0;

            if dir == Direction::DstEa {
                self.set_long(ea, res);
            } else {
                self.d[reg as usize] = res;
            }
        }

        self.sr.v = false;
        self.sr.c = false;

        1
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
            self.exception(Vector::PrivilegeViolation as u8);
            0
        }
    }

    pub(super) fn asm(&mut self, inst: &mut Instruction) -> usize {
        let (dir, ea) = inst.operands.direction_effective_address();

        let mut data = self.get_word(ea) as i16;
        let sign = data & 0x8000;

        if dir == Direction::Left {
            data <<= 1;
            self.sr.x = sign != 0;
            self.sr.v = sign ^ data & 0x8000 != 0;
            self.sr.c = sign != 0;
        } else {
            let bit = data & 1;
            data >>= 1;
            self.sr.x = bit != 0;
            self.sr.v = false;
            self.sr.c = bit != 0;
        }

        self.sr.n = data < 0;
        self.sr.z = data == 0;

        self.set_word(ea, data as u16);

        1
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

        if self.sr.condition(condition) {
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
        let (reg, ea) = inst.operands.register_effective_address();

        let src = self.get_word(ea) as i16;
        let data = self.d[reg as usize] as i16;

        if data < 0 || data > src {
            self.exception(Vector::ChkInstruction as u8);
            0
        } else {
            1
        }
    }

    pub(super) fn clr(&mut self, inst: &mut Instruction) -> usize {
        let (size, ea) = inst.operands.size_effective_address();

        if size.byte() {
            self.set_byte(ea, 0);
        } else if size.word() {
            self.set_word(ea, 0);
        } else {
            self.set_long(ea, 0);
        }

        self.sr.n = false;
        self.sr.z = true;
        self.sr.v = false;
        self.sr.c = false;

        1
    }

    pub(super) fn cmp(&mut self, inst: &mut Instruction) -> usize {
        let (reg, _, size, ea) = inst.operands.register_direction_size_effective_address();

        if size.byte() {
            let src = self.get_byte(ea);
            let dst = self.d[reg as usize] as u8;

            let (res, v) = (dst as i8).overflowing_sub(src as i8);
            let (_, c) = dst.overflowing_sub(src);

            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        } else if size.word() {
            let src = self.get_word(ea);
            let dst = self.d[reg as usize] as u16;

            let (res, v) = (dst as i16).overflowing_sub(src as i16);
            let (_, c) = dst.overflowing_sub(src);

            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        } else {
            let src = self.get_long(ea);
            let dst = self.d[reg as usize];

            let (res, v) = (dst as i32).overflowing_sub(src as i32);
            let (_, c) = dst.overflowing_sub(src);

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
            self.get_word(ea) as i16 as u32
        } else {
            self.get_long(ea)
        };

        let (res, v) = (self.a(reg) as i32).overflowing_sub(src as i32);
        let (_, c) = self.a(reg).overflowing_sub(src);

        self.sr.n = res < 0;
        self.sr.z = res == 0;
        self.sr.v = v;
        self.sr.c = c;

        1
    }

    pub(super) fn cmpi(&mut self, inst: &mut Instruction) -> usize {
        let (size, ea, imm) = inst.operands.size_effective_address_immediate();

        if size.byte() {
            let data = self.get_byte(ea);
            let (res, v) = (data as i8).overflowing_sub(imm as i8);
            let (_, c) = data.overflowing_sub(imm as u8);

            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        } else if size.word() {
            let data = self.get_word(ea);
            let (res, v) = (data as i16).overflowing_sub(imm as i16);
            let (_, c) = data.overflowing_sub(imm as u16);

            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        } else {
            let data = self.get_long(ea);
            let (res, v) = (data as i32).overflowing_sub(imm as i32);
            let (_, c) = data.overflowing_sub(imm);

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
            let src = self.memory.get_byte(ay);
            let dst = self.memory.get_byte(ax);

            let (res, v) = (dst as i8).overflowing_sub(src as i8);
            let (_, c) = dst.overflowing_sub(src);

            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        } else if size.word() {
            let ay = self.ariwpo(ay, size);
            let ax = self.ariwpo(ax, size);
            let src = self.memory.get_word(ay);
            let dst = self.memory.get_word(ax);

            let (res, v) = (dst as i16).overflowing_sub(src as i16);
            let (_, c) = dst.overflowing_sub(src);

            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        } else {
            let ay = self.ariwpo(ay, size);
            let ax = self.ariwpo(ax, size);
            let src = self.memory.get_long(ay);
            let dst = self.memory.get_long(ax);

            let (res, v) = (dst as i32).overflowing_sub(src as i32);
            let (_, c) = dst.overflowing_sub(src);

            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        }

        1
    }

    pub(super) fn dbcc(&mut self, inst: &mut Instruction) -> usize {
        let (cc, reg, disp) = inst.operands.condition_register_displacement();

        if !self.sr.condition(cc) {
            let counter = self.d[reg as usize] as i16 - 1;
            self.d_word(reg, counter as u16);

            if counter != -1 {
                self.pc = inst.pc + 2 + disp as u32;
            }
        }

        1
    }

    pub(super) fn divs(&mut self, inst: &mut Instruction) -> usize {
        let (reg, ea) = inst.operands.register_effective_address();

        let src = self.get_word(ea) as i16 as i32;
        let dst = self.d[reg as usize] as i32;

        if src == 0 {
            self.exception(Vector::ZeroDivide as u8);
            0
        } else {
            let quot = dst / src;
            let rem = dst % src;
            self.d[reg as usize] = (rem as u16 as u32) << 16 | (quot as u16 as u32);

            self.sr.n = quot < 0;
            self.sr.z = quot == 0;
            self.sr.v = quot < i16::MIN as i32 || quot > i16::MAX as i32;
            self.sr.c = false;

            1
        }
    }

    pub(super) fn divu(&mut self, inst: &mut Instruction) -> usize {
        let (reg, ea) = inst.operands.register_effective_address();

        let src = self.get_word(ea) as u32;
        let dst = self.d[reg as usize];

        if src == 0 {
            self.exception(Vector::ZeroDivide as u8);
            0
        } else {
            let quot = dst / src;
            let rem = dst % src;
            self.d[reg as usize] = (rem as u16 as u32) << 16 | (quot as u16 as u32);

            self.sr.n = quot & 0x0000_8000 != 0;
            self.sr.z = quot == 0;
            self.sr.v = (quot as i32) < i16::MIN as i32 || quot > i16::MAX as u32;
            self.sr.c = false;

            1
        }
    }

    pub(super) fn eor(&mut self, inst: &mut Instruction) -> usize {
        let (reg, _, size, ea) = inst.operands.register_direction_size_effective_address();

        if size.byte() {
            let src = self.d[reg as usize] as u8;
            let dst = self.get_byte(ea);

            let res = src ^ dst;

            self.sr.n = res & 0x80 != 0;
            self.sr.z = res == 0;

            self.set_byte(ea, res);
        } else if size.word() {
            let src = self.d[reg as usize] as u16;
            let dst = self.get_word(ea);

            let res = src ^ dst;

            self.sr.n = res & 0x8000 != 0;
            self.sr.z = res == 0;

            self.set_word(ea, res);
        } else {
            let src = self.d[reg as usize];
            let dst = self.get_long(ea);

            let res = src ^ dst;

            self.sr.n = res & 0x8000_0000 != 0;
            self.sr.z = res == 0;

            self.set_long(ea, res);
        }

        self.sr.v = false;
        self.sr.c = false;

        1
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
            self.exception(Vector::PrivilegeViolation as u8);
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

    pub(super) fn illegal(&mut self, _: &mut Instruction) -> usize {
        self.exception(Vector::IllegalInstruction as u8);
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
        let (reg, disp) = inst.operands.register_displacement();

        self.push_long(self.a(reg));
        *self.a_mut(reg) = self.sp();
        *self.sp_mut() += disp as u32;

        1
    }

    pub(super) fn lsm(&mut self, inst: &mut Instruction) -> usize {
        let (dir, ea) = inst.operands.direction_effective_address();

        let mut data = self.get_word(ea);

        if dir == Direction::Left {
            let sign = data & 0x8000;
            data <<= 1;
            self.sr.x = sign != 0;
            self.sr.c = sign != 0;
        } else {
            let bit = data & 1;
            data >>= 1;
            self.sr.x = bit != 0;
            self.sr.c = bit != 0;
        }

        self.sr.n = data & 0x8000 != 0;
        self.sr.z = data == 0;
        self.sr.v = false;

        self.set_word(ea, data);

        1
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
            self.exception(Vector::PrivilegeViolation as u8);
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
            self.exception(Vector::PrivilegeViolation as u8);
            0
        }
    }

    pub(super) fn movem(&mut self, inst: &mut Instruction) -> usize {
        let (dir, size, ea, mut list) = inst.operands.direction_size_effective_address_list();

        let gap = size as u32;

        if ea.mode.ariwpr() {
            let mut addr = self.a(ea.reg);

            for reg in (0..8).rev() {
                if list & 1 != 0 {
                    addr -= gap;
                    if size.word() { self.memory.set_word(addr, self.a(reg) as u16); }
                    else { self.memory.set_long(addr, self.a(reg)); }
                }

                list >>= 1;
            }

            for reg in (0..8).rev() {
                if list & 1 != 0 {
                    addr -= gap;
                    if size.word() { self.memory.set_word(addr, self.d[reg] as u16); }
                    else { self.memory.set_long(addr, self.d[reg]); }
                }

                list >>= 1;
            }

            *self.a_mut(ea.reg) = addr;
        } else {
            let mut addr = if ea.mode.ariwpo() {
                self.a(ea.reg)
            } else {
                self.get_effective_address(ea).unwrap()
            };

            for reg in 0..8 {
                if list & 1 != 0 {
                    if dir == Direction::MemoryToRegister {
                        let value = if size.word() { self.memory.get_word(addr) as i16 as u32 }
                            else { self.memory.get_long(addr) };
                        self.d[reg] = value;
                    } else {
                        if size.word() { self.memory.set_word(addr, self.d[reg] as u16); }
                        else { self.memory.set_long(addr, self.d[reg]); }
                    }

                    addr += gap;
                }

                list >>= 1;
            }

            for reg in 0..8 {
                if list & 1 != 0 {
                    if dir == Direction::MemoryToRegister {
                        let value = if size.word() { self.memory.get_word(addr) as i16 as u32 }
                            else { self.memory.get_long(addr) };
                        *self.a_mut(reg) = value;
                    } else {
                        if size.word() { self.memory.set_word(addr, self.a(reg as u8) as u16); }
                        else { self.memory.set_long(addr, self.a(reg as u8)); }
                    }

                    addr += gap;
                }

                list >>= 1;
            }

            if ea.mode.ariwpo() {
                *self.a_mut(ea.reg) = addr;
            }
        }

        1
    }

    pub(super) fn movep(&mut self, inst: &mut Instruction) -> usize {
        let (data, dir, size, addr, disp) = inst.operands.register_direction_size_register_displacement();

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
        let (reg, ea) = inst.operands.register_effective_address();

        let src = self.get_word(ea) as i16 as i32;
        let dst = self.d[reg as usize] as i16 as i32;

        let res = src * dst;
        self.d[reg as usize] = res as u32;

        self.sr.n = res < 0;
        self.sr.z = res == 0;
        self.sr.v = false;
        self.sr.c = false;

        1
    }

    pub(super) fn mulu(&mut self, inst: &mut Instruction) -> usize {
        let (reg, ea) = inst.operands.register_effective_address();

        let src = self.get_word(ea) as u32;
        let dst = self.d[reg as usize] as u16 as u32;

        let res = src * dst;
        self.d[reg as usize] = res;

        self.sr.n = res & 0x8000_0000 != 0;
        self.sr.z = res == 0;
        self.sr.v = false;
        self.sr.c = false;

        1
    }

    pub(super) fn nbcd(&mut self, inst: &mut Instruction) -> usize {
        let ea = inst.operands.effective_address();

        let data = self.get_byte(ea);

        let low = 0 - (data as i8 & 0x0F) - self.sr.x as i8;
        let high = 0 - (data as i8 >> 4 & 0x0F) - (low < 0) as i8;
        let res = (if high < 0 { 10 + high } else { high } as u8) << 4 |
                      if low < 0 { 10 + low } else { low } as u8;

        self.set_byte(ea, res);

        if res != 0 { self.sr.z = false; }
        self.sr.c = res != 0;
        self.sr.x = self.sr.c;

        1
    }

    pub(super) fn neg(&mut self, inst: &mut Instruction) -> usize {
        let (size, ea) = inst.operands.size_effective_address();

        if size.byte() {
            let data = -(self.get_byte(ea) as i8);
            self.set_byte(ea, data as u8);

            self.sr.n = data < 0;
            self.sr.z = data == 0;
            self.sr.v = data == i8::MIN;
            self.sr.c = data != 0;
            self.sr.x = self.sr.c;
        } else if size.word() {
            let data = -(self.get_word(ea) as i16);
            self.set_word(ea, data as u16);

            self.sr.n = data < 0;
            self.sr.z = data == 0;
            self.sr.v = data == i16::MIN;
            self.sr.c = data != 0;
            self.sr.x = self.sr.c;
        } else {
            let data = -(self.get_long(ea) as i32);
            self.set_long(ea, data as u32);

            self.sr.n = data < 0;
            self.sr.z = data == 0;
            self.sr.v = data == i32::MIN;
            self.sr.c = data != 0;
            self.sr.x = self.sr.c;
        }

        1
    }

    pub(super) fn negx(&mut self, inst: &mut Instruction) -> usize {
        let (size, ea) = inst.operands.size_effective_address();

        // using overflowing_sub indicates an overflow when negating -128 with the X flag set.
        // 0 - -128 stays -128, then -128 - 1 gives 127, which is an overflow.
        // However I don't know if the hardware has intermediate overflow.
        // The other way is 0 - -128 gives 128, then 128 - 1 gives 127 which generates no overflow.
        // TODO: test what the hardware actually does.
        if size.byte() {
            let data = self.get_byte(ea) as i8;
            let res = 0 - data - self.sr.x as i8;
            let vres = 0 - data as i16 - self.sr.x as i16;
            let (_, c) = 0u8.extended_sub(data as u8, self.sr.x);
            self.set_byte(ea, res as u8);

            self.sr.x = c;
            self.sr.n = res < 0;
            if res != 0 { self.sr.z = false };
            self.sr.v = vres < i8::MIN as i16 || vres > i8::MAX as i16;
            self.sr.c = c;
        } else if size.word() {
            let data = self.get_word(ea) as i16;
            let res = 0 - data - self.sr.x as i16;
            let vres = 0 - data as i32 - self.sr.x as i32;
            let (_, c) = 0u16.extended_sub(data as u16, self.sr.x);
            self.set_word(ea, res as u16);

            self.sr.x = c;
            self.sr.n = res < 0;
            if res != 0 { self.sr.z = false };
            self.sr.v = vres < i16::MIN as i32 || vres > i16::MAX as i32;
            self.sr.c = c;
        } else {
            let data = self.get_long(ea) as i32;
            let res = 0 - data - self.sr.x as i32;
            let vres = 0 - data as i64 - self.sr.x as i64;
            let (_, c) = 0u32.extended_sub(data as u32, self.sr.x);
            self.set_long(ea, res as u32);

            self.sr.x = c;
            self.sr.n = res < 0;
            if res != 0 { self.sr.z = false };
            self.sr.v = vres < i32::MIN as i64 || vres > i32::MAX as i64;
            self.sr.c = c;
        }

        1
    }

    pub(super) fn nop(&mut self, _: &mut Instruction) -> usize {
        1
    }

    pub(super) fn not(&mut self, inst: &mut Instruction) -> usize {
        let (size, ea) = inst.operands.size_effective_address();

        if size.byte() {
            let data = !self.get_byte(ea);
            self.set_byte(ea, data);

            self.sr.n = data & 0x80 != 0;
            self.sr.z = data == 0;
        } else if size.word() {
            let data = !self.get_word(ea);
            self.set_word(ea, data);

            self.sr.n = data & 0x8000 != 0;
            self.sr.z = data == 0;
        } else {
            let data = !self.get_long(ea);
            self.set_long(ea, data);

            self.sr.n = data & 0x8000_0000 != 0;
            self.sr.z = data == 0;
        }

        self.sr.v = false;
        self.sr.c = false;

        1
    }

    pub(super) fn or(&mut self, inst: &mut Instruction) -> usize {
        let (reg, dir, size, ea) = inst.operands.register_direction_size_effective_address();

        if size.byte() {
            let src = self.d[reg as usize] as u8;
            let dst = self.get_byte(ea);

            let res = src | dst;

            self.sr.n = res & 0x80 != 0;
            self.sr.z = res == 0;

            if dir == Direction::DstEa {
                self.set_byte(ea, res);
            } else {
                self.d_byte(reg, res);
            }
        } else if size.word() {
            let src = self.d[reg as usize] as u16;
            let dst = self.get_word(ea);

            let res = src | dst;

            self.sr.n = res & 0x8000 != 0;
            self.sr.z = res == 0;

            if dir == Direction::DstEa {
                self.set_word(ea, res);
            } else {
                self.d_word(reg, res);
            }
        } else {
            let src = self.d[reg as usize];
            let dst = self.get_long(ea);

            let res = src | dst;

            self.sr.n = res & 0x8000_0000 != 0;
            self.sr.z = res == 0;

            if dir == Direction::DstEa {
                self.set_long(ea, res);
            } else {
                self.d[reg as usize] = res;
            }
        }

        self.sr.v = false;
        self.sr.c = false;

        1
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
            self.exception(Vector::PrivilegeViolation as u8);
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
            self.exception(Vector::PrivilegeViolation as u8);
            0
        }
    }

    pub(super) fn rom(&mut self, inst: &mut Instruction) -> usize {
        let (dir, ea) = inst.operands.direction_effective_address();

        let mut data = self.get_word(ea);
        let sign = data & 0x8000;

        if dir == Direction::Left {
            data <<= 1;
            data |= (sign != 0) as u16;
            self.sr.c = sign != 0;
        } else {
            let bit = data & 1;
            data >>= 1;
            if bit != 0 {
                data |= sign;
            }
            self.sr.c = bit != 0;
        }

        self.sr.n = data & 0x8000 != 0;
        self.sr.z = data == 0;
        self.sr.v = false;

        self.set_word(ea, data);

        1
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
        let (dir, ea) = inst.operands.direction_effective_address();

        let mut data = self.get_word(ea);
        let sign = data & 0x8000;

        if dir == Direction::Left {
            data <<= 1;
            data |= self.sr.x as u16;
            self.sr.x = sign != 0;
            self.sr.c = sign != 0;
        } else {
            let bit = data & 1;
            data >>= 1;
            if self.sr.x {
                data |= 0x8000;
            }
            self.sr.x = bit != 0;
            self.sr.c = bit != 0;
        }

        self.sr.n = data & 0x8000 != 0;
        self.sr.z = data == 0;
        self.sr.v = false;

        self.set_word(ea, data);

        1
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

    pub(super) fn rte(&mut self, _: &mut Instruction) -> usize {
        if self.sr.s {
            let sr = self.pop_word();
            self.pc = self.pop_long();

            if self.stack_format == StackFormat::Stack68010 {
                let format = self.pop_word();
                if format & 0xF000 == 0xF000 {
                    *self.sp_mut() += 26;
                } else if format & 0xF000 != 0 {
                    self.exception(Vector::FormatError as u8);
                }
            }

            self.sr = sr.into();

            1
        } else {
            self.exception(Vector::PrivilegeViolation as u8);
            0
        }
    }

    pub(super) fn rtr(&mut self, _: &mut Instruction) -> usize {
        let ccr = self.pop_word();
        self.sr &= SR_UPPER_MASK;
        self.sr |= ccr & CCR_MASK;
        self.pc = self.pop_long();

        1
    }

    pub(super) fn rts(&mut self, _: &mut Instruction) -> usize {
        self.pc = self.pop_long();

        1
    }

    pub(super) fn sbcd(&mut self, inst: &mut Instruction) -> usize {
        let (ry, _, mode, rx) = inst.operands.register_size_mode_register();

        let (src, dst) = if mode == Direction::MemoryToMemory {
            let src_addr = self.ariwpr(rx, Size::Byte);
            let dst_addr = self.ariwpr(ry, Size::Byte);
            (self.memory.get_byte(src_addr), self.memory.get_byte(dst_addr))
        } else {
            (self.d[rx as usize] as u8, self.d[ry as usize] as u8)
        };

        let low = (dst as i8 & 0x0F) - (src as i8 & 0x0F) - self.sr.x as i8;
        let high = (dst as i8 >> 4 & 0x0F) - (src as i8 >> 4 & 0x0F) - (low < 0) as i8;
        let res = (if high < 0 { 10 + high } else { high } as u8) << 4 |
                      if low < 0 { 10 + low } else { low } as u8;

        if mode == Direction::MemoryToMemory {
            self.memory.set_byte(self.a(ry), res);
        } else {
            self.d_byte(ry, res);
        }

        if res != 0 { self.sr.z = false; }
        self.sr.c = high < 0;
        self.sr.x = self.sr.c;

        1
    }

    pub(super) fn scc(&mut self, inst: &mut Instruction) -> usize {
        let (cc, ea) = inst.operands.condition_effective_address();

        if self.sr.condition(cc) {
            self.set_byte(ea, 0xFF);
        } else {
            self.set_byte(ea, 0);
        }

        1
    }

    pub(super) fn stop(&mut self, inst: &mut Instruction) -> usize {
        let imm = inst.operands.immediate();

        if self.sr.s {
            // TODO: trace.
            self.sr = imm.into();
            self.stop = true;
            // TODO: how to regain control?
        } else {
            self.exception(Vector::PrivilegeViolation as u8);
        }

        1
    }

    pub(super) fn sub(&mut self, inst: &mut Instruction) -> usize {
        let (reg, dir, size, ea) = inst.operands.register_direction_size_effective_address();

        if size.byte() {
            let (src, dst) = if dir == Direction::DstEa {
                (self.d[reg as usize] as u8, self.get_byte(ea))
            } else {
                (self.get_byte(ea), self.d[reg as usize] as u8)
            };

            let (res, v) = (dst as i8).overflowing_sub(src as i8);
            let (_, c) = dst.overflowing_sub(src);

            self.sr.x = c;
            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;

            if dir == Direction::DstEa {
                self.set_byte(ea, res as u8);
            } else {
                self.d_byte(reg, res as u8);
            }
        } else if size.word() {
            let (src, dst) = if dir == Direction::DstEa {
                (self.d[reg as usize] as u16, self.get_word(ea))
            } else {
                (self.get_word(ea), self.d[reg as usize] as u16)
            };

            let (res, v) = (dst as i16).overflowing_sub(src as i16);
            let (_, c) = dst.overflowing_sub(src);

            self.sr.x = c;
            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;

            if dir == Direction::DstEa {
                self.set_word(ea, res as u16);
            } else {
                self.d_word(reg, res as u16);
            }
        } else {
            let (src, dst) = if dir == Direction::DstEa {
                (self.d[reg as usize] as u32, self.get_long(ea))
            } else {
                (self.get_long(ea), self.d[reg as usize] as u32)
            };

            let (res, v) = (dst as i32).overflowing_sub(src as i32);
            let (_, c) = dst.overflowing_sub(src);

            self.sr.x = c;
            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;

            if dir == Direction::DstEa {
                self.set_long(ea, res as u32);
            } else {
                self.d[reg as usize] = res as u32;
            }
        }

        1
    }

    pub(super) fn suba(&mut self, inst: &mut Instruction) -> usize {
        let (reg, size, ea) = inst.operands.register_size_effective_address();

        let src = if size.word() {
            self.get_word(ea) as i16 as u32
        } else {
            self.get_long(ea)
        };

        *self.a_mut(reg) -= src;

        1
    }

    pub(super) fn subi(&mut self, inst: &mut Instruction) -> usize {
        let (size, ea, imm) = inst.operands.size_effective_address_immediate();

        if size.byte() {
            let data = self.get_byte(ea);
            let (res, v) = (data as i8).overflowing_sub(imm as i8);
            let (_, c) = data.overflowing_sub(imm as u8);
            self.set_byte(ea, res as u8);

            self.sr.x = c;
            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        } else if size.word() {
            let data = self.get_word(ea);
            let (res, v) = (data as i16).overflowing_sub(imm as i16);
            let (_, c) = data.overflowing_sub(imm as u16);
            self.set_word(ea, res as u16);

            self.sr.x = c;
            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        } else {
            let data = self.get_long(ea);
            let (res, v) = (data as i32).overflowing_sub(imm as i32);
            let (_, c) = data.overflowing_sub(imm);
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
        let (imm, size, ea) = inst.operands.data_size_effective_address();

        if size.byte() {
            let data = self.get_byte(ea);
            let (res, v) = (data as i8).overflowing_sub(imm as i8);
            let (_, c) = data.overflowing_sub(imm);
            self.set_byte(ea, res as u8);

            self.sr.x = c;
            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        } else if size.word() {
            let data = self.get_word(ea);
            let (res, v) = (data as i16).overflowing_sub(imm as i16);
            let (_, c) = data.overflowing_sub(imm as u16);
            self.set_word(ea, res as u16);

            if !ea.mode.ard() {
                self.sr.x = c;
                self.sr.n = res < 0;
                self.sr.z = res == 0;
                self.sr.v = v;
                self.sr.c = c;
            }
        } else {
            let data = self.get_long(ea);
            let (res, v) = (data as i32).overflowing_sub(imm as i32);
            let (_, c) = data.overflowing_sub(imm as u32);
            self.set_long(ea, res as u32);

            if !ea.mode.ard() {
                self.sr.x = c;
                self.sr.n = res < 0;
                self.sr.z = res == 0;
                self.sr.v = v;
                self.sr.c = c;
            }
        }

        1
    }

    pub(super) fn subx(&mut self, inst: &mut Instruction) -> usize {
        let (ry, size, mode, rx) = inst.operands.register_size_mode_register();

        if size.byte() {
            let (src, dst) = if mode == Direction::MemoryToMemory {
                let src_addr = self.ariwpr(rx, size);
                let dst_addr = self.ariwpr(ry, size);
                (self.memory.get_byte(src_addr), self.memory.get_byte(dst_addr))
            } else {
                (self.d[rx as usize] as u8, self.d[ry as usize] as u8)
            };

            let (res, v) = (dst as i8).extended_sub(src as i8, self.sr.x);
            let (_, c) = dst.extended_sub(src, self.sr.x);

            self.sr.n = res < 0;
            if res != 0 {
                self.sr.z = false;
            }
            self.sr.v = v;
            self.sr.c = c;
            self.sr.x = c;

            if mode == Direction::MemoryToMemory {
                self.memory.set_byte(self.a(ry), res as u8);
            } else {
                self.d_byte(ry, res as u8);
            }
        } else if size.word() {
            let (src, dst) = if mode == Direction::MemoryToMemory {
                let src_addr = self.ariwpr(rx, size);
                let dst_addr = self.ariwpr(ry, size);
                (self.memory.get_word(src_addr), self.memory.get_word(dst_addr))
            } else {
                (self.d[rx as usize] as u16, self.d[ry as usize] as u16)
            };

            let (res, v) = (dst as i16).extended_sub(src as i16, self.sr.x);
            let (_, c) = dst.extended_sub(src, self.sr.x);

            self.sr.n = res < 0;
            if res != 0 {
                self.sr.z = false;
            }
            self.sr.v = v;
            self.sr.c = c;
            self.sr.x = c;

            if mode == Direction::MemoryToMemory {
                self.memory.set_word(self.a(ry), res as u16);
            } else {
                self.d_word(ry, res as u16);
            }
        } else {
            let (src, dst) = if mode == Direction::MemoryToMemory {
                let src_addr = self.ariwpr(rx, size);
                let dst_addr = self.ariwpr(ry, size);
                (self.memory.get_long(src_addr), self.memory.get_long(dst_addr))
            } else {
                (self.d[rx as usize], self.d[ry as usize])
            };

            let (res, v) = (dst as i32).extended_sub(src as i32, self.sr.x);
            let (_, c) = dst.extended_sub(src, self.sr.x);

            self.sr.n = res < 0;
            if res != 0 {
                self.sr.z = false;
            }
            self.sr.v = v;
            self.sr.c = c;
            self.sr.x = c;

            if mode == Direction::MemoryToMemory {
                self.memory.set_long(self.a(ry), res as u32);
            } else {
                self.d[ry as usize] = res as u32;
            }
        }

        1
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
        let vector = inst.operands.vector();
        self.exception(vector);
        0
    }

    pub(super) fn trapv(&mut self, _: &mut Instruction) -> usize {
        if self.sr.v {
            self.exception(Vector::TrapVInstruction as u8);
            0
        } else {
            1
        }
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
