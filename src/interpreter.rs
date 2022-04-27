use crate::{M68000, MemoryAccess};
use crate::addressing_modes::EffectiveAddress;
use crate::exception::Vector;
use crate::instruction::{Direction, Instruction, Size};
use crate::isa::{Execute, Isa, IsaEntry};
use crate::utils::{BigInt, bits};

const SR_UPPER_MASK: u16 = 0xA700;
const CCR_MASK: u16 = 0x001F;
const SIGN_BIT_8: u8 = 0x80;
const SIGN_BIT_16: u16 = 0x8000;
const SIGN_BIT_32: u32 = 0x8000_0000;

/// Returns the execution time on success, an exception vector on error. Alias for `Result<usize, u8>`.
pub(super) type InterpreterResult = Result<usize, u8>;

impl M68000 {
    /// Runs the CPU for `cycles` number of cycles.
    ///
    /// This function executes **at least** the given number of cycles.
    /// Returns the number of cycles actually executed.
    ///
    /// If you ask to execute 4 cycles but the next instruction takes 6 cycles to execute,
    /// it will be executed and the 2 extra cycles will be subtracted in the next call.
    pub fn execute_cycles(&mut self, memory: &mut impl MemoryAccess, cycles: usize) -> usize {
        let mut total_cycles = 0;

        while self.extra_cycles < cycles {
            let c = self.interpreter(memory);
            self.extra_cycles += c;
            total_cycles += c;
        }

        self.extra_cycles -= cycles;
        total_cycles
    }

    /// Runs the CPU until either an exception occurs or `cycle` cycles have been executed.
    ///
    /// This function executes **at least** the given number of cycles.
    /// Returns the number of cycles actually executed, and the exception that occured if any.
    ///
    /// If you ask to execute 4 cycles but the next instruction takes 6 cycles to execute,
    /// it will be executed and the 2 extra cycles will be subtracted in the next call.
    pub fn execute_cycles_exception(&mut self, memory: &mut impl MemoryAccess, cycles: usize) -> (usize, Option<u8>) {
        let mut total_cycles = 0;

        while self.extra_cycles < cycles {
            let (c, vector) = self.interpreter_exception(memory);
            self.extra_cycles += c;
            total_cycles += c;

            if vector.is_some() {
                return (total_cycles, vector);
            }
        }

        self.extra_cycles -= cycles;
        (total_cycles, None)
    }

    /// Runs indefinitely until an exception or STOP instruction occurs.
    ///
    /// Returns the number of cycles executed and the exception that occured.
    /// If exception is None, this means the CPU has executed a STOP instruction.
    pub fn loop_until_exception_stop(&mut self, memory: &mut impl MemoryAccess) -> (usize, Option<u8>) {
        let mut total_cycles = 0;
        loop {
            let (cycles, vector) = self.interpreter_exception(memory);
            total_cycles += cycles;
            if vector.is_some() || self.stop {
                return (total_cycles, vector);
            }
        }
    }

    /// Executes a single instruction, returning the cycle count necessary to execute it.
    pub fn interpreter<M: MemoryAccess>(&mut self, memory: &mut M) -> usize {
        let mut cycle_count = 0;

        if let Some(vector) = self.exceptions.pop_front() {
            cycle_count += match self.process_exception(memory, vector) {
                Ok(cycles) => cycles,
                Err(e) => panic!("An exception occured during exception processing: {} (at {:#X})", e, self.pc),
            };
        }

        if self.stop {
            return if cycle_count != 0 { cycle_count } else { 1 };
        }

        let pc = self.pc;
        let opcode = match self.get_next_word(memory) {
            Ok(op) => op,
            Err(e) => {
                self.exception(e);
                return 0;
            },
        };
        let isa: Isa = opcode.into();

        let mut iter = memory.iter_u16(self.pc);
        let (instruction, len) = Instruction::from_opcode(opcode, pc, &mut iter);
        self.pc += len as u32;

        if self.disassemble {
            memory.disassembler(pc, (IsaEntry::ISA_ENTRY[isa as usize].disassemble)(&instruction));
        }

        match Execute::<M>::EXECUTE[isa as usize](self, memory, &instruction) {
            Ok(cycles) => cycle_count += cycles,
            Err(e) => self.exception(e), // TODO: return 0 cycles ?
        }

        cycle_count
    }

    /// Executes a single instruction, returning the cycle count necessary to execute it,
    /// and the vector of the exception that occured during the execution if any.
    ///
    /// To process the returned exception, call [M68000::exception].
    pub fn interpreter_exception<M: MemoryAccess>(&mut self, memory: &mut M) -> (usize, Option<u8>) {
        let mut cycle_count = 0;

        if let Some(vector) = self.exceptions.pop_front() {
            cycle_count += match self.process_exception(memory, vector) {
                Ok(cycles) => cycles,
                Err(e) => panic!("An exception occured during exception processing: {} (at {:#X})", e, self.pc),
            };
        }

        if self.stop {
            return (if cycle_count != 0 { cycle_count } else { 1 }, None);
        }

        let pc = self.pc;
        let opcode = match self.get_next_word(memory) {
            Ok(value) => value,
            Err(e) => return (cycle_count, Some(e)),
        };
        let isa: Isa = opcode.into();

        let mut iter = memory.iter_u16(self.pc);
        let (instruction, len) = Instruction::from_opcode(opcode, pc, &mut iter);
        self.pc += len as u32;

        if self.disassemble {
            memory.disassembler(pc, (IsaEntry::ISA_ENTRY[isa as usize].disassemble)(&instruction));
        }

        match Execute::<M>::EXECUTE[isa as usize](self, memory, &instruction) {
            Ok(cycles) => cycle_count += cycles,
            Err(e) => return (cycle_count, Some(e)),
        }

        (cycle_count, None)
    }

    pub(super) fn unknown_instruction(&mut self, _: &mut impl MemoryAccess, _: &Instruction) -> InterpreterResult {
        Err(Vector::IllegalInstruction as u8)
    }

    pub(super) fn abcd(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (rx, _, mode, ry) = inst.operands.register_size_mode_register();

        let (src, dst) = if mode == Direction::MemoryToMemory {
            let src_addr = self.ariwpr(ry, Size::Byte);
            let dst_addr = self.ariwpr(rx, Size::Byte);
            (memory.get_byte(src_addr)?, memory.get_byte(dst_addr)?)
        } else {
            (self.d[ry as usize] as u8, self.d[rx as usize] as u8)
        };

        let low = (src & 0x0F) + (dst & 0x0F) + self.sr.x as u8;
        let high = (src >> 4 & 0x0F) + (dst >> 4 & 0x0F) + (low > 10) as u8;
        let res = (high << 4) | low;

        if mode == Direction::MemoryToMemory {
            memory.set_byte(self.a(rx), res)?;
        } else {
            self.d_byte(rx, res);
        }

        if res != 0 { self.sr.z = false; }
        self.sr.c = high > 10;
        self.sr.x = self.sr.c;

        Ok(1)
    }

    pub(super) fn add(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (reg, dir, size, am) = inst.operands.register_direction_size_effective_address();

        let mut ea = EffectiveAddress::new(am, Some(size));

        if size.is_byte() {
            let (src, dst) = if dir == Direction::DstEa {
                (self.d[reg as usize] as u8, self.get_byte(memory, &mut ea)?)
            } else {
                (self.get_byte(memory, &mut ea)?, self.d[reg as usize] as u8)
            };

            let (res, v) = (src as i8).overflowing_add(dst as i8);
            let (_, c) = src.overflowing_add(dst);

            self.sr.x = c;
            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;

            if dir == Direction::DstEa {
                self.set_byte(memory, &mut ea, res as u8)?;
            } else {
                self.d_byte(reg, res as u8);
            }
        } else if size.is_word() {
            let (src, dst) = if dir == Direction::DstEa {
                (self.d[reg as usize] as u16, self.get_word(memory, &mut ea)?)
            } else {
                (self.get_word(memory, &mut ea)?, self.d[reg as usize] as u16)
            };

            let (res, v) = (src as i16).overflowing_add(dst as i16);
            let (_, c) = src.overflowing_add(dst);

            self.sr.x = c;
            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;

            if dir == Direction::DstEa {
                self.set_word(memory, &mut ea, res as u16)?;
            } else {
                self.d_word(reg, res as u16);
            }
        } else {
            let (src, dst) = if dir == Direction::DstEa {
                (self.d[reg as usize] as u32, self.get_long(memory, &mut ea)?)
            } else {
                (self.get_long(memory, &mut ea)?, self.d[reg as usize] as u32)
            };

            let (res, v) = (src as i32).overflowing_add(dst as i32);
            let (_, c) = src.overflowing_add(dst);

            self.sr.x = c;
            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;

            if dir == Direction::DstEa {
                self.set_long(memory, &mut ea, res as u32)?;
            } else {
                self.d[reg as usize] = res as u32;
            }
        }

        Ok(1)
    }

