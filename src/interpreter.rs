use crate::{M68000, MemoryAccess};
use crate::addressing_modes::{EffectiveAddress, AddressingMode};
use crate::exception::Vector;
use crate::execution_times as EXEC;
use crate::instruction::{Direction, Size};
use crate::utils::{BigInt, bits, IsEven};

pub(super) const SR_UPPER_MASK: u16 = 0xA700;
pub(super) const CCR_MASK: u16 = 0x001F;
pub(super) const SIGN_BIT_8: u8 = 0x80;
pub(super) const SIGN_BIT_16: u16 = 0x8000;
pub(super) const SIGN_BIT_32: u32 = 0x8000_0000;

/// Returns the execution time on success, an exception vector on error. Alias for `Result<usize, u8>`.
pub(super) type InterpreterResult = Result<usize, u8>;

// TODO: return a tuple with the current execution time and the exception that occured (CHK, DIVS, DIVU).
// All this for only 3 instructions ?

impl M68000 {
    #[must_use]
    const fn check_supervisor(&self) -> Result<(), u8> {
        if self.regs.sr.s {
            Ok(())
        } else {
            Err(Vector::PrivilegeViolation as u8)
        }
    }

    pub(super) fn execute_unknown_instruction(&self) -> InterpreterResult {
        Err(Vector::IllegalInstruction as u8)
    }

    pub(super) fn execute_abcd(&mut self, memory: &mut impl MemoryAccess, rx: u8, mode: Direction, ry: u8) -> InterpreterResult {
        let (src, dst) = if mode == Direction::MemoryToMemory {
            let src_addr = self.ariwpr(ry, Size::Byte);
            let dst_addr = self.ariwpr(rx, Size::Byte);
            (memory.get_byte(src_addr)?, memory.get_byte(dst_addr)?)
        } else {
            (self.regs.d[ry as usize] as u8, self.regs.d[rx as usize] as u8)
        };

        let low = (src & 0x0F) + (dst & 0x0F) + self.regs.sr.x as u8;
        let high = (src >> 4 & 0x0F) + (dst >> 4 & 0x0F) + (low > 10) as u8;
        let res = (high << 4) | low;

        if res != 0 { self.regs.sr.z = false; }
        self.regs.sr.c = high > 10;
        self.regs.sr.x = self.regs.sr.c;

        if mode == Direction::MemoryToMemory {
            memory.set_byte(self.a(rx), res)?;
            Ok(EXEC::ABCD_MEM)
        } else {
            self.d_byte(rx, res);
            Ok(EXEC::ABCD_REG)
        }
    }

    pub(super) fn execute_add(&mut self, memory: &mut impl MemoryAccess, reg: u8, dir: Direction, size: Size, am: AddressingMode) -> InterpreterResult {
        let mut exec_time;

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte => {
                let (src, dst) = if dir == Direction::DstEa {
                    exec_time = EXEC::ADD_MEM_BW;
                    (self.regs.d[reg as usize] as u8, self.get_byte(memory, &mut ea, &mut exec_time)?)
                } else {
                    exec_time = EXEC::ADD_REG_BW;
                    (self.get_byte(memory, &mut ea, &mut exec_time)?, self.regs.d[reg as usize] as u8)
                };

                let (res, v) = (src as i8).overflowing_add(dst as i8);
                let (_, c) = src.overflowing_add(dst);

                self.regs.sr.x = c;
                self.regs.sr.n = res < 0;
                self.regs.sr.z = res == 0;
                self.regs.sr.v = v;
                self.regs.sr.c = c;

                if dir == Direction::DstEa {
                    self.set_byte(memory, &mut ea, &mut exec_time, res as u8)?;
                } else {
                    self.d_byte(reg, res as u8);
                }
            },
            Size::Word => {
                let (src, dst) = if dir == Direction::DstEa {
                    exec_time = EXEC::ADD_MEM_BW;
                    (self.regs.d[reg as usize] as u16, self.get_word(memory, &mut ea, &mut exec_time)?)
                } else {
                    exec_time = EXEC::ADD_REG_BW;
                    (self.get_word(memory, &mut ea, &mut exec_time)?, self.regs.d[reg as usize] as u16)
                };

                let (res, v) = (src as i16).overflowing_add(dst as i16);
                let (_, c) = src.overflowing_add(dst);

                self.regs.sr.x = c;
                self.regs.sr.n = res < 0;
                self.regs.sr.z = res == 0;
                self.regs.sr.v = v;
                self.regs.sr.c = c;

                if dir == Direction::DstEa {
                    self.set_word(memory, &mut ea, &mut exec_time, res as u16)?;
                } else {
                    self.d_word(reg, res as u16);
                }
            },
            Size::Long => {
                let (src, dst) = if dir == Direction::DstEa {
                    exec_time = EXEC::ADD_MEM_L;
                    (self.regs.d[reg as usize] as u32, self.get_long(memory, &mut ea, &mut exec_time)?)
                } else {
                    exec_time = if am.is_dard() || am.is_immediate() { EXEC::ADD_REG_L_RDIMM } else { EXEC::ADD_REG_L };
                    (self.get_long(memory, &mut ea, &mut exec_time)?, self.regs.d[reg as usize] as u32)
                };

                let (res, v) = (src as i32).overflowing_add(dst as i32);
                let (_, c) = src.overflowing_add(dst);

                self.regs.sr.x = c;
                self.regs.sr.n = res < 0;
                self.regs.sr.z = res == 0;
                self.regs.sr.v = v;
                self.regs.sr.c = c;

                if dir == Direction::DstEa {
                    self.set_long(memory, &mut ea, &mut exec_time, res as u32)?;
                } else {
                    self.regs.d[reg as usize] = res as u32;
                }
            },
        }