    pub(super) fn adda(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (reg, size, am) = inst.operands.register_size_effective_address();

        let mut ea = EffectiveAddress::new(am, Some(size));

        let src = if size.is_word() {
            self.get_word(memory, &mut ea)? as i16 as u32
        } else {
            self.get_long(memory, &mut ea)?
        };

        *self.a_mut(reg) += src;

        Ok(1)
    }

    pub(super) fn addi(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (size, am, imm) = inst.operands.size_effective_address_immediate();

        let mut ea = EffectiveAddress::new(am, Some(size));

        if size.is_byte() {
            let data = self.get_byte(memory, &mut ea)?;
            let (res, v) = (data as i8).overflowing_add(imm as i8);
            let (_, c) = data.overflowing_add(imm as u8);
            self.set_byte(memory, &mut ea, res as u8)?;

            self.sr.x = c;
            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        } else if size.is_word() {
            let data = self.get_word(memory, &mut ea)?;
            let (res, v) = (data as i16).overflowing_add(imm as i16);
            let (_, c) = data.overflowing_add(imm as u16);
            self.set_word(memory, &mut ea, res as u16)?;

            self.sr.x = c;
            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        } else {
            let data = self.get_long(memory, &mut ea)?;
            let (res, v) = (data as i32).overflowing_add(imm as i32);
            let (_, c) = data.overflowing_add(imm);
            self.set_long(memory, &mut ea, res as u32)?;

            self.sr.x = c;
            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        }

        Ok(1)
    }

    pub(super) fn addq(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (imm, size, am) = inst.operands.data_size_effective_address();

        let mut ea = EffectiveAddress::new(am, Some(size));

        if size.is_byte() {
            let data = self.get_byte(memory, &mut ea)?;
            let (res, v) = (data as i8).overflowing_add(imm as i8);
            let (_, c) = data.overflowing_add(imm);
            self.set_byte(memory, &mut ea, res as u8)?;

            self.sr.x = c;
            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        } else if size.is_word() {
            let data = self.get_word(memory, &mut ea)?;
            let (res, v) = (data as i16).overflowing_add(imm as i16);
            let (_, c) = data.overflowing_add(imm as u16);
            self.set_word(memory, &mut ea, res as u16)?;

            if !ea.mode.is_ard() {
                self.sr.x = c;
                self.sr.n = res < 0;
                self.sr.z = res == 0;
                self.sr.v = v;
                self.sr.c = c;
            }
        } else {
            let data = self.get_long(memory, &mut ea)?;
            let (res, v) = (data as i32).overflowing_add(imm as i32);
            let (_, c) = data.overflowing_add(imm as u32);
            self.set_long(memory, &mut ea, res as u32)?;

            if !ea.mode.is_ard() {
                self.sr.x = c;
                self.sr.n = res < 0;
                self.sr.z = res == 0;
                self.sr.v = v;
                self.sr.c = c;
            }
        }

        Ok(1)
    }

    pub(super) fn addx(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (rx, size, mode, ry) = inst.operands.register_size_mode_register();

        if size.is_byte() {
            let (src, dst) = if mode == Direction::MemoryToMemory {
                let src_addr = self.ariwpr(ry, size);
                let dst_addr = self.ariwpr(rx, size);
                (memory.get_byte(src_addr)?, memory.get_byte(dst_addr)?)
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
                memory.set_byte(self.a(rx), res as u8)?;
            } else {
                self.d_byte(rx, res as u8);
            }
        } else if size.is_word() {
            let (src, dst) = if mode == Direction::MemoryToMemory {
                let src_addr = self.ariwpr(ry, size);
                let dst_addr = self.ariwpr(rx, size);
                (memory.get_word(src_addr)?, memory.get_word(dst_addr)?)
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
                memory.set_word(self.a(rx), res as u16)?;
            } else {
                self.d_word(rx, res as u16);
            }
        } else {
            let (src, dst) = if mode == Direction::MemoryToMemory {
                let src_addr = self.ariwpr(ry, size);
                let dst_addr = self.ariwpr(rx, size);
                (memory.get_long(src_addr)?, memory.get_long(dst_addr)?)
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
                memory.set_long(self.a(rx), res as u32)?;
            } else {
                self.d[rx as usize] = res as u32;
            }
        }

        Ok(1)
    }

    pub(super) fn and(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (reg, dir, size, am) = inst.operands.register_direction_size_effective_address();

        let mut ea = EffectiveAddress::new(am, Some(size));

        if size.is_byte() {
            let src = self.d[reg as usize] as u8;
            let dst = self.get_byte(memory, &mut ea)?;

            let res = src & dst;

            self.sr.n = res & SIGN_BIT_8 != 0;
            self.sr.z = res == 0;

            if dir == Direction::DstEa {
                self.set_byte(memory, &mut ea, res)?;
            } else {
                self.d_byte(reg, res);
            }
        } else if size.is_word() {
            let src = self.d[reg as usize] as u16;
            let dst = self.get_word(memory, &mut ea)?;

            let res = src & dst;

            self.sr.n = res & SIGN_BIT_16 != 0;
            self.sr.z = res == 0;

            if dir == Direction::DstEa {
                self.set_word(memory, &mut ea, res)?;
            } else {
                self.d_word(reg, res);
            }
        } else {
            let src = self.d[reg as usize];
            let dst = self.get_long(memory, &mut ea)?;

            let res = src & dst;

            self.sr.n = res & SIGN_BIT_32 != 0;
            self.sr.z = res == 0;

            if dir == Direction::DstEa {
                self.set_long(memory, &mut ea, res)?;
            } else {
                self.d[reg as usize] = res;
            }
        }

        self.sr.v = false;
        self.sr.c = false;

        Ok(1)
    }

    pub(super) fn andi(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (size, am, imm) = inst.operands.size_effective_address_immediate();

        let mut ea = EffectiveAddress::new(am, Some(size));

        if size.is_byte() {
            let data = self.get_byte(memory, &mut ea)? & imm as u8;
            self.set_byte(memory, &mut ea, data)?;

            self.sr.n = data & SIGN_BIT_8 != 0;
            self.sr.z = data == 0;
        } else if size.is_word() {
            let data = self.get_word(memory, &mut ea)? & imm as u16;
            self.set_word(memory, &mut ea, data)?;

            self.sr.n = data & SIGN_BIT_16 != 0;
            self.sr.z = data == 0;
        } else {
            let data = self.get_long(memory, &mut ea)? & imm;
            self.set_long(memory, &mut ea, data)?;

            self.sr.n = data & SIGN_BIT_32 != 0;
            self.sr.z = data == 0;
        }

        self.sr.v = false;
        self.sr.c = false;

        Ok(1)
    }

    pub(super) fn andiccr(&mut self, _: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let imm = inst.operands.immediate();

        self.sr &= SR_UPPER_MASK | imm;

        Ok(1)
    }

    pub(super) fn andisr(&mut self, _: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        if self.sr.s {
            let imm = inst.operands.immediate();
            self.sr &= imm;
            Ok(1)
        } else {
            Err(Vector::PrivilegeViolation as u8)
        }
    }