        Ok(exec_time)
    }

    pub(super) fn execute_adda(&mut self, memory: &mut impl MemoryAccess, reg: u8, size: Size, am: AddressingMode) -> InterpreterResult {
        let mut exec_time;

        let mut ea = EffectiveAddress::new(am, Some(size));

        let src = if size.is_word() {
            exec_time = EXEC::ADDA_WORD;
            self.get_word(memory, &mut ea, &mut exec_time)? as i16 as u32
        } else {
            exec_time = if am.is_dard() || am.is_immediate() {
                EXEC::ADDA_LONG_RDIMM
            } else {
                EXEC::ADDA_LONG
            };
            self.get_long(memory, &mut ea, &mut exec_time)?
        };

        *self.a_mut(reg) += src;

        Ok(exec_time)
    }

    pub(super) fn execute_addi(&mut self, memory: &mut impl MemoryAccess, size: Size, am: AddressingMode, imm: u32) -> InterpreterResult {
        let mut exec_time;

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte =>  {
                exec_time = if am.is_drd() { EXEC::ADDI_REG_BW } else { EXEC::ADDI_MEM_BW };
                let data = self.get_byte(memory, &mut ea, &mut exec_time)?;
                let (res, v) = (data as i8).overflowing_add(imm as i8);
                let (_, c) = data.overflowing_add(imm as u8);
                self.set_byte(memory, &mut ea, &mut exec_time, res as u8)?;

                self.regs.sr.x = c;
                self.regs.sr.n = res < 0;
                self.regs.sr.z = res == 0;
                self.regs.sr.v = v;
                self.regs.sr.c = c;
            },
            Size::Word => {
                exec_time = if am.is_drd() { EXEC::ADDI_REG_BW } else { EXEC::ADDI_MEM_BW };
                let data = self.get_word(memory, &mut ea, &mut exec_time)?;
                let (res, v) = (data as i16).overflowing_add(imm as i16);
                let (_, c) = data.overflowing_add(imm as u16);
                self.set_word(memory, &mut ea, &mut exec_time, res as u16)?;

                self.regs.sr.x = c;
                self.regs.sr.n = res < 0;
                self.regs.sr.z = res == 0;
                self.regs.sr.v = v;
                self.regs.sr.c = c;
            },
            Size::Long => {
                exec_time = if am.is_drd() { EXEC::ADDI_REG_L } else { EXEC::ADDI_MEM_L };
                let data = self.get_long(memory, &mut ea, &mut exec_time)?;
                let (res, v) = (data as i32).overflowing_add(imm as i32);
                let (_, c) = data.overflowing_add(imm);
                self.set_long(memory, &mut ea, &mut exec_time, res as u32)?;

                self.regs.sr.x = c;
                self.regs.sr.n = res < 0;
                self.regs.sr.z = res == 0;
                self.regs.sr.v = v;
                self.regs.sr.c = c;
            },
        }

        Ok(exec_time)
    }

    pub(super) fn execute_addq(&mut self, memory: &mut impl MemoryAccess, imm: u8, size: Size, am: AddressingMode) -> InterpreterResult {
        let mut exec_time;

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte => {
                exec_time = if am.is_drd() { EXEC::ADDQ_REG_BW } else { EXEC::ADDQ_MEM_BW };
                let data = self.get_byte(memory, &mut ea, &mut exec_time)?;
                let (res, v) = (data as i8).overflowing_add(imm as i8);
                let (_, c) = data.overflowing_add(imm);
                self.set_byte(memory, &mut ea, &mut exec_time, res as u8)?;

                self.regs.sr.x = c;
                self.regs.sr.n = res < 0;
                self.regs.sr.z = res == 0;
                self.regs.sr.v = v;
                self.regs.sr.c = c;
            },
            Size::Word => {
                exec_time = if am.is_dard() { EXEC::ADDQ_REG_BW } else { EXEC::ADDQ_MEM_BW };
                let data = self.get_word(memory, &mut ea, &mut exec_time)?;
                let (res, v) = (data as i16).overflowing_add(imm as i16);
                let (_, c) = data.overflowing_add(imm as u16);
                self.set_word(memory, &mut ea, &mut exec_time, res as u16)?;

                if !ea.mode.is_ard() {
                    self.regs.sr.x = c;
                    self.regs.sr.n = res < 0;
                    self.regs.sr.z = res == 0;
                    self.regs.sr.v = v;
                    self.regs.sr.c = c;
                }
            },
            Size::Long => {
                exec_time = if am.is_dard() { EXEC::ADDQ_REG_L } else { EXEC::ADDQ_MEM_L };
                let data = self.get_long(memory, &mut ea, &mut exec_time)?;
                let (res, v) = (data as i32).overflowing_add(imm as i32);
                let (_, c) = data.overflowing_add(imm as u32);
                self.set_long(memory, &mut ea, &mut exec_time, res as u32)?;

                if !ea.mode.is_ard() {
                    self.regs.sr.x = c;
                    self.regs.sr.n = res < 0;
                    self.regs.sr.z = res == 0;
                    self.regs.sr.v = v;
                    self.regs.sr.c = c;
                }
            },
        }

        Ok(exec_time)
    }

    pub(super) fn execute_addx(&mut self, memory: &mut impl MemoryAccess, rx: u8, size: Size, mode: Direction, ry: u8) -> InterpreterResult {
        match size {
            Size::Byte => {
                let (src, dst) = if mode == Direction::MemoryToMemory {
                    let src_addr = self.ariwpr(ry, size);
                    let dst_addr = self.ariwpr(rx, size);
                    (memory.get_byte(src_addr)?, memory.get_byte(dst_addr)?)
                } else {
                    (self.regs.d[ry as usize] as u8, self.regs.d[rx as usize] as u8)
                };

                let (res, v) = (src as i8).extended_add(dst as i8, self.regs.sr.x);
                let (_, c) = src.extended_add(dst, self.regs.sr.x);

                self.regs.sr.x = c;
                self.regs.sr.n = res < 0;
                if res != 0 {
                    self.regs.sr.z = false;
                }
                self.regs.sr.v = v;
                self.regs.sr.c = c;

                if mode == Direction::MemoryToMemory {
                    memory.set_byte(self.a(rx), res as u8)?;
                    Ok(EXEC::ADDX_MEM_BW)
                } else {
                    self.d_byte(rx, res as u8);
                    Ok(EXEC::ADDX_REG_BW)
                }
            },
            Size::Word => {
                let (src, dst) = if mode == Direction::MemoryToMemory {
                    let src_addr = self.ariwpr(ry, size);
                    let dst_addr = self.ariwpr(rx, size);
                    (memory.get_word(src_addr.even()?)?, memory.get_word(dst_addr.even()?)?)
                } else {
                    (self.regs.d[ry as usize] as u16, self.regs.d[rx as usize] as u16)
                };

                let (res, v) = (src as i16).extended_add(dst as i16, self.regs.sr.x);
                let (_, c) = src.extended_add(dst, self.regs.sr.x);

                self.regs.sr.x = c;
                self.regs.sr.n = res < 0;
                if res != 0 {
                    self.regs.sr.z = false;
                }
                self.regs.sr.v = v;
                self.regs.sr.c = c;

                if mode == Direction::MemoryToMemory {
                    memory.set_word(self.a(rx), res as u16)?;
                    Ok(EXEC::ADDX_MEM_BW)
                } else {
                    self.d_word(rx, res as u16);
                    Ok(EXEC::ADDX_REG_BW)
                }
            },
            Size::Long => {
                let (src, dst) = if mode == Direction::MemoryToMemory {
                    let src_addr = self.ariwpr(ry, size);
                    let dst_addr = self.ariwpr(rx, size);
                    (memory.get_long(src_addr.even()?)?, memory.get_long(dst_addr.even()?)?)
                } else {
                    (self.regs.d[ry as usize], self.regs.d[rx as usize])
                };

                let (res, v) = (src as i32).extended_add(dst as i32, self.regs.sr.x);
                let (_, c) = src.extended_add(dst, self.regs.sr.x);

                self.regs.sr.x = c;
                self.regs.sr.n = res < 0;
                if res != 0 {
                    self.regs.sr.z = false;
                }
                self.regs.sr.v = v;
                self.regs.sr.c = c;

                if mode == Direction::MemoryToMemory {
                    memory.set_long(self.a(rx), res as u32)?;
                    Ok(EXEC::ADDX_MEM_L)
                } else {
                    self.regs.d[rx as usize] = res as u32;
                    Ok(EXEC::ADDX_REG_L)
                }
            },
        }
    }

    pub(super) fn execute_and(&mut self, memory: &mut impl MemoryAccess, reg: u8, dir: Direction, size: Size, am: AddressingMode) -> InterpreterResult {
        let mut exec_time;

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte => {
                if dir == Direction::DstEa {
                    exec_time = EXEC::AND_MEM_BW;
                } else {
                    exec_time = EXEC::AND_REG_BW;
                }
                let src = self.regs.d[reg as usize] as u8;
                let dst = self.get_byte(memory, &mut ea, &mut exec_time)?;

                let res = src & dst;

                self.regs.sr.n = res & SIGN_BIT_8 != 0;
                self.regs.sr.z = res == 0;

                if dir == Direction::DstEa {
                    self.set_byte(memory, &mut ea, &mut exec_time, res)?;
                } else {
                    self.d_byte(reg, res);
                }
            },
            Size::Word => {
                if dir == Direction::DstEa {
                    exec_time = EXEC::AND_MEM_BW;
                } else {
                    exec_time = EXEC::AND_REG_BW;
                }
                let src = self.regs.d[reg as usize] as u16;
                let dst = self.get_word(memory, &mut ea, &mut exec_time)?;

                let res = src & dst;

                self.regs.sr.n = res & SIGN_BIT_16 != 0;
                self.regs.sr.z = res == 0;

                if dir == Direction::DstEa {
                    self.set_word(memory, &mut ea, &mut exec_time, res)?;
                } else {
                    self.d_word(reg, res);
                }
            },
            Size::Long => {
                if dir == Direction::DstEa {
                    exec_time = EXEC::AND_MEM_L;
                } else {
                    exec_time = if am.is_dard() || am.is_immediate() { EXEC::AND_REG_L_RDIMM } else { EXEC::AND_REG_L };
                }
                let src = self.regs.d[reg as usize];
                let dst = self.get_long(memory, &mut ea, &mut exec_time)?;

                let res = src & dst;

                self.regs.sr.n = res & SIGN_BIT_32 != 0;
                self.regs.sr.z = res == 0;

                if dir == Direction::DstEa {
                    self.set_long(memory, &mut ea, &mut exec_time, res)?;
                } else {
                    self.regs.d[reg as usize] = res;
                }
            },
        }

        self.regs.sr.v = false;
        self.regs.sr.c = false;

        Ok(exec_time)
    }

    pub(super) fn execute_andi(&mut self, memory: &mut impl MemoryAccess, size: Size, am: AddressingMode, imm: u32) -> InterpreterResult {
        let mut exec_time;

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte => {
                exec_time = if am.is_drd() { EXEC::ANDI_REG_BW } else { EXEC::ANDI_MEM_BW };
                let data = self.get_byte(memory, &mut ea, &mut exec_time)? & imm as u8;
                self.set_byte(memory, &mut ea, &mut exec_time, data)?;

                self.regs.sr.n = data & SIGN_BIT_8 != 0;
                self.regs.sr.z = data == 0;
            },
            Size::Word => {
                exec_time = if am.is_drd() { EXEC::ANDI_REG_BW } else { EXEC::ANDI_MEM_BW };
                let data = self.get_word(memory, &mut ea, &mut exec_time)? & imm as u16;
                self.set_word(memory, &mut ea, &mut exec_time, data)?;

                self.regs.sr.n = data & SIGN_BIT_16 != 0;
                self.regs.sr.z = data == 0;
            },
            Size::Long => {
                exec_time = if am.is_drd() { EXEC::ANDI_REG_L } else { EXEC::ANDI_MEM_L };
                let data = self.get_long(memory, &mut ea, &mut exec_time)? & imm;
                self.set_long(memory, &mut ea, &mut exec_time, data)?;

                self.regs.sr.n = data & SIGN_BIT_32 != 0;
                self.regs.sr.z = data == 0;
            },
        }

        self.regs.sr.v = false;
        self.regs.sr.c = false;

        Ok(exec_time)
    }

    pub(super) fn execute_andiccr(&mut self, imm: u16) -> InterpreterResult {
        self.regs.sr &= SR_UPPER_MASK | imm;

        Ok(EXEC::ANDICCR)
    }

    pub(super) fn execute_andisr(&mut self, imm: u16) -> InterpreterResult {
        self.check_supervisor()?;

        self.regs.sr &= imm;
        Ok(EXEC::ANDISR)
    }

    pub(super) fn execute_asm(&mut self, memory: &mut impl MemoryAccess, dir: Direction, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = EXEC::ASM;

        let mut ea = EffectiveAddress::new(am, Some(Size::Word));

        let mut data = self.get_word(memory, &mut ea, &mut exec_time)? as i16;
        let sign = data & SIGN_BIT_16 as i16;

        if dir == Direction::Left {
            data <<= 1;
            self.regs.sr.x = sign != 0;
            self.regs.sr.v = sign ^ data & SIGN_BIT_16 as i16 != 0;
            self.regs.sr.c = sign != 0;
        } else {
            let bit = data & 1;
            data >>= 1;
            self.regs.sr.x = bit != 0;
            self.regs.sr.v = false;
            self.regs.sr.c = bit != 0;
        }

        self.regs.sr.n = data < 0;
        self.regs.sr.z = data == 0;

        self.set_word(memory, &mut ea, &mut exec_time, data as u16)?;

        Ok(exec_time)
    }

    pub(super) fn execute_asr(&mut self, rot: u8, dir: Direction, size: Size, mode: u8, reg: u8) -> InterpreterResult {
        self.regs.sr.v = false;
        self.regs.sr.c = false;

        let shift_count = if mode == 1 {
            (self.regs.d[rot as usize] % 64) as u8
        } else {
            if rot == 0 { 8 } else { rot }
        };

        let (mut data, mask) = match size {
            Size::Byte => (self.regs.d[reg as usize] & 0x0000_00FF, SIGN_BIT_8 as u32),
            Size::Word => (self.regs.d[reg as usize] & 0x0000_FFFF, SIGN_BIT_16 as u32),
            Size::Long => (self.regs.d[reg as usize], SIGN_BIT_32),
        };

        if dir == Direction::Left {
            for _ in 0..shift_count {
                let sign = data & mask;
                data <<= 1;
                self.regs.sr.x = sign != 0;
                self.regs.sr.c = sign != 0;
                if sign ^ data & mask != 0 {
                    self.regs.sr.v = true;
                }
            }
        } else {
            let sign = data & mask;
            for _ in 0..shift_count {
                let bit = data & 1;
                data >>= 1;
                data |= sign;
                self.regs.sr.x = bit != 0;
                self.regs.sr.c = bit != 0;
            }
        }

        self.regs.sr.n = data & mask != 0;

        Ok(match size {
            Size::Byte => {
                self.d_byte(reg, data as u8);
                self.regs.sr.z = data & 0x0000_00FF == 0;
                EXEC::ASR_BW + EXEC::ASR_COUNT * shift_count as usize
            },
            Size::Word => {
                self.d_word(reg, data as u16);
                self.regs.sr.z = data & 0x0000_FFFF == 0;
                EXEC::ASR_BW + EXEC::ASR_COUNT * shift_count as usize
            },
            Size::Long => {
                self.regs.d[reg as usize] = data;
                self.regs.sr.z = data == 0;
                EXEC::ASR_L + EXEC::ASR_COUNT * shift_count as usize
            }
        })
    }

    pub(super) fn execute_bcc(&mut self, pc: u32, condition: u8, displacement: i16) -> InterpreterResult {
        if self.regs.sr.condition(condition) {
            self.regs.pc = pc + displacement as u32;
            Ok(EXEC::BCC_BRANCH)
        } else {
            Ok(if self.current_opcode as u8 == 0 {
                EXEC::BCC_NO_BRANCH_WORD
            } else {
                EXEC::BCC_NO_BRANCH_BYTE
            })
        }
    }

    pub(super) fn execute_bchg(&mut self, memory: &mut impl MemoryAccess, am: AddressingMode, mut count: u8) -> InterpreterResult {
        let mut exec_time = if bits(self.current_opcode, 8, 8) != 0 {
            count = self.regs.d[count as usize] as u8;
            if am.is_drd() { EXEC::BCHG_DYN_REG } else { EXEC::BCHG_DYN_MEM }
        } else {
            if am.is_drd() { EXEC::BCHG_STA_REG } else { EXEC::BCHG_STA_MEM }
        };

        if am.is_drd() {
            count %= 32;
            let reg = am.register().unwrap() as usize;
            self.regs.sr.z = self.regs.d[reg] & 1 << count == 0;
            self.regs.d[reg] ^= 1 << count;
        } else {
            let mut ea = EffectiveAddress::new(am, Some(Size::Byte)); // Memory is byte only.
            count %= 8;
            let mut data = self.get_byte(memory, &mut ea, &mut exec_time)?;
            self.regs.sr.z = data & 1 << count == 0;
            data ^= 1 << count;
            self.set_byte(memory, &mut ea, &mut exec_time, data)?;
        }

        Ok(exec_time)
    }

    pub(super) fn execute_bclr(&mut self, memory: &mut impl MemoryAccess, am: AddressingMode, mut count: u8) -> InterpreterResult {
        let mut exec_time = if bits(self.current_opcode, 8, 8) != 0 {
            count = self.regs.d[count as usize] as u8;
            if am.is_drd() { EXEC::BCLR_DYN_REG } else { EXEC::BCLR_DYN_MEM }
        } else {
            if am.is_drd() { EXEC::BCLR_STA_REG } else { EXEC::BCLR_STA_MEM }
        };

        if am.is_drd() {
            count %= 32;
            let reg = am.register().unwrap() as usize;
            self.regs.sr.z = self.regs.d[reg] & 1 << count == 0;
            self.regs.d[reg] &= !(1 << count);
        } else {
            let mut ea = EffectiveAddress::new(am, Some(Size::Byte)); // Memory is byte only.
            count %= 8;
            let mut data = self.get_byte(memory, &mut ea, &mut exec_time)?;
            self.regs.sr.z = data & 1 << count == 0;
            data &= !(1 << count);
            self.set_byte(memory, &mut ea, &mut exec_time, data)?;
        }

        Ok(exec_time)
    }

    pub(super) fn execute_bra(&mut self, pc: u32, disp: i16) -> InterpreterResult {
        self.regs.pc = pc + disp as u32;

        Ok(if self.current_opcode as u8 == 0 {
            EXEC::BRA_WORD
        } else {
            EXEC::BRA_BYTE
        })
    }

    pub(super) fn execute_bset(&mut self, memory: &mut impl MemoryAccess, am: AddressingMode, mut count: u8) -> InterpreterResult {
        let mut exec_time = if bits(self.current_opcode, 8, 8) != 0 {
            count = self.regs.d[count as usize] as u8;
            if am.is_drd() { EXEC::BSET_DYN_REG } else { EXEC::BSET_DYN_MEM }
        } else {
            if am.is_drd() { EXEC::BSET_STA_REG } else { EXEC::BSET_STA_MEM }
        };

        if am.is_drd() {
            count %= 32;
            let reg = am.register().unwrap() as usize;
            self.regs.sr.z = self.regs.d[reg] & 1 << count == 0;
            self.regs.d[reg] |= 1 << count;
        } else {
            let mut ea = EffectiveAddress::new(am, Some(Size::Byte)); // Memory is byte only.
            count %= 8;
            let mut data = self.get_byte(memory, &mut ea, &mut exec_time)?;
            self.regs.sr.z = data & 1 << count == 0;
            data |= 1 << count;
            self.set_byte(memory, &mut ea, &mut exec_time, data)?;
        }

        Ok(exec_time)
    }

    pub(super) fn execute_bsr(&mut self, memory: &mut impl MemoryAccess, pc: u32, disp: i16) -> InterpreterResult {
        self.push_long(memory, self.regs.pc)?;
        self.regs.pc = pc + disp as u32;

        Ok(if self.current_opcode as u8 == 0 {
            EXEC::BSR_WORD
        } else {
            EXEC::BSR_BYTE
        })
    }

    pub(super) fn execute_btst(&mut self, memory: &mut impl MemoryAccess, am: AddressingMode, mut count: u8) -> InterpreterResult {
        let mut exec_time = if bits(self.current_opcode, 8, 8) != 0 {
            count = self.regs.d[count as usize] as u8;
            if am.is_drd() { EXEC::BTST_DYN_REG } else { EXEC::BTST_DYN_MEM }
        } else {
            if am.is_drd() { EXEC::BTST_STA_REG } else { EXEC::BTST_STA_MEM }
        };

        if am.is_drd() {
            count %= 32;
            let reg = am.register().unwrap() as usize;
            self.regs.sr.z = self.regs.d[reg] & 1 << count == 0;
        } else {
            let mut ea = EffectiveAddress::new(am, Some(Size::Byte)); // Memory is byte only.
            count %= 8;
            let data = self.get_byte(memory, &mut ea, &mut exec_time)?;
            self.regs.sr.z = data & 1 << count == 0;
        }

        Ok(exec_time)
    }

    /// If a CHK exception occurs, this method returns the effective address calculation time, and the
    /// process_exception method returns the exception processing time.
    pub(super) fn execute_chk(&mut self, memory: &mut impl MemoryAccess, reg: u8, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = 0;

        let mut ea = EffectiveAddress::new(am, Some(Size::Word));

        let src = self.get_word(memory, &mut ea, &mut exec_time)? as i16;
        let data = self.regs.d[reg as usize] as i16;

        if data < 0 || data > src {
            Err(Vector::ChkInstruction as u8)
        } else {
            Ok(EXEC::CHK_NO_TRAP + exec_time)
        }
    }

    pub(super) fn execute_clr(&mut self, memory: &mut impl MemoryAccess, size: Size, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = single_operands_time(size.is_long(), am.is_drd(), EXEC::CLR_REG_BW, EXEC::CLR_REG_L, EXEC::CLR_MEM_BW, EXEC::CLR_MEM_L);

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte => self.set_byte(memory, &mut ea, &mut exec_time, 0)?,
            Size::Word => self.set_word(memory, &mut ea, &mut exec_time, 0)?,
            Size::Long => self.set_long(memory, &mut ea, &mut exec_time, 0)?,
        }

        self.regs.sr.n = false;
        self.regs.sr.z = true;
        self.regs.sr.v = false;
        self.regs.sr.c = false;

        Ok(exec_time)
    }

    pub(super) fn execute_cmp(&mut self, memory: &mut impl MemoryAccess, reg: u8, size: Size, am: AddressingMode) -> InterpreterResult {
        let mut exec_time;

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte => {
                exec_time = EXEC::CMP_BW;
                let src = self.get_byte(memory, &mut ea, &mut exec_time)?;
                let dst = self.regs.d[reg as usize] as u8;

                let (res, v) = (dst as i8).overflowing_sub(src as i8);
                let (_, c) = dst.overflowing_sub(src);

                self.regs.sr.n = res < 0;
                self.regs.sr.z = res == 0;
                self.regs.sr.v = v;
                self.regs.sr.c = c;
            },
            Size::Word => {
                exec_time = EXEC::CMP_BW;
                let src = self.get_word(memory, &mut ea, &mut exec_time)?;
                let dst = self.regs.d[reg as usize] as u16;

                let (res, v) = (dst as i16).overflowing_sub(src as i16);
                let (_, c) = dst.overflowing_sub(src);

                self.regs.sr.n = res < 0;
                self.regs.sr.z = res == 0;
                self.regs.sr.v = v;
                self.regs.sr.c = c;
            },
            Size::Long => {
                exec_time = EXEC::CMP_L;
                let src = self.get_long(memory, &mut ea, &mut exec_time)?;
                let dst = self.regs.d[reg as usize];

                let (res, v) = (dst as i32).overflowing_sub(src as i32);
                let (_, c) = dst.overflowing_sub(src);

                self.regs.sr.n = res < 0;
                self.regs.sr.z = res == 0;
                self.regs.sr.v = v;
                self.regs.sr.c = c;
            },
        }

        Ok(exec_time)
    }

    pub(super) fn execute_cmpa(&mut self, memory: &mut impl MemoryAccess, reg: u8, size: Size, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = EXEC::CMPA;

        let mut ea = EffectiveAddress::new(am, Some(size));

        let src = if size.is_word() {
            self.get_word(memory, &mut ea, &mut exec_time)? as i16 as u32
        } else {
            self.get_long(memory, &mut ea, &mut exec_time)?
        };

        let (res, v) = (self.a(reg) as i32).overflowing_sub(src as i32);
        let (_, c) = self.a(reg).overflowing_sub(src);

        self.regs.sr.n = res < 0;
        self.regs.sr.z = res == 0;
        self.regs.sr.v = v;
        self.regs.sr.c = c;

        Ok(exec_time)
    }

    pub(super) fn execute_cmpi(&mut self, memory: &mut impl MemoryAccess, size: Size, am: AddressingMode, imm: u32) -> InterpreterResult {
        let mut exec_time;

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte => {
                exec_time = if am.is_drd() { EXEC::CMPI_REG_BW } else { EXEC::CMPI_MEM_BW };
                let data = self.get_byte(memory, &mut ea, &mut exec_time)?;
                let (res, v) = (data as i8).overflowing_sub(imm as i8);
                let (_, c) = data.overflowing_sub(imm as u8);

                self.regs.sr.n = res < 0;
                self.regs.sr.z = res == 0;
                self.regs.sr.v = v;
                self.regs.sr.c = c;
            },
            Size::Word => {
                exec_time = if am.is_drd() { EXEC::CMPI_REG_BW } else { EXEC::CMPI_MEM_BW };
                let data = self.get_word(memory, &mut ea, &mut exec_time)?;
                let (res, v) = (data as i16).overflowing_sub(imm as i16);
                let (_, c) = data.overflowing_sub(imm as u16);

                self.regs.sr.n = res < 0;
                self.regs.sr.z = res == 0;
                self.regs.sr.v = v;
                self.regs.sr.c = c;
            },
            Size::Long => {
                exec_time = if am.is_drd() { EXEC::CMPI_REG_L } else { EXEC::CMPI_MEM_L };
                let data = self.get_long(memory, &mut ea, &mut exec_time)?;
                let (res, v) = (data as i32).overflowing_sub(imm as i32);
                let (_, c) = data.overflowing_sub(imm);

                self.regs.sr.n = res < 0;
                self.regs.sr.z = res == 0;
                self.regs.sr.v = v;
                self.regs.sr.c = c;
            },
        }

        Ok(exec_time)
    }

    pub(super) fn execute_cmpm(&mut self, memory: &mut impl MemoryAccess, ax: u8, size: Size, ay: u8) -> InterpreterResult {
        let addry = self.ariwpo(ay, size);
        let addrx = self.ariwpo(ax, size);

        match size {
            Size::Byte => {
                let src = memory.get_byte(addry)?;
                let dst = memory.get_byte(addrx)?;

                let (res, v) = (dst as i8).overflowing_sub(src as i8);
                let (_, c) = dst.overflowing_sub(src);

                self.regs.sr.n = res < 0;
                self.regs.sr.z = res == 0;
                self.regs.sr.v = v;
                self.regs.sr.c = c;

                Ok(EXEC::CMPM_BW)
            },
            Size::Word => {
                let src = memory.get_word(addry.even()?)?;
                let dst = memory.get_word(addrx.even()?)?;

                let (res, v) = (dst as i16).overflowing_sub(src as i16);
                let (_, c) = dst.overflowing_sub(src);

                self.regs.sr.n = res < 0;
                self.regs.sr.z = res == 0;
                self.regs.sr.v = v;
                self.regs.sr.c = c;

                Ok(EXEC::CMPM_BW)
            },
            Size::Long => {
                let src = memory.get_long(addry.even()?)?;
                let dst = memory.get_long(addrx.even()?)?;

                let (res, v) = (dst as i32).overflowing_sub(src as i32);
                let (_, c) = dst.overflowing_sub(src);

                self.regs.sr.n = res < 0;
                self.regs.sr.z = res == 0;
                self.regs.sr.v = v;
                self.regs.sr.c = c;

                Ok(EXEC::CMPM_L)
            },
        }
    }

    pub(super) fn execute_dbcc(&mut self, pc: u32, cc: u8, reg: u8, disp: i16) -> InterpreterResult {
        if !self.regs.sr.condition(cc) {
            let counter = self.regs.d[reg as usize] as i16 - 1;
            self.d_word(reg, counter as u16);

            if counter != -1 {
                self.regs.pc = pc + disp as u32;
                Ok(EXEC::DBCC_FALSE_BRANCH)
            } else {
                Ok(EXEC::DBCC_FALSE_NO_BRANCH)
            }
        } else {
            Ok(EXEC::DBCC_TRUE)
        }
    }

    /// If a zero divide exception occurs, this method returns the effective address calculation time, and the
    /// process_exception method returns the exception processing time.
    pub(super) fn execute_divs(&mut self, memory: &mut impl MemoryAccess, reg: u8, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = EXEC::DIVS;

        let mut ea = EffectiveAddress::new(am, Some(Size::Word));

        let src = self.get_word(memory, &mut ea, &mut exec_time)? as i16 as i32;
        let dst = self.regs.d[reg as usize] as i32;

        if src == 0 {
            Err(Vector::ZeroDivide as u8)
        } else {
            let quot = dst / src;
            let rem = dst % src;
            self.regs.d[reg as usize] = (rem as u16 as u32) << 16 | (quot as u16 as u32);

            self.regs.sr.n = quot < 0;
            self.regs.sr.z = quot == 0;
            self.regs.sr.v = quot < i16::MIN as i32 || quot > i16::MAX as i32;
            self.regs.sr.c = false;

            Ok(exec_time)
        }
    }

    /// If a zero divide exception occurs, this method returns the effective address calculation time, and the
    /// process_exception method returns the exception processing time.
    pub(super) fn execute_divu(&mut self, memory: &mut impl MemoryAccess, reg: u8, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = EXEC::DIVU;

        let mut ea = EffectiveAddress::new(am, Some(Size::Word));

        let src = self.get_word(memory, &mut ea, &mut exec_time)? as u32;
        let dst = self.regs.d[reg as usize];

        if src == 0 {
            Err(Vector::ZeroDivide as u8)
        } else {
            let quot = dst / src;
            let rem = dst % src;
            self.regs.d[reg as usize] = (rem as u16 as u32) << 16 | (quot as u16 as u32);

            self.regs.sr.n = quot & 0x0000_8000 != 0;
            self.regs.sr.z = quot == 0;
            self.regs.sr.v = (quot as i32) < i16::MIN as i32 || quot > i16::MAX as u32;
            self.regs.sr.c = false;

            Ok(exec_time)
        }
    }

    pub(super) fn execute_eor(&mut self, memory: &mut impl MemoryAccess, reg: u8, size: Size, am: AddressingMode) -> InterpreterResult {
        let mut exec_time;

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte => {
                exec_time = if am.is_drd() { EXEC::EOR_REG_BW } else { EXEC::EOR_MEM_BW };
                let src = self.regs.d[reg as usize] as u8;
                let dst = self.get_byte(memory, &mut ea, &mut exec_time)?;

                let res = src ^ dst;

                self.regs.sr.n = res & SIGN_BIT_8 != 0;
                self.regs.sr.z = res == 0;

                self.set_byte(memory, &mut ea, &mut exec_time, res)?;
            },
            Size::Word => {
                exec_time = if am.is_drd() { EXEC::EOR_REG_BW } else { EXEC::EOR_MEM_BW };
                let src = self.regs.d[reg as usize] as u16;
                let dst = self.get_word(memory, &mut ea, &mut exec_time)?;

                let res = src ^ dst;

                self.regs.sr.n = res & SIGN_BIT_16 != 0;
                self.regs.sr.z = res == 0;

                self.set_word(memory, &mut ea, &mut exec_time, res)?;
            },
            Size::Long => {
                exec_time = if am.is_drd() { EXEC::EOR_REG_L } else { EXEC::EOR_MEM_L };
                let src = self.regs.d[reg as usize];
                let dst = self.get_long(memory, &mut ea, &mut exec_time)?;

                let res = src ^ dst;

                self.regs.sr.n = res & SIGN_BIT_32 != 0;
                self.regs.sr.z = res == 0;

                self.set_long(memory, &mut ea, &mut exec_time, res)?;
            },
        }

        self.regs.sr.v = false;
        self.regs.sr.c = false;

        Ok(exec_time)
    }

    pub(super) fn execute_eori(&mut self, memory: &mut impl MemoryAccess, size: Size, am: AddressingMode, imm: u32) -> InterpreterResult {
        let mut exec_time;

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte => {
                exec_time = if am.is_drd() { EXEC::EORI_REG_BW } else { EXEC::EORI_MEM_BW };
                let data = self.get_byte(memory, &mut ea, &mut exec_time)? ^ imm as u8;
                self.set_byte(memory, &mut ea, &mut exec_time, data)?;

                self.regs.sr.n = data & SIGN_BIT_8 != 0;
                self.regs.sr.z = data == 0;
            },
            Size::Word => {
                exec_time = if am.is_drd() { EXEC::EORI_REG_BW } else { EXEC::EORI_MEM_BW };
                let data = self.get_word(memory, &mut ea, &mut exec_time)? ^ imm as u16;
                self.set_word(memory, &mut ea, &mut exec_time, data)?;

                self.regs.sr.n = data & SIGN_BIT_16 != 0;
                self.regs.sr.z = data == 0;
            },
            Size::Long => {
                exec_time = if am.is_drd() { EXEC::EORI_REG_L } else { EXEC::EORI_MEM_L };
                let data = self.get_long(memory, &mut ea, &mut exec_time)? ^ imm;
                self.set_long(memory, &mut ea, &mut exec_time, data)?;

                self.regs.sr.n = data & SIGN_BIT_32 != 0;
                self.regs.sr.z = data == 0;
            },
        }

        self.regs.sr.v = false;
        self.regs.sr.c = false;

        Ok(exec_time)
    }

    pub(super) fn execute_eoriccr(&mut self, imm: u16) -> InterpreterResult {
        self.regs.sr ^= imm;

        Ok(EXEC::EORICCR)
    }

    pub(super) fn execute_eorisr(&mut self, imm: u16) -> InterpreterResult {
        self.check_supervisor()?;

        self.regs.sr ^= imm;
        Ok(EXEC::EORISR)
    }

    pub(super) fn execute_exg(&mut self, rx: u8, mode: Direction, ry: u8) -> InterpreterResult {
        if mode == Direction::ExchangeData {
            self.regs.d.swap(rx as usize, ry as usize);
        } else if mode == Direction::ExchangeAddress {
            // TODO: change to std::mem::swap when new borrow checker is available
            let y = self.a(ry);
            *self.a_mut(ry) = self.a(rx);
            *self.a_mut(rx) = y;
        } else {
            let y = self.a(ry);
            *self.a_mut(ry) = self.regs.d[rx as usize];
            self.regs.d[rx as usize] = y;
        }

        Ok(EXEC::EXG)
    }

    pub(super) fn execute_ext(&mut self, mode: u8, reg: u8) -> InterpreterResult {
        if mode == 0b010 {
            let d = self.regs.d[reg as usize] as i8 as u16;
            self.d_word(reg, d);
        } else {
            self.regs.d[reg as usize] = self.regs.d[reg as usize] as i16 as u32;
        }

        self.regs.sr.n = self.regs.d[reg as usize] & SIGN_BIT_32 != 0;
        self.regs.sr.z = self.regs.d[reg as usize] == 0;
        self.regs.sr.v = false;
        self.regs.sr.c = false;

        Ok(EXEC::EXT)
    }

    pub(super) fn execute_illegal(&self) -> InterpreterResult {
        Err(Vector::IllegalInstruction as u8)
    }

    pub(super) fn execute_jmp(&mut self, am: AddressingMode) -> InterpreterResult {
        let mut ea = EffectiveAddress::new(am, None);

        let mut exec_time = 0;
        self.regs.pc = self.get_effective_address(&mut ea, &mut exec_time);

        Ok(match am {
            AddressingMode::Ari(_) => EXEC::JMP_ARI,
            AddressingMode::Ariwd(..) => EXEC::JMP_ARIWD,
            AddressingMode::Ariwi8(..) => EXEC::JMP_ARIWI8,
            AddressingMode::AbsShort(_) => EXEC::JMP_ABSSHORT,
            AddressingMode::AbsLong(_) => EXEC::JMP_ABSLONG,
            AddressingMode::Pciwd(..) => EXEC::JMP_PCIWD,
            AddressingMode::Pciwi8(..) => EXEC::JMP_PCIWI8,
            _ => panic!("Wrong addressing mode in JMP."),
        })
    }

    pub(super) fn execute_jsr(&mut self, memory: &mut impl MemoryAccess, am: AddressingMode) -> InterpreterResult {
        let mut ea = EffectiveAddress::new(am, None);

        let mut exec_time = 0;
        self.push_long(memory, self.regs.pc)?;
        self.regs.pc = self.get_effective_address(&mut ea, &mut exec_time);

        Ok(match am {
            AddressingMode::Ari(_) => EXEC::JSR_ARI,
            AddressingMode::Ariwd(..) => EXEC::JSR_ARIWD,
            AddressingMode::Ariwi8(..) => EXEC::JSR_ARIWI8,
            AddressingMode::AbsShort(_) => EXEC::JSR_ABSSHORT,
            AddressingMode::AbsLong(_) => EXEC::JSR_ABSLONG,
            AddressingMode::Pciwd(..) => EXEC::JSR_PCIWD,
            AddressingMode::Pciwi8(..) => EXEC::JSR_PCIWI8,
            _ => panic!("Wrong addressing mode in JSR."),
        })
    }

    pub(super) fn execute_lea(&mut self, reg: u8, am: AddressingMode) -> InterpreterResult {
        let mut ea = EffectiveAddress::new(am, None);

        let mut exec_time = 0;
        *self.a_mut(reg) = self.get_effective_address(&mut ea, &mut exec_time);

        Ok(match am {
            AddressingMode::Ari(_) => EXEC::LEA_ARI,
            AddressingMode::Ariwd(..) => EXEC::LEA_ARIWD,
            AddressingMode::Ariwi8(..) => EXEC::LEA_ARIWI8,
            AddressingMode::AbsShort(_) => EXEC::LEA_ABSSHORT,
            AddressingMode::AbsLong(_) => EXEC::LEA_ABSLONG,
            AddressingMode::Pciwd(..) => EXEC::LEA_PCIWD,
            AddressingMode::Pciwi8(..) => EXEC::LEA_PCIWI8,
            _ => panic!("Wrong addressing mode in LEA."),
        })
    }

    pub(super) fn execute_link(&mut self, memory: &mut impl MemoryAccess, reg: u8, disp: i16) -> InterpreterResult {
        self.push_long(memory, self.a(reg))?;
        *self.a_mut(reg) = self.sp();
        *self.sp_mut() += disp as u32;

        Ok(EXEC::LINK)
    }

    pub(super) fn execute_lsm(&mut self, memory: &mut impl MemoryAccess, dir: Direction, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = EXEC::LSM;

        let mut ea = EffectiveAddress::new(am, Some(Size::Word));

        let mut data = self.get_word(memory, &mut ea, &mut exec_time)?;

        if dir == Direction::Left {
            let sign = data & SIGN_BIT_16;
            data <<= 1;
            self.regs.sr.x = sign != 0;
            self.regs.sr.c = sign != 0;
        } else {
            let bit = data & 1;
            data >>= 1;
            self.regs.sr.x = bit != 0;
            self.regs.sr.c = bit != 0;
        }

        self.regs.sr.n = data & SIGN_BIT_16 != 0;
        self.regs.sr.z = data == 0;
        self.regs.sr.v = false;

        self.set_word(memory, &mut ea, &mut exec_time, data)?;

        Ok(exec_time)
    }

    pub(super) fn execute_lsr(&mut self, rot: u8, dir: Direction, size: Size, mode: u8, reg: u8) -> InterpreterResult {
        self.regs.sr.v = false;
        self.regs.sr.c = false;

        let shift_count = if mode == 1 {
            (self.regs.d[rot as usize] % 64) as u8
        } else {
            if rot == 0 { 8 } else { rot }
        };

        let (mut data, mask) = match size {
            Size::Byte => (self.regs.d[reg as usize] & 0x0000_00FF, SIGN_BIT_8 as u32),
            Size::Word => (self.regs.d[reg as usize] & 0x0000_FFFF, SIGN_BIT_16 as u32),
            Size::Long => (self.regs.d[reg as usize], SIGN_BIT_32),
        };

        if dir == Direction::Left {
            for _ in 0..shift_count {
                let sign = data & mask;
                data <<= 1;
                self.regs.sr.x = sign != 0;
                self.regs.sr.c = sign != 0;
            }
        } else {
            for _ in 0..shift_count {
                let bit = data & 1;
                data >>= 1;
                self.regs.sr.x = bit != 0;
                self.regs.sr.c = bit != 0;
            }
        }

        self.regs.sr.n = data & mask != 0;

        Ok(match size {
            Size::Byte => {
                self.d_byte(reg, data as u8);
                self.regs.sr.z = data & 0x0000_00FF == 0;
                EXEC::LSR_BW + EXEC::LSR_COUNT * shift_count as usize
            },
            Size::Word => {
                self.d_word(reg, data as u16);
                self.regs.sr.z = data & 0x0000_FFFF == 0;
                EXEC::LSR_BW + EXEC::LSR_COUNT * shift_count as usize
            },
            Size::Long => {
                self.regs.d[reg as usize] = data;
                self.regs.sr.z = data == 0;
                EXEC::LSR_L + EXEC::LSR_COUNT * shift_count as usize
            },
        })
    }

    pub(super) fn execute_move(&mut self, memory: &mut impl MemoryAccess, size: Size, amdst: AddressingMode, amsrc: AddressingMode) -> InterpreterResult {
        let mut exec_time = if amdst.is_ariwpr() { EXEC::MOVE_DST_ARIWPR } else { EXEC::MOVE_OTHER };

        let mut src = EffectiveAddress::new(amsrc, Some(size));
        let mut dst = EffectiveAddress::new(amdst, Some(size));

        match size {
            Size::Byte => {
                let d = self.get_byte(memory, &mut src, &mut exec_time)?;
                self.set_byte(memory, &mut dst, &mut exec_time, d)?;
                self.regs.sr.n = d & SIGN_BIT_8 != 0;
                self.regs.sr.z = d == 0;
            },
            Size::Word => {
                let d = self.get_word(memory, &mut src, &mut exec_time)?;
                self.set_word(memory, &mut dst, &mut exec_time, d)?;
                self.regs.sr.n = d & SIGN_BIT_16 != 0;
                self.regs.sr.z = d == 0;
            },
            Size::Long => {
                let d = self.get_long(memory, &mut src, &mut exec_time)?;
                self.set_long(memory, &mut dst, &mut exec_time, d)?;
                self.regs.sr.n = d & SIGN_BIT_32 != 0;
                self.regs.sr.z = d == 0;
            },
        }

        self.regs.sr.v = false;
        self.regs.sr.c = false;

        Ok(exec_time)
    }

    pub(super) fn execute_movea(&mut self, memory: &mut impl MemoryAccess, size: Size, reg: u8, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = EXEC::MOVEA;

        let mut ea = EffectiveAddress::new(am, Some(size));

        *self.a_mut(reg) = if size.is_word() {
            self.get_word(memory, &mut ea, &mut exec_time)? as i16 as u32
        } else {
            self.get_long(memory, &mut ea, &mut exec_time)?
        };

        Ok(exec_time)
    }

    pub(super) fn execute_moveccr(&mut self, memory: &mut impl MemoryAccess, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = EXEC::MOVECCR;

        let mut ea = EffectiveAddress::new(am, Some(Size::Word));

        let ccr = self.get_word(memory, &mut ea, &mut exec_time)?;
        self.regs.sr.set_ccr(ccr);

        Ok(exec_time)
    }

    pub(super) fn execute_movefsr(&mut self, memory: &mut impl MemoryAccess, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = if am.is_drd() { EXEC::MOVEFSR_REG } else { EXEC::MOVEFSR_MEM };

        let mut ea = EffectiveAddress::new(am, Some(Size::Word));

        self.set_word(memory, &mut ea, &mut exec_time, self.regs.sr.into())?;

        Ok(exec_time)
    }

    pub(super) fn execute_movesr(&mut self, memory: &mut impl MemoryAccess, am: AddressingMode) -> InterpreterResult {
        self.check_supervisor()?;

        let mut ea = EffectiveAddress::new(am, Some(Size::Word));
        let mut exec_time = EXEC::MOVESR;

        let sr = self.get_word(memory, &mut ea, &mut exec_time)?;
        self.regs.sr = sr.into();
        Ok(exec_time)
    }

    pub(super) fn execute_moveusp(&mut self, dir: Direction, reg: u8) -> InterpreterResult {
        self.check_supervisor()?;

        if dir == Direction::UspToRegister {
            *self.a_mut(reg) = self.regs.usp;
        } else {
            self.regs.usp = self.a(reg);
        }
        Ok(EXEC::MOVEUSP)
    }

    pub(super) fn execute_movem(&mut self, memory: &mut impl MemoryAccess, dir: Direction, size: Size, am: AddressingMode, mut list: u16) -> InterpreterResult {
        let count = list.count_ones() as usize;
        let mut exec_time = 0;

        let mut ea = EffectiveAddress::new(am, Some(size));

        let gap = size as u32;
        let eareg = ea.mode.register().unwrap_or(u8::MAX);

        if ea.mode.is_ariwpr() {
            let mut addr = self.a(eareg);

            for reg in (0..8).rev() {
                if list & 1 != 0 {
                    addr -= gap;
                    if size.is_word() { memory.set_word(addr.even()?, self.a(reg) as u16)?; }
                        else { memory.set_long(addr.even()?, self.a(reg))?; }
                }

                list >>= 1;
            }

            for reg in (0..8).rev() {
                if list & 1 != 0 {
                    addr -= gap;
                    if size.is_word() { memory.set_word(addr.even()?, self.regs.d[reg] as u16)?; }
                        else { memory.set_long(addr.even()?, self.regs.d[reg])?; }
                }

                list >>= 1;
            }

            *self.a_mut(eareg) = addr;
        } else {
            let mut addr = if ea.mode.is_ariwpo() {
                self.a(eareg)
            } else {
                self.get_effective_address(&mut ea, &mut exec_time)
            };

            for reg in 0..8 {
                if list & 1 != 0 {
                    if dir == Direction::MemoryToRegister {
                        let value = if size.is_word() { memory.get_word(addr.even()?)? as i16 as u32 }
                            else { memory.get_long(addr.even()?)? };
                        self.regs.d[reg] = value;
                    } else {
                        if size.is_word() { memory.set_word(addr.even()?, self.regs.d[reg] as u16)?; }
                            else { memory.set_long(addr.even()?, self.regs.d[reg])?; }
                    }

                    addr += gap;
                }

                list >>= 1;
            }

            for reg in 0..8 {
                if list & 1 != 0 {
                    if dir == Direction::MemoryToRegister {
                        let value = if size.is_word() { memory.get_word(addr.even()?)? as i16 as u32 }
                            else { memory.get_long(addr.even()?)? };
                        *self.a_mut(reg) = value;
                    } else {
                        if size.is_word() { memory.set_word(addr.even()?, self.a(reg as u8) as u16)?; }
                            else { memory.set_long(addr.even()?, self.a(reg as u8))?; }
                    }

                    addr += gap;
                }

                list >>= 1;
            }

            if ea.mode.is_ariwpo() {
                *self.a_mut(eareg) = addr;
            }
        }

        exec_time = match am {
            AddressingMode::Ari(_) => EXEC::MOVEM_ARI,
            AddressingMode::Ariwpo(_) => EXEC::MOVEM_ARIWPO,
            AddressingMode::Ariwpr(_) => EXEC::MOVEM_ARIWPR,
            AddressingMode::Ariwd(..) => EXEC::MOVEM_ARIWD,
            AddressingMode::Ariwi8(..) => EXEC::MOVEM_ARIWI8,
            AddressingMode::AbsShort(_) => EXEC::MOVEM_ABSSHORT,
            AddressingMode::AbsLong(_) => EXEC::MOVEM_ABSLONG,
            AddressingMode::Pciwd(..) => EXEC::MOVEM_PCIWD,
            AddressingMode::Pciwi8(..) => EXEC::MOVEM_PCIWI8,
            _ => panic!("Wrong addressing mode for MOVEM."),
        };
        if dir == Direction::MemoryToRegister {
            exec_time += EXEC::MOVEM_MTR;
        }
        Ok(exec_time + count * if size.is_long() { EXEC::MOVEM_LONG } else { EXEC::MOVEM_WORD })
    }

    pub(super) fn execute_movep(&mut self, memory: &mut impl MemoryAccess, data: u8, dir: Direction, size: Size, addr: u8, disp: i16) -> InterpreterResult {
        let mut shift = if size.is_word() { 8 } else { 24 };
        let mut addr = self.a(addr) + disp as u32;

        if dir == Direction::RegisterToMemory {
            while shift >= 0 {
                let d = (self.regs.d[data as usize] >> shift) as u8;
                memory.set_byte(addr, d)?;
                shift -= 8;
                addr += 2;
            }

            Ok(if size.is_long() {
                EXEC::MOVEP_RTM_LONG
            } else {
                EXEC::MOVEP_RTM_WORD
            })
        } else {
            if size.is_word() { self.regs.d[data as usize] &= 0xFFFF_0000 } else { self.regs.d[data as usize] = 0 }

            while shift >= 0 {
                let d = memory.get_byte(addr)? as u32;
                self.regs.d[data as usize] |= d << shift;
                shift -= 8;
                addr += 2;
            }

            Ok(if size.is_long() {
                EXEC::MOVEP_MTR_LONG
            } else {
                EXEC::MOVEP_MTR_WORD
            })
        }
    }

    pub(super) fn execute_moveq(&mut self, reg: u8, data: i8) -> InterpreterResult {
        self.regs.d[reg as usize] = data as u32;

        self.regs.sr.n = data <  0;
        self.regs.sr.z = data == 0;
        self.regs.sr.v = false;
        self.regs.sr.c = false;

        Ok(EXEC::MOVEQ)
    }

    pub(super) fn execute_muls(&mut self, memory: &mut impl MemoryAccess, reg: u8, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = EXEC::MULS;

        let mut ea = EffectiveAddress::new(am, Some(Size::Word));

        let src = self.get_word(memory, &mut ea, &mut exec_time)? as i16 as i32;
        let dst = self.regs.d[reg as usize] as i16 as i32;

        let res = src * dst;
        self.regs.d[reg as usize] = res as u32;

        self.regs.sr.n = res < 0;
        self.regs.sr.z = res == 0;
        self.regs.sr.v = false;
        self.regs.sr.c = false;

        Ok(exec_time)
    }

    pub(super) fn execute_mulu(&mut self, memory: &mut impl MemoryAccess, reg: u8, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = EXEC::MULU;

        let mut ea = EffectiveAddress::new(am, Some(Size::Word));

        let src = self.get_word(memory, &mut ea, &mut exec_time)? as u32;
        let dst = self.regs.d[reg as usize] as u16 as u32;

        let res = src * dst;
        self.regs.d[reg as usize] = res;

        self.regs.sr.n = res & SIGN_BIT_32 != 0;
        self.regs.sr.z = res == 0;
        self.regs.sr.v = false;
        self.regs.sr.c = false;

        Ok(exec_time)
    }

    pub(super) fn execute_nbcd(&mut self, memory: &mut impl MemoryAccess, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = if am.is_drd() { EXEC::NBCD_REG } else { EXEC::NBCD_MEM };

        let mut ea = EffectiveAddress::new(am, Some(Size::Byte));

        let data = self.get_byte(memory, &mut ea, &mut exec_time)?;

        let low = 0 - (data as i8 & 0x0F) - self.regs.sr.x as i8;
        let high = 0 - (data as i8 >> 4 & 0x0F) - (low < 0) as i8;
        let res = (if high < 0 { 10 + high } else { high } as u8) << 4 |
                      if low < 0 { 10 + low } else { low } as u8;

        self.set_byte(memory, &mut ea, &mut exec_time, res)?;

        if res != 0 { self.regs.sr.z = false; }
        self.regs.sr.c = res != 0;
        self.regs.sr.x = self.regs.sr.c;

        Ok(exec_time)
    }

    pub(super) fn execute_neg(&mut self, memory: &mut impl MemoryAccess, size: Size, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = single_operands_time(size.is_long(), am.is_drd(), EXEC::NEG_REG_BW, EXEC::NEG_REG_L, EXEC::NEG_MEM_BW, EXEC::NEG_MEM_L);

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte => {
                let data = -(self.get_byte(memory, &mut ea, &mut exec_time)? as i8);
                self.set_byte(memory, &mut ea, &mut exec_time, data as u8)?;

                self.regs.sr.n = data < 0;
                self.regs.sr.z = data == 0;
                self.regs.sr.v = data == i8::MIN;
                self.regs.sr.c = data != 0;
                self.regs.sr.x = self.regs.sr.c;
            },
            Size::Word => {
                let data = -(self.get_word(memory, &mut ea, &mut exec_time)? as i16);
                self.set_word(memory, &mut ea, &mut exec_time, data as u16)?;

                self.regs.sr.n = data < 0;
                self.regs.sr.z = data == 0;
                self.regs.sr.v = data == i16::MIN;
                self.regs.sr.c = data != 0;
                self.regs.sr.x = self.regs.sr.c;
            },
            Size::Long => {
                let data = -(self.get_long(memory, &mut ea, &mut exec_time)? as i32);
                self.set_long(memory, &mut ea, &mut exec_time, data as u32)?;

                self.regs.sr.n = data < 0;
                self.regs.sr.z = data == 0;
                self.regs.sr.v = data == i32::MIN;
                self.regs.sr.c = data != 0;
                self.regs.sr.x = self.regs.sr.c;
            },
        }

        Ok(exec_time)
    }

    pub(super) fn execute_negx(&mut self, memory: &mut impl MemoryAccess, size: Size, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = single_operands_time(size.is_long(), am.is_drd(), EXEC::NEGX_REG_BW, EXEC::NEGX_REG_L, EXEC::NEGX_MEM_BW, EXEC::NEGX_MEM_L);

        let mut ea = EffectiveAddress::new(am, Some(size));

        // using overflowing_sub indicates an overflow when negating -128 with the X flag set.
        // 0 - -128 stays -128, then -128 - 1 gives 127, which is an overflow.
        // However I don't know if the hardware has intermediate overflow.
        // The other way is 0 - -128 gives 128, then 128 - 1 gives 127 which generates no overflow.
        // TODO: test what the hardware actually does.
        match size {
            Size::Byte => {
                let data = self.get_byte(memory, &mut ea, &mut exec_time)? as i8;
                let res = 0 - data - self.regs.sr.x as i8;
                let vres = 0 - data as i16 - self.regs.sr.x as i16;
                let (_, c) = 0u8.extended_sub(data as u8, self.regs.sr.x);
                self.set_byte(memory, &mut ea, &mut exec_time, res as u8)?;

                self.regs.sr.x = c;
                self.regs.sr.n = res < 0;
                if res != 0 { self.regs.sr.z = false };
                self.regs.sr.v = vres < i8::MIN as i16 || vres > i8::MAX as i16;
                self.regs.sr.c = c;
            },
            Size::Word => {
                let data = self.get_word(memory, &mut ea, &mut exec_time)? as i16;
                let res = 0 - data - self.regs.sr.x as i16;
                let vres = 0 - data as i32 - self.regs.sr.x as i32;
                let (_, c) = 0u16.extended_sub(data as u16, self.regs.sr.x);
                self.set_word(memory, &mut ea, &mut exec_time, res as u16)?;

                self.regs.sr.x = c;
                self.regs.sr.n = res < 0;
                if res != 0 { self.regs.sr.z = false };
                self.regs.sr.v = vres < i16::MIN as i32 || vres > i16::MAX as i32;
                self.regs.sr.c = c;
            },
            Size::Long => {
                let data = self.get_long(memory, &mut ea, &mut exec_time)? as i32;
                let res = 0 - data - self.regs.sr.x as i32;
                let vres = 0 - data as i64 - self.regs.sr.x as i64;
                let (_, c) = 0u32.extended_sub(data as u32, self.regs.sr.x);
                self.set_long(memory, &mut ea, &mut exec_time, res as u32)?;

                self.regs.sr.x = c;
                self.regs.sr.n = res < 0;
                if res != 0 { self.regs.sr.z = false };
                self.regs.sr.v = vres < i32::MIN as i64 || vres > i32::MAX as i64;
                self.regs.sr.c = c;
            },
        }

        Ok(exec_time)
    }

    pub(super) fn execute_nop(&self) -> InterpreterResult {
        Ok(EXEC::NOP)
    }

    pub(super) fn execute_not(&mut self, memory: &mut impl MemoryAccess, size: Size, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = single_operands_time(size.is_long(), am.is_drd(), EXEC::NOT_REG_BW, EXEC::NOT_REG_L, EXEC::NOT_MEM_BW, EXEC::NOT_MEM_L);

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte => {
                let data = !self.get_byte(memory, &mut ea, &mut exec_time)?;
                self.set_byte(memory, &mut ea, &mut exec_time, data)?;

                self.regs.sr.n = data & SIGN_BIT_8 != 0;
                self.regs.sr.z = data == 0;
            },
            Size::Word => {
                let data = !self.get_word(memory, &mut ea, &mut exec_time)?;
                self.set_word(memory, &mut ea, &mut exec_time, data)?;

                self.regs.sr.n = data & SIGN_BIT_16 != 0;
                self.regs.sr.z = data == 0;
            },
            Size::Long => {
                let data = !self.get_long(memory, &mut ea, &mut exec_time)?;
                self.set_long(memory, &mut ea, &mut exec_time, data)?;

                self.regs.sr.n = data & SIGN_BIT_32 != 0;
                self.regs.sr.z = data == 0;
            },
        }

        self.regs.sr.v = false;
        self.regs.sr.c = false;

        Ok(exec_time)
    }

    pub(super) fn execute_or(&mut self, memory: &mut impl MemoryAccess, reg: u8, dir: Direction, size: Size, am: AddressingMode) -> InterpreterResult {
        let mut exec_time;

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte => {
                if dir == Direction::DstEa {
                    exec_time = EXEC::OR_MEM_BW;
                } else {
                    exec_time = EXEC::OR_REG_BW;
                }
                let src = self.regs.d[reg as usize] as u8;
                let dst = self.get_byte(memory, &mut ea, &mut exec_time)?;

                let res = src | dst;

                self.regs.sr.n = res & SIGN_BIT_8 != 0;
                self.regs.sr.z = res == 0;

                if dir == Direction::DstEa {
                    self.set_byte(memory, &mut ea, &mut exec_time, res)?;
                } else {
                    self.d_byte(reg, res);
                }
            },
            Size::Word => {
                if dir == Direction::DstEa {
                    exec_time = EXEC::OR_MEM_BW;
                } else {
                    exec_time = EXEC::OR_REG_BW;
                }
                let src = self.regs.d[reg as usize] as u16;
                let dst = self.get_word(memory, &mut ea, &mut exec_time)?;

                let res = src | dst;

                self.regs.sr.n = res & SIGN_BIT_16 != 0;
                self.regs.sr.z = res == 0;

                if dir == Direction::DstEa {
                    self.set_word(memory, &mut ea, &mut exec_time, res)?;
                } else {
                    self.d_word(reg, res);
                }
            },
            Size::Long => {
                if dir == Direction::DstEa {
                    exec_time = EXEC::OR_MEM_L;
                } else {
                    exec_time = if am.is_dard() || am.is_immediate() { EXEC::OR_REG_L_RDIMM } else { EXEC::OR_REG_L };
                }
                let src = self.regs.d[reg as usize];
                let dst = self.get_long(memory, &mut ea, &mut exec_time)?;

                let res = src | dst;

                self.regs.sr.n = res & SIGN_BIT_32 != 0;
                self.regs.sr.z = res == 0;

                if dir == Direction::DstEa {
                    self.set_long(memory, &mut ea, &mut exec_time, res)?;
                } else {
                    self.regs.d[reg as usize] = res;
                }
            },
        }

        self.regs.sr.v = false;
        self.regs.sr.c = false;

        Ok(exec_time)
    }

    pub(super) fn execute_ori(&mut self, memory: &mut impl MemoryAccess, size: Size, am: AddressingMode, imm: u32) -> InterpreterResult {
        let mut exec_time;

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte => {
                exec_time = if am.is_drd() { EXEC::ORI_REG_BW } else { EXEC::ORI_MEM_BW };
                let data = self.get_byte(memory, &mut ea, &mut exec_time)? | imm as u8;
                self.set_byte(memory, &mut ea, &mut exec_time, data)?;

                self.regs.sr.n = data & SIGN_BIT_8 != 0;
                self.regs.sr.z = data == 0;
            },
            Size::Word => {
                exec_time = if am.is_drd() { EXEC::ORI_REG_BW } else { EXEC::ORI_MEM_BW };
                let data = self.get_word(memory, &mut ea, &mut exec_time)? | imm as u16;
                self.set_word(memory, &mut ea, &mut exec_time, data)?;

                self.regs.sr.n = data & SIGN_BIT_16 != 0;
                self.regs.sr.z = data == 0;
            },
            Size::Long => {
                exec_time = if am.is_drd() { EXEC::ORI_REG_L } else { EXEC::ORI_MEM_L };
                let data = self.get_long(memory, &mut ea, &mut exec_time)? | imm;
                self.set_long(memory, &mut ea, &mut exec_time, data)?;

                self.regs.sr.n = data & SIGN_BIT_32 != 0;
                self.regs.sr.z = data == 0;
            },
        }

        self.regs.sr.v = false;
        self.regs.sr.c = false;

        Ok(exec_time)
    }

    pub(super) fn execute_oriccr(&mut self, imm: u16) -> InterpreterResult {
        self.regs.sr |= imm;

        Ok(EXEC::ORICCR)
    }

    pub(super) fn execute_orisr(&mut self, imm: u16) -> InterpreterResult {
        self.check_supervisor()?;

        self.regs.sr |= imm;
        Ok(EXEC::ORISR)
    }

    pub(super) fn execute_pea(&mut self, memory: &mut impl MemoryAccess, am: AddressingMode) -> InterpreterResult {
        let mut ea = EffectiveAddress::new(am, None);

        let mut exec_time = 0;
        let addr = self.get_effective_address(&mut ea, &mut exec_time);
        self.push_long(memory, addr)?;

        Ok(match am {
            AddressingMode::Ari(_) => EXEC::PEA_ARI,
            AddressingMode::Ariwd(..) => EXEC::PEA_ARIWD,
            AddressingMode::Ariwi8(..) => EXEC::PEA_ARIWI8,
            AddressingMode::AbsShort(_) => EXEC::PEA_ABSSHORT,
            AddressingMode::AbsLong(_) => EXEC::PEA_ABSLONG,
            AddressingMode::Pciwd(..) => EXEC::PEA_PCIWD,
            AddressingMode::Pciwi8(..) => EXEC::PEA_PCIWI8,
            _ => panic!("Wrong addressing mode in PEA."),
        })
    }

    pub(super) fn execute_reset(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        self.check_supervisor()?;

        memory.reset_instruction();
        Ok(EXEC::RESET)
    }

    pub(super) fn execute_rom(&mut self, memory: &mut impl MemoryAccess, dir: Direction, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = EXEC::ROM;

        let mut ea = EffectiveAddress::new(am, Some(Size::Word));

        let mut data = self.get_word(memory, &mut ea, &mut exec_time)?;
        let sign = data & SIGN_BIT_16;

        if dir == Direction::Left {
            data <<= 1;
            data |= (sign != 0) as u16;
            self.regs.sr.c = sign != 0;
        } else {
            let bit = data & 1;
            data >>= 1;
            if bit != 0 {
                data |= SIGN_BIT_16;
            }
            self.regs.sr.c = bit != 0;
        }

        self.regs.sr.n = data & SIGN_BIT_16 != 0;
        self.regs.sr.z = data == 0;
        self.regs.sr.v = false;

        self.set_word(memory, &mut ea, &mut exec_time, data)?;

        Ok(exec_time)
    }

    pub(super) fn execute_ror(&mut self, rot: u8, dir: Direction, size: Size, mode: u8, reg: u8) -> InterpreterResult {
        self.regs.sr.v = false;
        self.regs.sr.c = false;

        let shift_count = if mode == 1 {
            (self.regs.d[rot as usize] % 64) as u8
        } else {
            if rot == 0 { 8 } else { rot }
        };

        let (mut data, mask) = match size {
            Size::Byte => (self.regs.d[reg as usize] & 0x0000_00FF, SIGN_BIT_8 as u32),
            Size::Word => (self.regs.d[reg as usize] & 0x0000_FFFF, SIGN_BIT_16 as u32),
            Size::Long => (self.regs.d[reg as usize], SIGN_BIT_32),
        };

        if dir == Direction::Left {
            for _ in 0..shift_count {
                let sign = data & mask;
                data <<= 1;
                if sign != 0 {
                    data |= 1;
                }
                self.regs.sr.c = sign != 0;
            }
        } else {
            for _ in 0..shift_count {
                let bit = data & 1;
                data >>= 1;
                if bit != 0 {
                    data |= mask;
                }
                self.regs.sr.c = bit != 0;
            }
        }

        self.regs.sr.n = data & mask != 0;

        Ok(match size {
            Size::Byte => {
                self.d_byte(reg, data as u8);
                self.regs.sr.z = data & 0x0000_00FF == 0;
                EXEC::ROR_BW + EXEC::ROR_COUNT * shift_count as usize
            },
            Size::Word => {
                self.d_word(reg, data as u16);
                self.regs.sr.z = data & 0x0000_FFFF == 0;
                EXEC::ROR_BW + EXEC::ROR_COUNT * shift_count as usize
            },
            Size::Long => {
                self.regs.d[reg as usize] = data;
                self.regs.sr.z = data == 0;
                EXEC::ROR_L + EXEC::ROR_COUNT * shift_count as usize
            },
        })
    }

    pub(super) fn execute_roxm(&mut self, memory: &mut impl MemoryAccess, dir: Direction, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = EXEC::ROXM;

        let mut ea = EffectiveAddress::new(am, Some(Size::Word));

        let mut data = self.get_word(memory, &mut ea, &mut exec_time)?;
        let sign = data & SIGN_BIT_16;

        if dir == Direction::Left {
            data <<= 1;
            data |= self.regs.sr.x as u16;
            self.regs.sr.x = sign != 0;
            self.regs.sr.c = sign != 0;
        } else {
            let bit = data & 1;
            data >>= 1;
            if self.regs.sr.x {
                data |= SIGN_BIT_16;
            }
            self.regs.sr.x = bit != 0;
            self.regs.sr.c = bit != 0;
        }

        self.regs.sr.n = data & SIGN_BIT_16 != 0;
        self.regs.sr.z = data == 0;
        self.regs.sr.v = false;

        self.set_word(memory, &mut ea, &mut exec_time, data)?;

        Ok(exec_time)
    }

    pub(super) fn execute_roxr(&mut self, rot: u8, dir: Direction, size: Size, mode: u8, reg: u8) -> InterpreterResult {
        self.regs.sr.v = false;
        self.regs.sr.c = self.regs.sr.x;

        let shift_count = if mode == 1 {
            (self.regs.d[rot as usize] % 64) as u8
        } else {
            if rot == 0 { 8 } else { rot }
        };

        let (mut data, mask) = match size {
            Size::Byte => (self.regs.d[reg as usize] & 0x0000_00FF, SIGN_BIT_8 as u32),
            Size::Word => (self.regs.d[reg as usize] & 0x0000_FFFF, SIGN_BIT_16 as u32),
            Size::Long => (self.regs.d[reg as usize], SIGN_BIT_32),
        };

        if dir == Direction::Left {
            for _ in 0..shift_count {
                let sign = data & mask;
                data <<= 1;
                data |= self.regs.sr.x as u32;
                self.regs.sr.x = sign != 0;
                self.regs.sr.c = sign != 0;
            }
        } else {
            for _ in 0..shift_count {
                let bit = data & 1;
                data >>= 1;
                if self.regs.sr.x {
                    data |= mask;
                }
                self.regs.sr.x = bit != 0;
                self.regs.sr.c = bit != 0;
            }
        }

        self.regs.sr.n = data & mask != 0;

        Ok(match size {
            Size::Byte => {
                self.d_byte(reg, data as u8);
                self.regs.sr.z = data & 0x0000_00FF == 0;
                EXEC::ROXR_BW + EXEC::ROXR_COUNT * shift_count as usize
            },
            Size::Word => {
                self.d_word(reg, data as u16);
                self.regs.sr.z = data & 0x0000_FFFF == 0;
                EXEC::ROXR_BW + EXEC::ROXR_COUNT * shift_count as usize
            },
            Size::Long => {
                self.regs.d[reg as usize] = data;
                self.regs.sr.z = data == 0;
                EXEC::ROXR_L + EXEC::ROXR_COUNT * shift_count as usize
            },
        })
    }

    pub(super) fn execute_rte(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        self.check_supervisor()?;

        let sr = self.pop_word(memory)?;
        self.regs.pc = self.pop_long(memory)?;
        #[allow(unused_mut)]
        let mut exec_time = EXEC::RTE;

        #[cfg(feature = "cpu-scc68070")] {
            let format = self.pop_word(memory)?;

            if format & 0xF000 == 0xF000 { // Long format
                *self.sp_mut() += 26;
                exec_time += 101;
                // TODO: execution times when rerun and rerun TAS.
            } else if format & 0xF000 != 0 {
                return Err(Vector::FormatError as u8);
            }
        }

        self.regs.sr = sr.into();

        Ok(exec_time)
    }

    pub(super) fn execute_rtr(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let ccr = self.pop_word(memory)?;
        self.regs.sr &= SR_UPPER_MASK;
        self.regs.sr |= ccr & CCR_MASK;
        self.regs.pc = self.pop_long(memory)?;

        Ok(EXEC::RTR)
    }

    pub(super) fn execute_rts(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        self.regs.pc = self.pop_long(memory)?;

        Ok(EXEC::RTS)
    }

    pub(super) fn execute_sbcd(&mut self, memory: &mut impl MemoryAccess, ry: u8, mode: Direction, rx: u8) -> InterpreterResult {
        let (src, dst) = if mode == Direction::MemoryToMemory {
            let src_addr = self.ariwpr(rx, Size::Byte);
            let dst_addr = self.ariwpr(ry, Size::Byte);
            (memory.get_byte(src_addr)?, memory.get_byte(dst_addr)?)
        } else {
            (self.regs.d[rx as usize] as u8, self.regs.d[ry as usize] as u8)
        };

        let low = (dst as i8 & 0x0F) - (src as i8 & 0x0F) - self.regs.sr.x as i8;
        let high = (dst as i8 >> 4 & 0x0F) - (src as i8 >> 4 & 0x0F) - (low < 0) as i8;
        let res = (if high < 0 { 10 + high } else { high } as u8) << 4 |
                      if low < 0 { 10 + low } else { low } as u8;

        if res != 0 { self.regs.sr.z = false; }
        self.regs.sr.c = high < 0;
        self.regs.sr.x = self.regs.sr.c;

        if mode == Direction::MemoryToMemory {
            memory.set_byte(self.a(ry), res)?;
            Ok(EXEC::SBCD_MEM)
        } else {
            self.d_byte(ry, res);
            Ok(EXEC::SBCD_REG)
        }
    }

    pub(super) fn execute_scc(&mut self, memory: &mut impl MemoryAccess, cc: u8, am: AddressingMode) -> InterpreterResult {
        let condition = self.regs.sr.condition(cc);
        let mut exec_time = single_operands_time(condition, am.is_drd(), EXEC::SCC_REG_FALSE, EXEC::SCC_REG_TRUE, EXEC::SCC_MEM_FALSE, EXEC::SCC_MEM_TRUE);

        let mut ea = EffectiveAddress::new(am, Some(Size::Byte));

        if condition {
            self.set_byte(memory, &mut ea, &mut exec_time, 0xFF)?;
        } else {
            self.set_byte(memory, &mut ea, &mut exec_time, 0)?;
        }

        Ok(exec_time)
    }

    pub(super) fn execute_stop(&mut self, imm: u16) -> InterpreterResult {
        self.check_supervisor()?;

        self.regs.sr = imm.into();
        self.stop = true;
        Ok(EXEC::STOP)
    }

    pub(super) fn execute_sub(&mut self, memory: &mut impl MemoryAccess, reg: u8, dir: Direction, size: Size, am: AddressingMode) -> InterpreterResult {
        let mut exec_time;

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte => {
                let (src, dst) = if dir == Direction::DstEa {
                    exec_time = EXEC::SUB_MEM_BW;
                    (self.regs.d[reg as usize] as u8, self.get_byte(memory, &mut ea, &mut exec_time)?)
                } else {
                    exec_time = EXEC::SUB_REG_BW;
                    (self.get_byte(memory, &mut ea, &mut exec_time)?, self.regs.d[reg as usize] as u8)
                };

                let (res, v) = (dst as i8).overflowing_sub(src as i8);
                let (_, c) = dst.overflowing_sub(src);

                self.regs.sr.x = c;
                self.regs.sr.n = res < 0;
                self.regs.sr.z = res == 0;
                self.regs.sr.v = v;
                self.regs.sr.c = c;

                if dir == Direction::DstEa {
                    self.set_byte(memory, &mut ea, &mut exec_time, res as u8)?;
                } else {
                    self.d_byte(reg, res as u8);
                }
            },
            Size::Word => {
                let (src, dst) = if dir == Direction::DstEa {
                    exec_time = EXEC::SUB_MEM_BW;
                    (self.regs.d[reg as usize] as u16, self.get_word(memory, &mut ea, &mut exec_time)?)
                } else {
                    exec_time = EXEC::SUB_REG_BW;
                    (self.get_word(memory, &mut ea, &mut exec_time)?, self.regs.d[reg as usize] as u16)
                };

                let (res, v) = (dst as i16).overflowing_sub(src as i16);
                let (_, c) = dst.overflowing_sub(src);

                self.regs.sr.x = c;
                self.regs.sr.n = res < 0;
                self.regs.sr.z = res == 0;
                self.regs.sr.v = v;
                self.regs.sr.c = c;

                if dir == Direction::DstEa {
                    self.set_word(memory, &mut ea, &mut exec_time, res as u16)?;
                } else {
                    self.d_word(reg, res as u16);
                }
            },
            Size::Long => {
                let (src, dst) = if dir == Direction::DstEa {
                    exec_time = EXEC::SUB_MEM_L;
                    (self.regs.d[reg as usize] as u32, self.get_long(memory, &mut ea, &mut exec_time)?)
                } else {
                    exec_time = if am.is_dard() || am.is_immediate() { EXEC::SUB_REG_L_RDIMM } else { EXEC::SUB_REG_L };
                    (self.get_long(memory, &mut ea, &mut exec_time)?, self.regs.d[reg as usize] as u32)
                };

                let (res, v) = (dst as i32).overflowing_sub(src as i32);
                let (_, c) = dst.overflowing_sub(src);

                self.regs.sr.x = c;
                self.regs.sr.n = res < 0;
                self.regs.sr.z = res == 0;
                self.regs.sr.v = v;
                self.regs.sr.c = c;

                if dir == Direction::DstEa {
                    self.set_long(memory, &mut ea, &mut exec_time, res as u32)?;
                } else {
                    self.regs.d[reg as usize] = res as u32;
                }
            },
        }

        Ok(exec_time)
    }

    pub(super) fn execute_suba(&mut self, memory: &mut impl MemoryAccess, reg: u8, size: Size, am: AddressingMode) -> InterpreterResult {
        let mut exec_time;

        let mut ea = EffectiveAddress::new(am, Some(size));

        let src = if size.is_word() {
            exec_time = EXEC::SUBA_WORD;
            self.get_word(memory, &mut ea, &mut exec_time)? as i16 as u32
        } else {
            exec_time = if am.is_dard() || am.is_immediate() {
                EXEC::SUBA_LONG_RDIMM
            } else {
                EXEC::SUBA_LONG
            };
            self.get_long(memory, &mut ea, &mut exec_time)?
        };

        *self.a_mut(reg) -= src;

        Ok(exec_time)
    }

    pub(super) fn execute_subi(&mut self, memory: &mut impl MemoryAccess, size: Size, am: AddressingMode, imm: u32) -> InterpreterResult {
        let mut exec_time;

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte => {
                exec_time = if am.is_drd() { EXEC::SUBI_REG_BW } else { EXEC::SUBI_MEM_BW };
                let data = self.get_byte(memory, &mut ea, &mut exec_time)?;
                let (res, v) = (data as i8).overflowing_sub(imm as i8);
                let (_, c) = data.overflowing_sub(imm as u8);
                self.set_byte(memory, &mut ea, &mut exec_time, res as u8)?;

                self.regs.sr.x = c;
                self.regs.sr.n = res < 0;
                self.regs.sr.z = res == 0;
                self.regs.sr.v = v;
                self.regs.sr.c = c;
            },
            Size::Word => {
                exec_time = if am.is_drd() { EXEC::SUBI_REG_BW } else { EXEC::SUBI_MEM_BW };
                let data = self.get_word(memory, &mut ea, &mut exec_time)?;
                let (res, v) = (data as i16).overflowing_sub(imm as i16);
                let (_, c) = data.overflowing_sub(imm as u16);
                self.set_word(memory, &mut ea, &mut exec_time, res as u16)?;

                self.regs.sr.x = c;
                self.regs.sr.n = res < 0;
                self.regs.sr.z = res == 0;
                self.regs.sr.v = v;
                self.regs.sr.c = c;
            },
            Size::Long => {
                exec_time = if am.is_drd() { EXEC::SUBI_REG_L } else { EXEC::SUBI_MEM_L };
                let data = self.get_long(memory, &mut ea, &mut exec_time)?;
                let (res, v) = (data as i32).overflowing_sub(imm as i32);
                let (_, c) = data.overflowing_sub(imm);
                self.set_long(memory, &mut ea, &mut exec_time, res as u32)?;

                self.regs.sr.x = c;
                self.regs.sr.n = res < 0;
                self.regs.sr.z = res == 0;
                self.regs.sr.v = v;
                self.regs.sr.c = c;
            },
        }

        Ok(exec_time)
    }

    pub(super) fn execute_subq(&mut self, memory: &mut impl MemoryAccess, imm: u8, size: Size, am: AddressingMode) -> InterpreterResult {
        let mut exec_time;

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte => {
                exec_time = if am.is_drd() { EXEC::SUBQ_DREG_BW } else { EXEC::SUBQ_MEM_BW };
                let data = self.get_byte(memory, &mut ea, &mut exec_time)?;
                let (res, v) = (data as i8).overflowing_sub(imm as i8);
                let (_, c) = data.overflowing_sub(imm);
                self.set_byte(memory, &mut ea, &mut exec_time, res as u8)?;

                self.regs.sr.x = c;
                self.regs.sr.n = res < 0;
                self.regs.sr.z = res == 0;
                self.regs.sr.v = v;
                self.regs.sr.c = c;
            },
            Size::Word => {
                exec_time = if am.is_drd() {
                    EXEC::SUBQ_DREG_BW
                } else if am.is_ard() {
                    EXEC::SUBQ_AREG_BW
                } else {
                    EXEC::SUBQ_MEM_BW
                };
                let data = self.get_word(memory, &mut ea, &mut exec_time)?;
                let (res, v) = (data as i16).overflowing_sub(imm as i16);
                let (_, c) = data.overflowing_sub(imm as u16);
                self.set_word(memory, &mut ea, &mut exec_time, res as u16)?;

                if !ea.mode.is_ard() {
                    self.regs.sr.x = c;
                    self.regs.sr.n = res < 0;
                    self.regs.sr.z = res == 0;
                    self.regs.sr.v = v;
                    self.regs.sr.c = c;
                }
            },
            Size::Long => {
                exec_time = if am.is_dard() { EXEC::SUBQ_REG_L } else { EXEC::SUBQ_MEM_L };
                let data = self.get_long(memory, &mut ea, &mut exec_time)?;
                let (res, v) = (data as i32).overflowing_sub(imm as i32);
                let (_, c) = data.overflowing_sub(imm as u32);
                self.set_long(memory, &mut ea, &mut exec_time, res as u32)?;

                if !ea.mode.is_ard() {
                    self.regs.sr.x = c;
                    self.regs.sr.n = res < 0;
                    self.regs.sr.z = res == 0;
                    self.regs.sr.v = v;
                    self.regs.sr.c = c;
                }
            },
        }

        Ok(exec_time)
    }

    pub(super) fn execute_subx(&mut self, memory: &mut impl MemoryAccess, ry: u8, size: Size, mode: Direction, rx: u8) -> InterpreterResult {
        match size {
            Size::Byte => {
                let (src, dst) = if mode == Direction::MemoryToMemory {
                    let src_addr = self.ariwpr(rx, size);
                    let dst_addr = self.ariwpr(ry, size);
                    (memory.get_byte(src_addr)?, memory.get_byte(dst_addr)?)
                } else {
                    (self.regs.d[rx as usize] as u8, self.regs.d[ry as usize] as u8)
                };

                let (res, v) = (dst as i8).extended_sub(src as i8, self.regs.sr.x);
                let (_, c) = dst.extended_sub(src, self.regs.sr.x);

                self.regs.sr.n = res < 0;
                if res != 0 {
                    self.regs.sr.z = false;
                }
                self.regs.sr.v = v;
                self.regs.sr.c = c;
                self.regs.sr.x = c;

                if mode == Direction::MemoryToMemory {
                    memory.set_byte(self.a(ry), res as u8)?;
                    Ok(EXEC::SUBX_MEM_BW)
                } else {
                    self.d_byte(ry, res as u8);
                    Ok(EXEC::SUBX_REG_BW)
                }
            },
            Size::Word => {
                let (src, dst) = if mode == Direction::MemoryToMemory {
                    let src_addr = self.ariwpr(rx, size);
                    let dst_addr = self.ariwpr(ry, size);
                    (memory.get_word(src_addr.even()?)?, memory.get_word(dst_addr.even()?)?)
                } else {
                    (self.regs.d[rx as usize] as u16, self.regs.d[ry as usize] as u16)
                };

                let (res, v) = (dst as i16).extended_sub(src as i16, self.regs.sr.x);
                let (_, c) = dst.extended_sub(src, self.regs.sr.x);

                self.regs.sr.n = res < 0;
                if res != 0 {
                    self.regs.sr.z = false;
                }
                self.regs.sr.v = v;
                self.regs.sr.c = c;
                self.regs.sr.x = c;

                if mode == Direction::MemoryToMemory {
                    memory.set_word(self.a(ry), res as u16)?;
                    Ok(EXEC::SUBX_MEM_BW)
                } else {
                    self.d_word(ry, res as u16);
                    Ok(EXEC::SUBX_REG_BW)
                }
            },
            Size::Long => {
                let (src, dst) = if mode == Direction::MemoryToMemory {
                    let src_addr = self.ariwpr(rx, size);
                    let dst_addr = self.ariwpr(ry, size);
                    (memory.get_long(src_addr.even()?)?, memory.get_long(dst_addr.even()?)?)
                } else {
                    (self.regs.d[rx as usize], self.regs.d[ry as usize])
                };

                let (res, v) = (dst as i32).extended_sub(src as i32, self.regs.sr.x);
                let (_, c) = dst.extended_sub(src, self.regs.sr.x);

                self.regs.sr.n = res < 0;
                if res != 0 {
                    self.regs.sr.z = false;
                }
                self.regs.sr.v = v;
                self.regs.sr.c = c;
                self.regs.sr.x = c;

                if mode == Direction::MemoryToMemory {
                    memory.set_long(self.a(ry), res as u32)?;
                    Ok(EXEC::SUBX_MEM_L)
                } else {
                    self.regs.d[ry as usize] = res as u32;
                    Ok(EXEC::SUBX_REG_L)
                }
            },
        }
    }

    pub(super) fn execute_swap(&mut self, reg: u8) -> InterpreterResult {
        let high = self.regs.d[reg as usize] >> 16;
        self.regs.d[reg as usize] <<= 16;
        self.regs.d[reg as usize] |= high;

        self.regs.sr.n = self.regs.d[reg as usize] & SIGN_BIT_32 != 0;
        self.regs.sr.z = self.regs.d[reg as usize] == 0;
        self.regs.sr.v = false;
        self.regs.sr.c = false;

        Ok(EXEC::SWAP)
    }

    pub(super) fn execute_tas(&mut self, memory: &mut impl MemoryAccess, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = if am.is_drd() { EXEC::TAS_REG } else { EXEC::TAS_MEM };

        let mut ea = EffectiveAddress::new(am, Some(Size::Byte));

        let mut data = self.get_byte(memory, &mut ea, &mut exec_time)?;

        self.regs.sr.n = data & SIGN_BIT_8 != 0;
        self.regs.sr.z = data == 0;
        self.regs.sr.v = false;
        self.regs.sr.c = false;

        data |= SIGN_BIT_8;
        self.set_byte(memory, &mut ea, &mut exec_time, data)?;

        Ok(exec_time)
    }

    pub(super) fn execute_trap(&mut self, vector: u8) -> InterpreterResult {
        Err(Vector::Trap0Instruction as u8 + vector)
    }

    pub(super) fn execute_trapv(&self) -> InterpreterResult {
        if self.regs.sr.v {
            Err(Vector::TrapVInstruction as u8)
        } else {
            Ok(EXEC::TRAPV_NO_TRAP)
        }
    }

    pub(super) fn execute_tst(&mut self, memory: &mut impl MemoryAccess, size: Size, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = single_operands_time(size.is_long(), am.is_drd(), EXEC::TST_REG_BW, EXEC::TST_REG_L, EXEC::TST_MEM_BW, EXEC::TST_MEM_L);

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte => {
                let data = self.get_byte(memory, &mut ea, &mut exec_time)?;
                self.regs.sr.n = data & SIGN_BIT_8 != 0;
                self.regs.sr.z = data == 0;
            },
            Size::Word => {
                let data = self.get_word(memory, &mut ea, &mut exec_time)?;
                self.regs.sr.n = data & SIGN_BIT_16 != 0;
                self.regs.sr.z = data == 0;
            },
            Size::Long => {
                let data = self.get_long(memory, &mut ea, &mut exec_time)?;
                self.regs.sr.n = data & SIGN_BIT_32 != 0;
                self.regs.sr.z = data == 0;
            },
        }

        self.regs.sr.v = false;
        self.regs.sr.c = false;

        Ok(exec_time)
    }

    pub(super) fn execute_unlk(&mut self, memory: &mut impl MemoryAccess, reg: u8) -> InterpreterResult {
        *self.sp_mut() = self.a(reg);
        *self.a_mut(reg) = self.pop_long(memory)?;

        Ok(EXEC::UNLK)
    }
}

#[inline(always)]
const fn single_operands_time(is_long: bool, in_register: bool, regbw: usize, regl: usize, membw: usize, meml: usize) -> usize {
    if in_register {
        if is_long {
            regl
        } else {
            regbw
        }
    } else {
        if is_long {
            meml
        } else {
            membw
        }
    }
}