    pub(super) fn asm(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (dir, am) = inst.operands.direction_effective_address();

        let mut ea = EffectiveAddress::new(am, Some(Size::Word));

        let mut data = self.get_word(memory, &mut ea)? as i16;
        let sign = data & SIGN_BIT_16 as i16;

        if dir == Direction::Left {
            data <<= 1;
            self.sr.x = sign != 0;
            self.sr.v = sign ^ data & SIGN_BIT_16 as i16 != 0;
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

        self.set_word(memory, &mut ea, data as u16)?;

        Ok(1)
    }

    pub(super) fn asr(&mut self, _: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (rot, dir, size, mode, reg) = inst.operands.rotation_direction_size_mode_register();

        self.sr.v = false;
        self.sr.c = false;

        let shift_count = if mode == 1 {
            (self.d[rot as usize] % 64) as u8
        } else {
            if rot == 0 { 8 } else { rot }
        };

        let (mut data, mask) = if size == Size::Byte {
            (self.d[reg as usize] & 0x0000_00FF, SIGN_BIT_8 as u32)
        } else if size == Size::Word {
            (self.d[reg as usize] & 0x0000_FFFF, SIGN_BIT_16 as u32)
        } else {
            (self.d[reg as usize], SIGN_BIT_32)
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

        Ok(1)
    }

    pub(super) fn bcc(&mut self, _: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (condition, displacement) = inst.operands.condition_displacement();

        if self.sr.condition(condition) {
            self.pc = inst.pc + 2 + displacement as u32;
        }

        Ok(1)
    }

    pub(super) fn bchg(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (am, mut count) = inst.operands.effective_address_count();

        let mut ea = EffectiveAddress::new(am, Some(Size::Byte)); // Memory is byte only.

        if bits(inst.opcode, 8, 8) != 0 {
            count = self.d[count as usize] as u8;
        }

        if ea.mode.is_drd() {
            count %= 32;
            let reg = ea.mode.register().unwrap() as usize;
            self.sr.z = self.d[reg] & 1 << count == 0;
            self.d[reg] ^= 1 << count;
        } else {
            count %= 8;
            let mut data = self.get_byte(memory, &mut ea)?;
            self.sr.z = data & 1 << count == 0;
            data ^= 1 << count;
            self.set_byte(memory, &mut ea, data)?;
        }

        Ok(1)
    }

    pub(super) fn bclr(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (am, mut count) = inst.operands.effective_address_count();

        let mut ea = EffectiveAddress::new(am, Some(Size::Byte)); // Memory is byte only.

        if bits(inst.opcode, 8, 8) != 0 {
            count = self.d[count as usize] as u8;
        }

        if ea.mode.is_drd() {
            count %= 32;
            let reg = ea.mode.register().unwrap() as usize;
            self.sr.z = self.d[reg] & 1 << count == 0;
            self.d[reg] &= !(1 << count);
        } else {
            count %= 8;
            let mut data = self.get_byte(memory, &mut ea)?;
            self.sr.z = data & 1 << count == 0;
            data &= !(1 << count);
            self.set_byte(memory, &mut ea, data)?;
        }

        Ok(1)
    }

    pub(super) fn bra(&mut self, _: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let disp = inst.operands.displacement();

        self.pc = inst.pc + 2 + disp as u32;

        Ok(1)
    }

    pub(super) fn bset(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (am, mut count) = inst.operands.effective_address_count();

        let mut ea = EffectiveAddress::new(am, Some(Size::Byte)); // Memory is byte only.

        if bits(inst.opcode, 8, 8) != 0 {
            count = self.d[count as usize] as u8;
        }

        if ea.mode.is_drd() {
            count %= 32;
            let reg = ea.mode.register().unwrap() as usize;
            self.sr.z = self.d[reg] & 1 << count == 0;
            self.d[reg] |= 1 << count;
        } else {
            count %= 8;
            let mut data = self.get_byte(memory, &mut ea)?;
            self.sr.z = data & 1 << count == 0;
            data |= 1 << count;
            self.set_byte(memory, &mut ea, data)?;
        }

        Ok(1)
    }

    pub(super) fn bsr(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let disp = inst.operands.displacement();

        self.push_long(memory, self.pc)?;
        self.pc = inst.pc + 2 + disp as u32;

        Ok(1)
    }

    pub(super) fn btst(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (am, mut count) = inst.operands.effective_address_count();

        let mut ea = EffectiveAddress::new(am, Some(Size::Byte)); // Memory is byte only.

        if bits(inst.opcode, 8, 8) != 0 {
            count = self.d[count as usize] as u8;
        }

        if ea.mode.is_drd() {
            count %= 32;
            let reg = ea.mode.register().unwrap() as usize;
            self.sr.z = self.d[reg] & 1 << count == 0;
        } else {
            count %= 8;
            let data = self.get_byte(memory, &mut ea)?;
            self.sr.z = data & 1 << count == 0;
        }

        Ok(1)
    }

    pub(super) fn chk(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (reg, am) = inst.operands.register_effective_address();

        let mut ea = EffectiveAddress::new(am, Some(Size::Word));

        let src = self.get_word(memory, &mut ea)? as i16;
        let data = self.d[reg as usize] as i16;

        if data < 0 || data > src {
            Err(Vector::ChkInstruction as u8)
        } else {
            Ok(1)
        }
    }

    pub(super) fn clr(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (size, am) = inst.operands.size_effective_address();

        let mut ea = EffectiveAddress::new(am, Some(size));

        if size.is_byte() {
            self.set_byte(memory, &mut ea, 0)?;
        } else if size.is_word() {
            self.set_word(memory, &mut ea, 0)?;
        } else {
            self.set_long(memory, &mut ea, 0)?;
        }

        self.sr.n = false;
        self.sr.z = true;
        self.sr.v = false;
        self.sr.c = false;

        Ok(1)
    }

    pub(super) fn cmp(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (reg, _, size, am) = inst.operands.register_direction_size_effective_address();

        let mut ea = EffectiveAddress::new(am, Some(size));

        if size.is_byte() {
            let src = self.get_byte(memory, &mut ea)?;
            let dst = self.d[reg as usize] as u8;

            let (res, v) = (dst as i8).overflowing_sub(src as i8);
            let (_, c) = dst.overflowing_sub(src);

            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        } else if size.is_word() {
            let src = self.get_word(memory, &mut ea)?;
            let dst = self.d[reg as usize] as u16;

            let (res, v) = (dst as i16).overflowing_sub(src as i16);
            let (_, c) = dst.overflowing_sub(src);

            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        } else {
            let src = self.get_long(memory, &mut ea)?;
            let dst = self.d[reg as usize];

            let (res, v) = (dst as i32).overflowing_sub(src as i32);
            let (_, c) = dst.overflowing_sub(src);

            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        }

        Ok(1)
    }

    pub(super) fn cmpa(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (reg, size, am) = inst.operands.register_size_effective_address();

        let mut ea = EffectiveAddress::new(am, Some(size));

        let src = if size.is_word() {
            self.get_word(memory, &mut ea)? as i16 as u32
        } else {
            self.get_long(memory, &mut ea)?
        };

        let (res, v) = (self.a(reg) as i32).overflowing_sub(src as i32);
        let (_, c) = self.a(reg).overflowing_sub(src);

        self.sr.n = res < 0;
        self.sr.z = res == 0;
        self.sr.v = v;
        self.sr.c = c;

        Ok(1)
    }

    pub(super) fn cmpi(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (size, am, imm) = inst.operands.size_effective_address_immediate();

        let mut ea = EffectiveAddress::new(am, Some(size));

        if size.is_byte() {
            let data = self.get_byte(memory, &mut ea)?;
            let (res, v) = (data as i8).overflowing_sub(imm as i8);
            let (_, c) = data.overflowing_sub(imm as u8);

            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        } else if size.is_word() {
            let data = self.get_word(memory, &mut ea)?;
            let (res, v) = (data as i16).overflowing_sub(imm as i16);
            let (_, c) = data.overflowing_sub(imm as u16);

            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        } else {
            let data = self.get_long(memory, &mut ea)?;
            let (res, v) = (data as i32).overflowing_sub(imm as i32);
            let (_, c) = data.overflowing_sub(imm);

            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        }

        Ok(1)
    }

    pub(super) fn cmpm(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (ax, size, ay) = inst.operands.register_size_register();

        if size.is_byte() {
            let ay = self.ariwpo(ay, size);
            let ax = self.ariwpo(ax, size);
            let src = memory.get_byte(ay)?;
            let dst = memory.get_byte(ax)?;

            let (res, v) = (dst as i8).overflowing_sub(src as i8);
            let (_, c) = dst.overflowing_sub(src);

            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        } else if size.is_word() {
            let ay = self.ariwpo(ay, size);
            let ax = self.ariwpo(ax, size);
            let src = memory.get_word(ay)?;
            let dst = memory.get_word(ax)?;

            let (res, v) = (dst as i16).overflowing_sub(src as i16);
            let (_, c) = dst.overflowing_sub(src);

            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        } else {
            let ay = self.ariwpo(ay, size);
            let ax = self.ariwpo(ax, size);
            let src = memory.get_long(ay)?;
            let dst = memory.get_long(ax)?;

            let (res, v) = (dst as i32).overflowing_sub(src as i32);
            let (_, c) = dst.overflowing_sub(src);

            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        }

        Ok(1)
    }

    pub(super) fn dbcc(&mut self, _: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (cc, reg, disp) = inst.operands.condition_register_displacement();

        if !self.sr.condition(cc) {
            let counter = self.d[reg as usize] as i16 - 1;
            self.d_word(reg, counter as u16);

            if counter != -1 {
                self.pc = inst.pc + 2 + disp as u32;
            }
        }

        Ok(1)
    }

    pub(super) fn divs(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (reg, am) = inst.operands.register_effective_address();

        let mut ea = EffectiveAddress::new(am, Some(Size::Word));

        let src = self.get_word(memory, &mut ea)? as i16 as i32;
        let dst = self.d[reg as usize] as i32;

        if src == 0 {
            Err(Vector::ZeroDivide as u8)
        } else {
            let quot = dst / src;
            let rem = dst % src;
            self.d[reg as usize] = (rem as u16 as u32) << 16 | (quot as u16 as u32);

            self.sr.n = quot < 0;
            self.sr.z = quot == 0;
            self.sr.v = quot < i16::MIN as i32 || quot > i16::MAX as i32;
            self.sr.c = false;

            Ok(1)
        }
    }

    pub(super) fn divu(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (reg, am) = inst.operands.register_effective_address();

        let mut ea = EffectiveAddress::new(am, Some(Size::Word));

        let src = self.get_word(memory, &mut ea)? as u32;
        let dst = self.d[reg as usize];

        if src == 0 {
            Err(Vector::ZeroDivide as u8)
        } else {
            let quot = dst / src;
            let rem = dst % src;
            self.d[reg as usize] = (rem as u16 as u32) << 16 | (quot as u16 as u32);

            self.sr.n = quot & 0x0000_8000 != 0;
            self.sr.z = quot == 0;
            self.sr.v = (quot as i32) < i16::MIN as i32 || quot > i16::MAX as u32;
            self.sr.c = false;

            Ok(1)
        }
    }

    pub(super) fn eor(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (reg, _, size, am) = inst.operands.register_direction_size_effective_address();

        let mut ea = EffectiveAddress::new(am, Some(size));

        if size.is_byte() {
            let src = self.d[reg as usize] as u8;
            let dst = self.get_byte(memory, &mut ea)?;

            let res = src ^ dst;

            self.sr.n = res & SIGN_BIT_8 != 0;
            self.sr.z = res == 0;

            self.set_byte(memory, &mut ea, res)?;
        } else if size.is_word() {
            let src = self.d[reg as usize] as u16;
            let dst = self.get_word(memory, &mut ea)?;

            let res = src ^ dst;

            self.sr.n = res & SIGN_BIT_16 != 0;
            self.sr.z = res == 0;

            self.set_word(memory, &mut ea, res)?;
        } else {
            let src = self.d[reg as usize];
            let dst = self.get_long(memory, &mut ea)?;

            let res = src ^ dst;

            self.sr.n = res & SIGN_BIT_32 != 0;
            self.sr.z = res == 0;

            self.set_long(memory, &mut ea, res)?;
        }

        self.sr.v = false;
        self.sr.c = false;

        Ok(1)
    }

    pub(super) fn eori(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (size, am, imm) = inst.operands.size_effective_address_immediate();

        let mut ea = EffectiveAddress::new(am, Some(size));

        if size.is_byte() {
            let data = self.get_byte(memory, &mut ea)? ^ imm as u8;
            self.set_byte(memory, &mut ea, data)?;

            self.sr.n = data & SIGN_BIT_8 != 0;
            self.sr.z = data == 0;
        } else if size.is_word() {
            let data = self.get_word(memory, &mut ea)? ^ imm as u16;
            self.set_word(memory, &mut ea, data)?;

            self.sr.n = data & SIGN_BIT_16 != 0;
            self.sr.z = data == 0;
        } else {
            let data = self.get_long(memory, &mut ea)? ^ imm;
            self.set_long(memory, &mut ea, data)?;

            self.sr.n = data & SIGN_BIT_32 != 0;
            self.sr.z = data == 0;
        }

        self.sr.v = false;
        self.sr.c = false;

        Ok(1)
    }

    pub(super) fn eoriccr(&mut self, _: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let imm = inst.operands.immediate();

        self.sr ^= imm;

        Ok(1)
    }

    pub(super) fn eorisr(&mut self, _: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        if self.sr.s {
            let imm = inst.operands.immediate();
            self.sr ^= imm;
            Ok(1)
        } else {
            Err(Vector::PrivilegeViolation as u8)
        }
    }

    pub(super) fn exg(&mut self, _: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (rx, mode, ry) = inst.operands.register_opmode_register();

        if mode == Direction::ExchangeData {
            self.d.swap(rx as usize, ry as usize);
        } else if mode == Direction::ExchangeAddress {
            // TODO: change to std::mem::swap when new borrow checker is available
            let y = self.a(ry);
            *self.a_mut(ry) = self.a(rx);
            *self.a_mut(rx) = y;
        } else {
            let y = self.a(ry);
            *self.a_mut(ry) = self.d[rx as usize];
            self.d[rx as usize] = y;
        }

        Ok(1)
    }

    pub(super) fn ext(&mut self, _: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (mode, reg) = inst.operands.opmode_register();

        if mode == 0b010 {
            let d = self.d[reg as usize] as i8 as u16;
            self.d_word(reg, d);
        } else {
            self.d[reg as usize] = self.d[reg as usize] as i16 as u32;
        }

        self.sr.n = self.d[reg as usize] & SIGN_BIT_32 != 0;
        self.sr.z = self.d[reg as usize] == 0;
        self.sr.v = false;
        self.sr.c = false;

        Ok(1)
    }

    pub(super) fn illegal(&mut self, _: &mut impl MemoryAccess, _: &Instruction) -> InterpreterResult {
        Err(Vector::IllegalInstruction as u8)
    }

    pub(super) fn jmp(&mut self, _: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let am = inst.operands.effective_address();

        let mut ea = EffectiveAddress::new(am, None);

        self.pc = self.get_effective_address(&mut ea).unwrap();

        Ok(1)
    }

    pub(super) fn jsr(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let am = inst.operands.effective_address();

        let mut ea = EffectiveAddress::new(am, None);

        self.push_long(memory, self.pc)?;
        self.pc = self.get_effective_address(&mut ea).unwrap();

        Ok(1)
    }

    pub(super) fn lea(&mut self, _: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (reg, am) = inst.operands.register_effective_address();

        let mut ea = EffectiveAddress::new(am, None);

        *self.a_mut(reg) = self.get_effective_address(&mut ea).unwrap();

        Ok(1)
    }

    pub(super) fn link(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (reg, disp) = inst.operands.register_displacement();

        self.push_long(memory, self.a(reg))?;
        *self.a_mut(reg) = self.sp();
        *self.sp_mut() += disp as u32;

        Ok(1)
    }

    pub(super) fn lsm(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (dir, am) = inst.operands.direction_effective_address();

        let mut ea = EffectiveAddress::new(am, Some(Size::Word));

        let mut data = self.get_word(memory, &mut ea)?;

        if dir == Direction::Left {
            let sign = data & SIGN_BIT_16;
            data <<= 1;
            self.sr.x = sign != 0;
            self.sr.c = sign != 0;
        } else {
            let bit = data & 1;
            data >>= 1;
            self.sr.x = bit != 0;
            self.sr.c = bit != 0;
        }

        self.sr.n = data & SIGN_BIT_16 != 0;
        self.sr.z = data == 0;
        self.sr.v = false;

        self.set_word(memory, &mut ea, data)?;

        Ok(1)
    }

    pub(super) fn lsr(&mut self, _: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (rot, dir, size, mode, reg) = inst.operands.rotation_direction_size_mode_register();

        self.sr.v = false;
        self.sr.c = false;

        let shift_count = if mode == 1 {
            (self.d[rot as usize] % 64) as u8
        } else {
            if rot == 0 { 8 } else { rot }
        };

        let (mut data, mask) = if size == Size::Byte {
            (self.d[reg as usize] & 0x0000_00FF, SIGN_BIT_8 as u32)
        } else if size == Size::Word {
            (self.d[reg as usize] & 0x0000_FFFF, SIGN_BIT_16 as u32)
        } else {
            (self.d[reg as usize], SIGN_BIT_32)
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

        Ok(1)
    }

    pub(super) fn r#move(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (size, amdst, amsrc) = inst.operands.size_effective_address_effective_address();

        let mut src = EffectiveAddress::new(amsrc, Some(size));
        let mut dst = EffectiveAddress::new(amdst, Some(size));

        if size.is_byte() {
            let d = self.get_byte(memory, &mut src)?;
            self.set_byte(memory, &mut dst, d)?;
            self.sr.n = d & SIGN_BIT_8 != 0;
            self.sr.z = d == 0;
        } else if size.is_word() {
            let d = self.get_word(memory, &mut src)?;
            self.set_word(memory, &mut dst, d)?;
            self.sr.n = d & SIGN_BIT_16 != 0;
            self.sr.z = d == 0;
        } else {
            let d = self.get_long(memory, &mut src)?;
            self.set_long(memory, &mut dst, d)?;
            self.sr.n = d & SIGN_BIT_32 != 0;
            self.sr.z = d == 0;
        }

        self.sr.v = false;
        self.sr.c = false;

        Ok(1)
    }

    pub(super) fn movea(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (size, reg, am) = inst.operands.size_register_effective_address();

        let mut ea = EffectiveAddress::new(am, Some(size));

        *self.a_mut(reg) = if size.is_word() {
            self.get_word(memory, &mut ea)? as i16 as u32
        } else {
            self.get_long(memory, &mut ea)?
        };

        Ok(1)
    }

    pub(super) fn moveccr(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let am = inst.operands.effective_address();

        let mut ea = EffectiveAddress::new(am, Some(Size::Word));

        let ccr = self.get_word(memory, &mut ea)?;
        self.sr.set_ccr(ccr);

        Ok(1)
    }

    pub(super) fn movefsr(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let am = inst.operands.effective_address();

        let mut ea = EffectiveAddress::new(am, Some(Size::Word));

        self.set_word(memory, &mut ea, self.sr.into())?;

        Ok(1)
    }

    pub(super) fn movesr(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        if self.sr.s {
            let am = inst.operands.effective_address();
            let mut ea = EffectiveAddress::new(am, Some(Size::Word));

            let sr = self.get_word(memory, &mut ea)?;
            self.sr = sr.into();
            Ok(1)
        } else {
            Err(Vector::PrivilegeViolation as u8)
        }
    }

    pub(super) fn moveusp(&mut self, _: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        if self.sr.s {
            let (d, reg) = inst.operands.direction_register();
            if d == Direction::UspToRegister {
                *self.a_mut(reg) = self.usp;
            } else {
                self.usp = self.a(reg);
            }
            Ok(1)
        } else {
            Err(Vector::PrivilegeViolation as u8)
        }
    }

    pub(super) fn movem(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (dir, size, am, mut list) = inst.operands.direction_size_effective_address_list();

        let mut ea = EffectiveAddress::new(am, Some(size));

        let gap = size as u32;
        let eareg = ea.mode.register().unwrap_or(u8::MAX);

        if ea.mode.is_ariwpr() {
            let mut addr = self.a(eareg);

            for reg in (0..8).rev() {
                if list & 1 != 0 {
                    addr -= gap;
                    if size.is_word() { memory.set_word(addr, self.a(reg) as u16)?; }
                    else { memory.set_long(addr, self.a(reg))?; }
                }

                list >>= 1;
            }

            for reg in (0..8).rev() {
                if list & 1 != 0 {
                    addr -= gap;
                    if size.is_word() { memory.set_word(addr, self.d[reg] as u16)?; }
                    else { memory.set_long(addr, self.d[reg])?; }
                }

                list >>= 1;
            }

            *self.a_mut(eareg) = addr;
        } else {
            let mut addr = if ea.mode.is_ariwpo() {
                self.a(eareg)
            } else {
                self.get_effective_address(&mut ea).unwrap()
            };

            for reg in 0..8 {
                if list & 1 != 0 {
                    if dir == Direction::MemoryToRegister {
                        let value = if size.is_word() { memory.get_word(addr)? as i16 as u32 }
                            else { memory.get_long(addr)? };
                        self.d[reg] = value;
                    } else {
                        if size.is_word() { memory.set_word(addr, self.d[reg] as u16)?; }
                        else { memory.set_long(addr, self.d[reg])?; }
                    }

                    addr += gap;
                }

                list >>= 1;
            }

            for reg in 0..8 {
                if list & 1 != 0 {
                    if dir == Direction::MemoryToRegister {
                        let value = if size.is_word() { memory.get_word(addr)? as i16 as u32 }
                            else { memory.get_long(addr)? };
                        *self.a_mut(reg) = value;
                    } else {
                        if size.is_word() { memory.set_word(addr, self.a(reg as u8) as u16)?; }
                        else { memory.set_long(addr, self.a(reg as u8))?; }
                    }

                    addr += gap;
                }

                list >>= 1;
            }

            if ea.mode.is_ariwpo() {
                *self.a_mut(eareg) = addr;
            }
        }

        Ok(1)
    }

    pub(super) fn movep(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (data, dir, size, addr, disp) = inst.operands.register_direction_size_register_displacement();

        let mut shift = if size.is_word() { 8 } else { 24 };
        let mut addr = self.a(addr) + disp as u32;

        if dir == Direction::RegisterToMemory {
            while shift >= 0 {
                let d = (self.d[data as usize] >> shift) as u8;
                memory.set_byte(addr, d)?;
                shift -= 8;
                addr += 2;
            }
        } else {
            if size.is_word() { self.d[data as usize] &= 0xFFFF_0000 } else { self.d[data as usize] = 0 }

            while shift >= 0 {
                let d = memory.get_byte(addr)? as u32;
                self.d[data as usize] |= d << shift;
                shift -= 8;
                addr += 2;
            }
        }

        Ok(1)
    }

    pub(super) fn moveq(&mut self, _: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (reg, data) = inst.operands.register_data();

        self.d[reg as usize] = data as u32;

        self.sr.n = data <  0;
        self.sr.z = data == 0;
        self.sr.v = false;
        self.sr.c = false;

        Ok(1)
    }

    pub(super) fn muls(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (reg, am) = inst.operands.register_effective_address();

        let mut ea = EffectiveAddress::new(am, Some(Size::Word));

        let src = self.get_word(memory, &mut ea)? as i16 as i32;
        let dst = self.d[reg as usize] as i16 as i32;

        let res = src * dst;
        self.d[reg as usize] = res as u32;

        self.sr.n = res < 0;
        self.sr.z = res == 0;
        self.sr.v = false;
        self.sr.c = false;

        Ok(1)
    }

    pub(super) fn mulu(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (reg, am) = inst.operands.register_effective_address();

        let mut ea = EffectiveAddress::new(am, Some(Size::Word));

        let src = self.get_word(memory, &mut ea)? as u32;
        let dst = self.d[reg as usize] as u16 as u32;

        let res = src * dst;
        self.d[reg as usize] = res;

        self.sr.n = res & SIGN_BIT_32 != 0;
        self.sr.z = res == 0;
        self.sr.v = false;
        self.sr.c = false;

        Ok(1)
    }

    pub(super) fn nbcd(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let am = inst.operands.effective_address();

        let mut ea = EffectiveAddress::new(am, Some(Size::Byte));

        let data = self.get_byte(memory, &mut ea)?;

        let low = 0 - (data as i8 & 0x0F) - self.sr.x as i8;
        let high = 0 - (data as i8 >> 4 & 0x0F) - (low < 0) as i8;
        let res = (if high < 0 { 10 + high } else { high } as u8) << 4 |
                      if low < 0 { 10 + low } else { low } as u8;

        self.set_byte(memory, &mut ea, res)?;

        if res != 0 { self.sr.z = false; }
        self.sr.c = res != 0;
        self.sr.x = self.sr.c;

        Ok(1)
    }

    pub(super) fn neg(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (size, am) = inst.operands.size_effective_address();

        let mut ea = EffectiveAddress::new(am, Some(size));

        if size.is_byte() {
            let data = -(self.get_byte(memory, &mut ea)? as i8);
            self.set_byte(memory, &mut ea, data as u8)?;

            self.sr.n = data < 0;
            self.sr.z = data == 0;
            self.sr.v = data == i8::MIN;
            self.sr.c = data != 0;
            self.sr.x = self.sr.c;
        } else if size.is_word() {
            let data = -(self.get_word(memory, &mut ea)? as i16);
            self.set_word(memory, &mut ea, data as u16)?;

            self.sr.n = data < 0;
            self.sr.z = data == 0;
            self.sr.v = data == i16::MIN;
            self.sr.c = data != 0;
            self.sr.x = self.sr.c;
        } else {
            let data = -(self.get_long(memory, &mut ea)? as i32);
            self.set_long(memory, &mut ea, data as u32)?;

            self.sr.n = data < 0;
            self.sr.z = data == 0;
            self.sr.v = data == i32::MIN;
            self.sr.c = data != 0;
            self.sr.x = self.sr.c;
        }

        Ok(1)
    }

    pub(super) fn negx(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (size, am) = inst.operands.size_effective_address();

        let mut ea = EffectiveAddress::new(am, Some(size));

        // using overflowing_sub indicates an overflow when negating -128 with the X flag set.
        // 0 - -128 stays -128, then -128 - 1 gives 127, which is an overflow.
        // However I don't know if the hardware has intermediate overflow.
        // The other way is 0 - -128 gives 128, then 128 - 1 gives 127 which generates no overflow.
        // TODO: test what the hardware actually does.
        if size.is_byte() {
            let data = self.get_byte(memory, &mut ea)? as i8;
            let res = 0 - data - self.sr.x as i8;
            let vres = 0 - data as i16 - self.sr.x as i16;
            let (_, c) = 0u8.extended_sub(data as u8, self.sr.x);
            self.set_byte(memory, &mut ea, res as u8)?;

            self.sr.x = c;
            self.sr.n = res < 0;
            if res != 0 { self.sr.z = false };
            self.sr.v = vres < i8::MIN as i16 || vres > i8::MAX as i16;
            self.sr.c = c;
        } else if size.is_word() {
            let data = self.get_word(memory, &mut ea)? as i16;
            let res = 0 - data - self.sr.x as i16;
            let vres = 0 - data as i32 - self.sr.x as i32;
            let (_, c) = 0u16.extended_sub(data as u16, self.sr.x);
            self.set_word(memory, &mut ea, res as u16)?;

            self.sr.x = c;
            self.sr.n = res < 0;
            if res != 0 { self.sr.z = false };
            self.sr.v = vres < i16::MIN as i32 || vres > i16::MAX as i32;
            self.sr.c = c;
        } else {
            let data = self.get_long(memory, &mut ea)? as i32;
            let res = 0 - data - self.sr.x as i32;
            let vres = 0 - data as i64 - self.sr.x as i64;
            let (_, c) = 0u32.extended_sub(data as u32, self.sr.x);
            self.set_long(memory, &mut ea, res as u32)?;

            self.sr.x = c;
            self.sr.n = res < 0;
            if res != 0 { self.sr.z = false };
            self.sr.v = vres < i32::MIN as i64 || vres > i32::MAX as i64;
            self.sr.c = c;
        }

        Ok(1)
    }

    pub(super) fn nop(&mut self, _: &mut impl MemoryAccess, _: &Instruction) -> InterpreterResult {
        Ok(1)
    }

    pub(super) fn not(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (size, am) = inst.operands.size_effective_address();

        let mut ea = EffectiveAddress::new(am, Some(size));

        if size.is_byte() {
            let data = !self.get_byte(memory, &mut ea)?;
            self.set_byte(memory, &mut ea, data)?;

            self.sr.n = data & SIGN_BIT_8 != 0;
            self.sr.z = data == 0;
        } else if size.is_word() {
            let data = !self.get_word(memory, &mut ea)?;
            self.set_word(memory, &mut ea, data)?;

            self.sr.n = data & SIGN_BIT_16 != 0;
            self.sr.z = data == 0;
        } else {
            let data = !self.get_long(memory, &mut ea)?;
            self.set_long(memory, &mut ea, data)?;

            self.sr.n = data & SIGN_BIT_32 != 0;
            self.sr.z = data == 0;
        }

        self.sr.v = false;
        self.sr.c = false;

        Ok(1)
    }

    pub(super) fn or(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (reg, dir, size, am) = inst.operands.register_direction_size_effective_address();

        let mut ea = EffectiveAddress::new(am, Some(size));

        if size.is_byte() {
            let src = self.d[reg as usize] as u8;
            let dst = self.get_byte(memory, &mut ea)?;

            let res = src | dst;

            self.sr.n = res & SIGN_BIT_8 != 0;
            self.sr.z = res == 0;

            if dir == Direction::DstEa {
                self.set_byte(memory, &mut ea, res)?;
            } else {
                self.d_byte(reg, res);
            }
        } else if size.is_word() {
            let src = self.d[reg as usize] as u16;
            let dst = self.get_word(memory, &mut ea)?;

            let res = src | dst;

            self.sr.n = res & SIGN_BIT_16 != 0;
            self.sr.z = res == 0;

            if dir == Direction::DstEa {
                self.set_word(memory, &mut ea, res)?;
            } else {
                self.d_word(reg, res);
            }
        } else {
            let src = self.d[reg as usize];
            let dst = self.get_long(memory, &mut ea)?;

            let res = src | dst;

            self.sr.n = res & SIGN_BIT_32 != 0;
            self.sr.z = res == 0;

            if dir == Direction::DstEa {
                self.set_long(memory, &mut ea, res)?;
            } else {
                self.d[reg as usize] = res;
            }
        }

        self.sr.v = false;
        self.sr.c = false;

        Ok(1)
    }

    pub(super) fn ori(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (size, am, imm) = inst.operands.size_effective_address_immediate();

        let mut ea = EffectiveAddress::new(am, Some(size));

        if size.is_byte() {
            let data = self.get_byte(memory, &mut ea)? | imm as u8;
            self.set_byte(memory, &mut ea, data)?;

            self.sr.n = data & SIGN_BIT_8 != 0;
            self.sr.z = data == 0;
        } else if size.is_word() {
            let data = self.get_word(memory, &mut ea)? | imm as u16;
            self.set_word(memory, &mut ea, data)?;

            self.sr.n = data & SIGN_BIT_16 != 0;
            self.sr.z = data == 0;
        } else {
            let data = self.get_long(memory, &mut ea)? | imm;
            self.set_long(memory, &mut ea, data)?;

            self.sr.n = data & SIGN_BIT_32 != 0;
            self.sr.z = data == 0;
        }

        self.sr.v = false;
        self.sr.c = false;

        Ok(1)
    }

    pub(super) fn oriccr(&mut self, _: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let imm = inst.operands.immediate();

        self.sr |= imm;

        Ok(1)
    }

    pub(super) fn orisr(&mut self, _: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        if self.sr.s {
            let imm = inst.operands.immediate();
            self.sr |= imm;
            Ok(1)
        } else {
            Err(Vector::PrivilegeViolation as u8)
        }
    }

    pub(super) fn pea(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let am = inst.operands.effective_address();

        let mut ea = EffectiveAddress::new(am, None);

        let addr = self.get_effective_address(&mut ea).unwrap();
        self.push_long(memory, addr)?;

        Ok(1)
    }

    pub(super) fn reset(&mut self, memory: &mut impl MemoryAccess, _: &Instruction) -> InterpreterResult {
        if self.sr.s {
            memory.reset();
            Ok(1)
        } else {
            Err(Vector::PrivilegeViolation as u8)
        }
    }

    pub(super) fn rom(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (dir, am) = inst.operands.direction_effective_address();

        let mut ea = EffectiveAddress::new(am, Some(Size::Word));

        let mut data = self.get_word(memory, &mut ea)?;
        let sign = data & SIGN_BIT_16;

        if dir == Direction::Left {
            data <<= 1;
            data |= (sign != 0) as u16;
            self.sr.c = sign != 0;
        } else {
            let bit = data & 1;
            data >>= 1;
            if bit != 0 {
                data |= SIGN_BIT_16;
            }
            self.sr.c = bit != 0;
        }

        self.sr.n = data & SIGN_BIT_16 != 0;
        self.sr.z = data == 0;
        self.sr.v = false;

        self.set_word(memory, &mut ea, data)?;

        Ok(1)
    }

    pub(super) fn ror(&mut self, _: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (rot, dir, size, mode, reg) = inst.operands.rotation_direction_size_mode_register();

        self.sr.v = false;
        self.sr.c = false;

        let shift_count = if mode == 1 {
            (self.d[rot as usize] % 64) as u8
        } else {
            if rot == 0 { 8 } else { rot }
        };

        let (mut data, mask) = if size == Size::Byte {
            (self.d[reg as usize] & 0x0000_00FF, SIGN_BIT_8 as u32)
        } else if size == Size::Word {
            (self.d[reg as usize] & 0x0000_FFFF, SIGN_BIT_16 as u32)
        } else {
            (self.d[reg as usize], SIGN_BIT_32)
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

        Ok(1)
    }

    pub(super) fn roxm(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (dir, am) = inst.operands.direction_effective_address();

        let mut ea = EffectiveAddress::new(am, Some(Size::Word));

        let mut data = self.get_word(memory, &mut ea)?;
        let sign = data & SIGN_BIT_16;

        if dir == Direction::Left {
            data <<= 1;
            data |= self.sr.x as u16;
            self.sr.x = sign != 0;
            self.sr.c = sign != 0;
        } else {
            let bit = data & 1;
            data >>= 1;
            if self.sr.x {
                data |= SIGN_BIT_16;
            }
            self.sr.x = bit != 0;
            self.sr.c = bit != 0;
        }

        self.sr.n = data & SIGN_BIT_16 != 0;
        self.sr.z = data == 0;
        self.sr.v = false;

        self.set_word(memory, &mut ea, data)?;

        Ok(1)
    }

    pub(super) fn roxr(&mut self, _: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (rot, dir, size, mode, reg) = inst.operands.rotation_direction_size_mode_register();

        self.sr.v = false;
        self.sr.c = self.sr.x;

        let shift_count = if mode == 1 {
            (self.d[rot as usize] % 64) as u8
        } else {
            if rot == 0 { 8 } else { rot }
        };

        let (mut data, mask) = if size == Size::Byte {
            (self.d[reg as usize] & 0x0000_00FF, SIGN_BIT_8 as u32)
        } else if size == Size::Word {
            (self.d[reg as usize] & 0x0000_FFFF, SIGN_BIT_16 as u32)
        } else {
            (self.d[reg as usize], SIGN_BIT_32)
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

        Ok(1)
    }

    pub(super) fn rte(&mut self, memory: &mut impl MemoryAccess, _: &Instruction) -> InterpreterResult {
        if self.sr.s {
            let sr = self.pop_word(memory)?;
            self.pc = self.pop_long(memory)?;

            if !self.stack_format.is_68000() {
                let format = self.pop_word(memory)?;

                if self.stack_format.is_68010() && format & 0xF000 == 0x8000 ||
                   self.stack_format.is_68070() && format & 0xF000 == 0xF000 {
                    *self.sp_mut() += 26;
                } else if format & 0xF000 != 0 {
                    return Err(Vector::FormatError as u8);
                }
            }

            self.sr = sr.into();

            Ok(1)
        } else {
            Err(Vector::PrivilegeViolation as u8)
        }
    }

    pub(super) fn rtr(&mut self, memory: &mut impl MemoryAccess, _: &Instruction) -> InterpreterResult {
        let ccr = self.pop_word(memory)?;
        self.sr &= SR_UPPER_MASK;
        self.sr |= ccr & CCR_MASK;
        self.pc = self.pop_long(memory)?;

        Ok(1)
    }

    pub(super) fn rts(&mut self, memory: &mut impl MemoryAccess, _: &Instruction) -> InterpreterResult {
        self.pc = self.pop_long(memory)?;

        Ok(1)
    }

    pub(super) fn sbcd(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (ry, _, mode, rx) = inst.operands.register_size_mode_register();

        let (src, dst) = if mode == Direction::MemoryToMemory {
            let src_addr = self.ariwpr(rx, Size::Byte);
            let dst_addr = self.ariwpr(ry, Size::Byte);
            (memory.get_byte(src_addr)?, memory.get_byte(dst_addr)?)
        } else {
            (self.d[rx as usize] as u8, self.d[ry as usize] as u8)
        };

        let low = (dst as i8 & 0x0F) - (src as i8 & 0x0F) - self.sr.x as i8;
        let high = (dst as i8 >> 4 & 0x0F) - (src as i8 >> 4 & 0x0F) - (low < 0) as i8;
        let res = (if high < 0 { 10 + high } else { high } as u8) << 4 |
                      if low < 0 { 10 + low } else { low } as u8;

        if mode == Direction::MemoryToMemory {
            memory.set_byte(self.a(ry), res)?;
        } else {
            self.d_byte(ry, res);
        }

        if res != 0 { self.sr.z = false; }
        self.sr.c = high < 0;
        self.sr.x = self.sr.c;

        Ok(1)
    }

    pub(super) fn scc(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (cc, am) = inst.operands.condition_effective_address();

        let mut ea = EffectiveAddress::new(am, Some(Size::Byte));

        if self.sr.condition(cc) {
            self.set_byte(memory, &mut ea, 0xFF)?;
        } else {
            self.set_byte(memory, &mut ea, 0)?;
        }

        Ok(1)
    }

    pub(super) fn stop(&mut self, _: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let imm = inst.operands.immediate();

        if self.sr.s {
            // TODO: trace.
            self.sr = imm.into();
            self.stop = true;
            // TODO: how to regain control?
            Ok(1)
        } else {
            Err(Vector::PrivilegeViolation as u8)
        }
    }

    pub(super) fn sub(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (reg, dir, size, am) = inst.operands.register_direction_size_effective_address();

        let mut ea = EffectiveAddress::new(am, Some(size));

        if size.is_byte() {
            let (src, dst) = if dir == Direction::DstEa {
                (self.d[reg as usize] as u8, self.get_byte(memory, &mut ea)?)
            } else {
                (self.get_byte(memory, &mut ea)?, self.d[reg as usize] as u8)
            };

            let (res, v) = (dst as i8).overflowing_sub(src as i8);
            let (_, c) = dst.overflowing_sub(src);

            self.sr.x = c;
            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;

            if dir == Direction::DstEa {
                self.set_byte(memory, &mut ea, res as u8)?;
            } else {
                self.d_byte(reg, res as u8);
            }
        } else if size.is_word() {
            let (src, dst) = if dir == Direction::DstEa {
                (self.d[reg as usize] as u16, self.get_word(memory, &mut ea)?)
            } else {
                (self.get_word(memory, &mut ea)?, self.d[reg as usize] as u16)
            };

            let (res, v) = (dst as i16).overflowing_sub(src as i16);
            let (_, c) = dst.overflowing_sub(src);

            self.sr.x = c;
            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;

            if dir == Direction::DstEa {
                self.set_word(memory, &mut ea, res as u16)?;
            } else {
                self.d_word(reg, res as u16);
            }
        } else {
            let (src, dst) = if dir == Direction::DstEa {
                (self.d[reg as usize] as u32, self.get_long(memory, &mut ea)?)
            } else {
                (self.get_long(memory, &mut ea)?, self.d[reg as usize] as u32)
            };

            let (res, v) = (dst as i32).overflowing_sub(src as i32);
            let (_, c) = dst.overflowing_sub(src);

            self.sr.x = c;
            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;

            if dir == Direction::DstEa {
                self.set_long(memory, &mut ea, res as u32)?;
            } else {
                self.d[reg as usize] = res as u32;
            }
        }

        Ok(1)
    }

    pub(super) fn suba(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (reg, size, am) = inst.operands.register_size_effective_address();

        let mut ea = EffectiveAddress::new(am, Some(size));

        let src = if size.is_word() {
            self.get_word(memory, &mut ea)? as i16 as u32
        } else {
            self.get_long(memory, &mut ea)?
        };

        *self.a_mut(reg) -= src;

        Ok(1)
    }

    pub(super) fn subi(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (size, am, imm) = inst.operands.size_effective_address_immediate();

        let mut ea = EffectiveAddress::new(am, Some(size));

        if size.is_byte() {
            let data = self.get_byte(memory, &mut ea)?;
            let (res, v) = (data as i8).overflowing_sub(imm as i8);
            let (_, c) = data.overflowing_sub(imm as u8);
            self.set_byte(memory, &mut ea, res as u8)?;

            self.sr.x = c;
            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        } else if size.is_word() {
            let data = self.get_word(memory, &mut ea)?;
            let (res, v) = (data as i16).overflowing_sub(imm as i16);
            let (_, c) = data.overflowing_sub(imm as u16);
            self.set_word(memory, &mut ea, res as u16)?;

            self.sr.x = c;
            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        } else {
            let data = self.get_long(memory, &mut ea)?;
            let (res, v) = (data as i32).overflowing_sub(imm as i32);
            let (_, c) = data.overflowing_sub(imm);
            self.set_long(memory, &mut ea, res as u32)?;

            self.sr.x = c;
            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        }

        Ok(1)
    }

    pub(super) fn subq(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (imm, size, am) = inst.operands.data_size_effective_address();

        let mut ea = EffectiveAddress::new(am, Some(size));

        if size.is_byte() {
            let data = self.get_byte(memory, &mut ea)?;
            let (res, v) = (data as i8).overflowing_sub(imm as i8);
            let (_, c) = data.overflowing_sub(imm);
            self.set_byte(memory, &mut ea, res as u8)?;

            self.sr.x = c;
            self.sr.n = res < 0;
            self.sr.z = res == 0;
            self.sr.v = v;
            self.sr.c = c;
        } else if size.is_word() {
            let data = self.get_word(memory, &mut ea)?;
            let (res, v) = (data as i16).overflowing_sub(imm as i16);
            let (_, c) = data.overflowing_sub(imm as u16);
            self.set_word(memory, &mut ea, res as u16)?;

            if !ea.mode.is_ard() {
                self.sr.x = c;
                self.sr.n = res < 0;
                self.sr.z = res == 0;
                self.sr.v = v;
                self.sr.c = c;
            }
        } else {
            let data = self.get_long(memory, &mut ea)?;
            let (res, v) = (data as i32).overflowing_sub(imm as i32);
            let (_, c) = data.overflowing_sub(imm as u32);
            self.set_long(memory, &mut ea, res as u32)?;

            if !ea.mode.is_ard() {
                self.sr.x = c;
                self.sr.n = res < 0;
                self.sr.z = res == 0;
                self.sr.v = v;
                self.sr.c = c;
            }
        }

        Ok(1)
    }

    pub(super) fn subx(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (ry, size, mode, rx) = inst.operands.register_size_mode_register();

        if size.is_byte() {
            let (src, dst) = if mode == Direction::MemoryToMemory {
                let src_addr = self.ariwpr(rx, size);
                let dst_addr = self.ariwpr(ry, size);
                (memory.get_byte(src_addr)?, memory.get_byte(dst_addr)?)
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
                memory.set_byte(self.a(ry), res as u8)?;
            } else {
                self.d_byte(ry, res as u8);
            }
        } else if size.is_word() {
            let (src, dst) = if mode == Direction::MemoryToMemory {
                let src_addr = self.ariwpr(rx, size);
                let dst_addr = self.ariwpr(ry, size);
                (memory.get_word(src_addr)?, memory.get_word(dst_addr)?)
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
                memory.set_word(self.a(ry), res as u16)?;
            } else {
                self.d_word(ry, res as u16);
            }
        } else {
            let (src, dst) = if mode == Direction::MemoryToMemory {
                let src_addr = self.ariwpr(rx, size);
                let dst_addr = self.ariwpr(ry, size);
                (memory.get_long(src_addr)?, memory.get_long(dst_addr)?)
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
                memory.set_long(self.a(ry), res as u32)?;
            } else {
                self.d[ry as usize] = res as u32;
            }
        }

        Ok(1)
    }

    pub(super) fn swap(&mut self, _: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let reg = inst.operands.register();

        let high = self.d[reg as usize] >> 16;
        self.d[reg as usize] <<= 16;
        self.d[reg as usize] |= high;

        self.sr.n = self.d[reg as usize] & SIGN_BIT_32 != 0;
        self.sr.z = self.d[reg as usize] == 0;
        self.sr.v = false;
        self.sr.c = false;

        Ok(1)
    }

    pub(super) fn tas(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let am = inst.operands.effective_address();

        let mut ea = EffectiveAddress::new(am, Some(Size::Byte));

        let mut data = self.get_byte(memory, &mut ea)?;

        self.sr.n = data & SIGN_BIT_8 != 0;
        self.sr.z = data == 0;
        self.sr.v = false;
        self.sr.c = false;

        data |= SIGN_BIT_8;
        self.set_byte(memory, &mut ea, data)?;

        Ok(1)
    }

    pub(super) fn trap(&mut self, _: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let vector = inst.operands.vector();
        Err(Vector::Trap0Instruction as u8 + vector)
    }

    pub(super) fn trapv(&mut self, _: &mut impl MemoryAccess, _: &Instruction) -> InterpreterResult {
        if self.sr.v {
            Err(Vector::TrapVInstruction as u8)
        } else {
            Ok(1)
        }
    }

    pub(super) fn tst(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let (size, am) = inst.operands.size_effective_address();

        let mut ea = EffectiveAddress::new(am, Some(size));

        if size.is_byte() {
            let data = self.get_byte(memory, &mut ea)?;
            self.sr.n = data & SIGN_BIT_8 != 0;
            self.sr.z = data == 0;
        } else if size.is_word() {
            let data = self.get_word(memory, &mut ea)?;
            self.sr.n = data & SIGN_BIT_16 != 0;
            self.sr.z = data == 0;
        } else {
            let data = self.get_long(memory, &mut ea)?;
            self.sr.n = data & SIGN_BIT_32 != 0;
            self.sr.z = data == 0;
        }

        self.sr.v = false;
        self.sr.c = false;

        Ok(1)
    }

    pub(super) fn unlk(&mut self, memory: &mut impl MemoryAccess, inst: &Instruction) -> InterpreterResult {
        let reg = inst.operands.register();

        *self.sp_mut() = self.a(reg);
        *self.a_mut(reg) = self.pop_long(memory)?;

        Ok(1)
    }
}
