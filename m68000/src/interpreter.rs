// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{CpuDetails, M68000, MemoryAccess, StackFormat};
use crate::addressing_modes::{EffectiveAddress, AddressingMode};
use crate::exception::{ACCESS_ERROR, Vector};
use crate::instruction::{Direction, Size};
use crate::utils::{bits, IsEven};

pub(super) const SR_UPPER_MASK: u16 = 0xA700;
pub(super) const CCR_MASK: u16 = 0x001F;
pub(super) const SIGN_BIT_8: u8 = 0x80;
pub(super) const SIGN_BIT_16: u16 = 0x8000;
pub(super) const SIGN_BIT_32: u32 = 0x8000_0000;

/// Returns the execution time on success, an exception vector on error. Alias for `Result<usize, u8>`.
pub(super) type InterpreterResult = Result<usize, u8>;

// TODO: return a tuple with the current execution time and the exception that occured (CHK, DIVS, DIVU).
// All this for only 3 instructions ?

impl<CPU: CpuDetails> M68000<CPU> {
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
            (memory.get_byte(src_addr).ok_or(ACCESS_ERROR)?, memory.get_byte(dst_addr).ok_or(ACCESS_ERROR)?)
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
            memory.set_byte(self.regs.a(rx), res).ok_or(ACCESS_ERROR)?;
            Ok(CPU::ABCD_MEM)
        } else {
            self.regs.d_byte(rx, res);
            Ok(CPU::ABCD_REG)
        }
    }

    pub(super) fn execute_add(&mut self, memory: &mut impl MemoryAccess, reg: u8, dir: Direction, size: Size, am: AddressingMode) -> InterpreterResult {
        let mut exec_time;

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte => {
                let (src, dst) = if dir == Direction::DstEa {
                    exec_time = CPU::ADD_MEM_BW;
                    (self.regs.d[reg as usize] as u8, self.get_byte(memory, &mut ea, &mut exec_time)?)
                } else {
                    exec_time = CPU::ADD_REG_BW;
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
                    self.regs.d_byte(reg, res as u8);
                }
            },
            Size::Word => {
                let (src, dst) = if dir == Direction::DstEa {
                    exec_time = CPU::ADD_MEM_BW;
                    (self.regs.d[reg as usize] as u16, self.get_word(memory, &mut ea, &mut exec_time)?)
                } else {
                    exec_time = CPU::ADD_REG_BW;
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
                    self.regs.d_word(reg, res as u16);
                }
            },
            Size::Long => {
                let (src, dst) = if dir == Direction::DstEa {
                    exec_time = CPU::ADD_MEM_L;
                    (self.regs.d[reg as usize] as u32, self.get_long(memory, &mut ea, &mut exec_time)?)
                } else {
                    exec_time = if am.is_dard() || am.is_immediate() { CPU::ADD_REG_L_RDIMM } else { CPU::ADD_REG_L };
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
            exec_time = CPU::ADDA_WORD;
            self.get_word(memory, &mut ea, &mut exec_time)? as i16 as u32
        } else {
            exec_time = if am.is_dard() || am.is_immediate() {
                CPU::ADDA_LONG_RDIMM
            } else {
                CPU::ADDA_LONG
            };
            self.get_long(memory, &mut ea, &mut exec_time)?
        };

        *self.regs.a_mut(reg) += src;

        Ok(exec_time)
    }

    pub(super) fn execute_addi(&mut self, memory: &mut impl MemoryAccess, size: Size, am: AddressingMode, imm: u32) -> InterpreterResult {
        let mut exec_time;

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte =>  {
                exec_time = if am.is_drd() { CPU::ADDI_REG_BW } else { CPU::ADDI_MEM_BW };
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
                exec_time = if am.is_drd() { CPU::ADDI_REG_BW } else { CPU::ADDI_MEM_BW };
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
                exec_time = if am.is_drd() { CPU::ADDI_REG_L } else { CPU::ADDI_MEM_L };
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
        if am.is_ard() {
            *self.regs.a_mut(am.register().unwrap()) += imm as u32;
            return Ok(if size.is_long() { CPU::ADDQ_REG_L } else { CPU::ADDQ_REG_BW });
        }

        let mut exec_time;

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte => {
                exec_time = if am.is_drd() { CPU::ADDQ_REG_BW } else { CPU::ADDQ_MEM_BW };
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
                exec_time = if am.is_drd() { CPU::ADDQ_REG_BW } else { CPU::ADDQ_MEM_BW };
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
                exec_time = if am.is_drd() { CPU::ADDQ_REG_L } else { CPU::ADDQ_MEM_L };
                let data = self.get_long(memory, &mut ea, &mut exec_time)?;
                let (res, v) = (data as i32).overflowing_add(imm as i32);
                let (_, c) = data.overflowing_add(imm as u32);
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

    pub(super) fn execute_addx(&mut self, memory: &mut impl MemoryAccess, rx: u8, size: Size, mode: Direction, ry: u8) -> InterpreterResult {
        match size {
            Size::Byte => {
                let (src, dst) = if mode == Direction::MemoryToMemory {
                    let src_addr = self.ariwpr(ry, size);
                    let dst_addr = self.ariwpr(rx, size);
                    (memory.get_byte(src_addr).ok_or(ACCESS_ERROR)?, memory.get_byte(dst_addr).ok_or(ACCESS_ERROR)?)
                } else {
                    (self.regs.d[ry as usize] as u8, self.regs.d[rx as usize] as u8)
                };

                let (res, v) = (src as i8).carrying_add(dst as i8, self.regs.sr.x);
                let (_, c) = src.carrying_add(dst, self.regs.sr.x);

                self.regs.sr.x = c;
                self.regs.sr.n = res < 0;
                if res != 0 {
                    self.regs.sr.z = false;
                }
                self.regs.sr.v = v;
                self.regs.sr.c = c;

                if mode == Direction::MemoryToMemory {
                    memory.set_byte(self.regs.a(rx), res as u8).ok_or(ACCESS_ERROR)?;
                    Ok(CPU::ADDX_MEM_BW)
                } else {
                    self.regs.d_byte(rx, res as u8);
                    Ok(CPU::ADDX_REG_BW)
                }
            },
            Size::Word => {
                let (src, dst) = if mode == Direction::MemoryToMemory {
                    let src_addr = self.ariwpr(ry, size);
                    let dst_addr = self.ariwpr(rx, size);
                    (memory.get_word(src_addr.even()?).ok_or(ACCESS_ERROR)?, memory.get_word(dst_addr.even()?).ok_or(ACCESS_ERROR)?)
                } else {
                    (self.regs.d[ry as usize] as u16, self.regs.d[rx as usize] as u16)
                };

                let (res, v) = (src as i16).carrying_add(dst as i16, self.regs.sr.x);
                let (_, c) = src.carrying_add(dst, self.regs.sr.x);

                self.regs.sr.x = c;
                self.regs.sr.n = res < 0;
                if res != 0 {
                    self.regs.sr.z = false;
                }
                self.regs.sr.v = v;
                self.regs.sr.c = c;

                if mode == Direction::MemoryToMemory {
                    memory.set_word(self.regs.a(rx), res as u16).ok_or(ACCESS_ERROR)?;
                    Ok(CPU::ADDX_MEM_BW)
                } else {
                    self.regs.d_word(rx, res as u16);
                    Ok(CPU::ADDX_REG_BW)
                }
            },
            Size::Long => {
                let (src, dst) = if mode == Direction::MemoryToMemory {
                    let src_addr = self.ariwpr(ry, size);
                    let dst_addr = self.ariwpr(rx, size);
                    (memory.get_long(src_addr.even()?).ok_or(ACCESS_ERROR)?, memory.get_long(dst_addr.even()?).ok_or(ACCESS_ERROR)?)
                } else {
                    (self.regs.d[ry as usize], self.regs.d[rx as usize])
                };

                let (res, v) = (src as i32).carrying_add(dst as i32, self.regs.sr.x);
                let (_, c) = src.carrying_add(dst, self.regs.sr.x);

                self.regs.sr.x = c;
                self.regs.sr.n = res < 0;
                if res != 0 {
                    self.regs.sr.z = false;
                }
                self.regs.sr.v = v;
                self.regs.sr.c = c;

                if mode == Direction::MemoryToMemory {
                    memory.set_long(self.regs.a(rx), res as u32).ok_or(ACCESS_ERROR)?;
                    Ok(CPU::ADDX_MEM_L)
                } else {
                    self.regs.d[rx as usize] = res as u32;
                    Ok(CPU::ADDX_REG_L)
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
                    exec_time = CPU::AND_MEM_BW;
                } else {
                    exec_time = CPU::AND_REG_BW;
                }
                let src = self.regs.d[reg as usize] as u8;
                let dst = self.get_byte(memory, &mut ea, &mut exec_time)?;

                let res = src & dst;

                self.regs.sr.n = res & SIGN_BIT_8 != 0;
                self.regs.sr.z = res == 0;

                if dir == Direction::DstEa {
                    self.set_byte(memory, &mut ea, &mut exec_time, res)?;
                } else {
                    self.regs.d_byte(reg, res);
                }
            },
            Size::Word => {
                if dir == Direction::DstEa {
                    exec_time = CPU::AND_MEM_BW;
                } else {
                    exec_time = CPU::AND_REG_BW;
                }
                let src = self.regs.d[reg as usize] as u16;
                let dst = self.get_word(memory, &mut ea, &mut exec_time)?;

                let res = src & dst;

                self.regs.sr.n = res & SIGN_BIT_16 != 0;
                self.regs.sr.z = res == 0;

                if dir == Direction::DstEa {
                    self.set_word(memory, &mut ea, &mut exec_time, res)?;
                } else {
                    self.regs.d_word(reg, res);
                }
            },
            Size::Long => {
                if dir == Direction::DstEa {
                    exec_time = CPU::AND_MEM_L;
                } else {
                    exec_time = if am.is_dard() || am.is_immediate() { CPU::AND_REG_L_RDIMM } else { CPU::AND_REG_L };
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
                exec_time = if am.is_drd() { CPU::ANDI_REG_BW } else { CPU::ANDI_MEM_BW };
                let data = self.get_byte(memory, &mut ea, &mut exec_time)? & imm as u8;
                self.set_byte(memory, &mut ea, &mut exec_time, data)?;

                self.regs.sr.n = data & SIGN_BIT_8 != 0;
                self.regs.sr.z = data == 0;
            },
            Size::Word => {
                exec_time = if am.is_drd() { CPU::ANDI_REG_BW } else { CPU::ANDI_MEM_BW };
                let data = self.get_word(memory, &mut ea, &mut exec_time)? & imm as u16;
                self.set_word(memory, &mut ea, &mut exec_time, data)?;

                self.regs.sr.n = data & SIGN_BIT_16 != 0;
                self.regs.sr.z = data == 0;
            },
            Size::Long => {
                exec_time = if am.is_drd() { CPU::ANDI_REG_L } else { CPU::ANDI_MEM_L };
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

        Ok(CPU::ANDICCR)
    }

    pub(super) fn execute_andisr(&mut self, imm: u16) -> InterpreterResult {
        self.check_supervisor()?;

        self.regs.sr &= imm;
        Ok(CPU::ANDISR)
    }

    pub(super) fn execute_asm(&mut self, memory: &mut impl MemoryAccess, dir: Direction, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = CPU::ASM;

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
                self.regs.d_byte(reg, data as u8);
                self.regs.sr.z = data & 0x0000_00FF == 0;
                CPU::ASR_BW + CPU::ASR_COUNT * shift_count as usize
            },
            Size::Word => {
                self.regs.d_word(reg, data as u16);
                self.regs.sr.z = data & 0x0000_FFFF == 0;
                CPU::ASR_BW + CPU::ASR_COUNT * shift_count as usize
            },
            Size::Long => {
                self.regs.d[reg as usize] = data;
                self.regs.sr.z = data == 0;
                CPU::ASR_L + CPU::ASR_COUNT * shift_count as usize
            }
        })
    }

    pub(super) fn execute_bcc(&mut self, pc: u32, condition: u8, displacement: i16) -> InterpreterResult {
        if self.regs.sr.condition(condition) {
            self.regs.pc = pc + displacement as u32;
            Ok(CPU::BCC_BRANCH)
        } else {
            Ok(if self.current_opcode as u8 == 0 {
                CPU::BCC_NO_BRANCH_WORD
            } else {
                CPU::BCC_NO_BRANCH_BYTE
            })
        }
    }

    pub(super) fn execute_bchg(&mut self, memory: &mut impl MemoryAccess, am: AddressingMode, mut count: u8) -> InterpreterResult {
        let mut exec_time = if bits(self.current_opcode, 8, 8) != 0 {
            count = self.regs.d[count as usize] as u8;
            if am.is_drd() { CPU::BCHG_DYN_REG } else { CPU::BCHG_DYN_MEM }
        } else {
            if am.is_drd() { CPU::BCHG_STA_REG } else { CPU::BCHG_STA_MEM }
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
            if am.is_drd() { CPU::BCLR_DYN_REG } else { CPU::BCLR_DYN_MEM }
        } else {
            if am.is_drd() { CPU::BCLR_STA_REG } else { CPU::BCLR_STA_MEM }
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
            CPU::BRA_WORD
        } else {
            CPU::BRA_BYTE
        })
    }

    pub(super) fn execute_bset(&mut self, memory: &mut impl MemoryAccess, am: AddressingMode, mut count: u8) -> InterpreterResult {
        let mut exec_time = if bits(self.current_opcode, 8, 8) != 0 {
            count = self.regs.d[count as usize] as u8;
            if am.is_drd() { CPU::BSET_DYN_REG } else { CPU::BSET_DYN_MEM }
        } else {
            if am.is_drd() { CPU::BSET_STA_REG } else { CPU::BSET_STA_MEM }
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
            CPU::BSR_WORD
        } else {
            CPU::BSR_BYTE
        })
    }

    pub(super) fn execute_btst(&mut self, memory: &mut impl MemoryAccess, am: AddressingMode, mut count: u8) -> InterpreterResult {
        let mut exec_time = if bits(self.current_opcode, 8, 8) != 0 {
            count = self.regs.d[count as usize] as u8;
            if am.is_drd() { CPU::BTST_DYN_REG } else { CPU::BTST_DYN_MEM }
        } else {
            if am.is_drd() { CPU::BTST_STA_REG } else { CPU::BTST_STA_MEM }
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
            Ok(CPU::CHK_NO_TRAP + exec_time)
        }
    }

    pub(super) fn execute_clr(&mut self, memory: &mut impl MemoryAccess, size: Size, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = single_operands_time(size.is_long(), am.is_drd(), CPU::CLR_REG_BW, CPU::CLR_REG_L, CPU::CLR_MEM_BW, CPU::CLR_MEM_L);

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
                exec_time = CPU::CMP_BW;
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
                exec_time = CPU::CMP_BW;
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
                exec_time = CPU::CMP_L;
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
        let mut exec_time = CPU::CMPA;

        let mut ea = EffectiveAddress::new(am, Some(size));

        let src = if size.is_word() {
            self.get_word(memory, &mut ea, &mut exec_time)? as i16 as u32
        } else {
            self.get_long(memory, &mut ea, &mut exec_time)?
        };

        let (res, v) = (self.regs.a(reg) as i32).overflowing_sub(src as i32);
        let (_, c) = self.regs.a(reg).overflowing_sub(src);

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
                exec_time = if am.is_drd() { CPU::CMPI_REG_BW } else { CPU::CMPI_MEM_BW };
                let data = self.get_byte(memory, &mut ea, &mut exec_time)?;
                let (res, v) = (data as i8).overflowing_sub(imm as i8);
                let (_, c) = data.overflowing_sub(imm as u8);

                self.regs.sr.n = res < 0;
                self.regs.sr.z = res == 0;
                self.regs.sr.v = v;
                self.regs.sr.c = c;
            },
            Size::Word => {
                exec_time = if am.is_drd() { CPU::CMPI_REG_BW } else { CPU::CMPI_MEM_BW };
                let data = self.get_word(memory, &mut ea, &mut exec_time)?;
                let (res, v) = (data as i16).overflowing_sub(imm as i16);
                let (_, c) = data.overflowing_sub(imm as u16);

                self.regs.sr.n = res < 0;
                self.regs.sr.z = res == 0;
                self.regs.sr.v = v;
                self.regs.sr.c = c;
            },
            Size::Long => {
                exec_time = if am.is_drd() { CPU::CMPI_REG_L } else { CPU::CMPI_MEM_L };
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
                let src = memory.get_byte(addry).ok_or(ACCESS_ERROR)?;
                let dst = memory.get_byte(addrx).ok_or(ACCESS_ERROR)?;

                let (res, v) = (dst as i8).overflowing_sub(src as i8);
                let (_, c) = dst.overflowing_sub(src);

                self.regs.sr.n = res < 0;
                self.regs.sr.z = res == 0;
                self.regs.sr.v = v;
                self.regs.sr.c = c;

                Ok(CPU::CMPM_BW)
            },
            Size::Word => {
                let src = memory.get_word(addry.even()?).ok_or(ACCESS_ERROR)?;
                let dst = memory.get_word(addrx.even()?).ok_or(ACCESS_ERROR)?;

                let (res, v) = (dst as i16).overflowing_sub(src as i16);
                let (_, c) = dst.overflowing_sub(src);

                self.regs.sr.n = res < 0;
                self.regs.sr.z = res == 0;
                self.regs.sr.v = v;
                self.regs.sr.c = c;

                Ok(CPU::CMPM_BW)
            },
            Size::Long => {
                let src = memory.get_long(addry.even()?).ok_or(ACCESS_ERROR)?;
                let dst = memory.get_long(addrx.even()?).ok_or(ACCESS_ERROR)?;

                let (res, v) = (dst as i32).overflowing_sub(src as i32);
                let (_, c) = dst.overflowing_sub(src);

                self.regs.sr.n = res < 0;
                self.regs.sr.z = res == 0;
                self.regs.sr.v = v;
                self.regs.sr.c = c;

                Ok(CPU::CMPM_L)
            },
        }
    }

    pub(super) fn execute_dbcc(&mut self, pc: u32, cc: u8, reg: u8, disp: i16) -> InterpreterResult {
        if !self.regs.sr.condition(cc) {
            let counter = self.regs.d[reg as usize] as i16 - 1;
            self.regs.d_word(reg, counter as u16);

            if counter != -1 {
                self.regs.pc = pc + disp as u32;
                Ok(CPU::DBCC_FALSE_BRANCH)
            } else {
                Ok(CPU::DBCC_FALSE_NO_BRANCH)
            }
        } else {
            Ok(CPU::DBCC_TRUE)
        }
    }

    /// If a zero divide exception occurs, this method returns the effective address calculation time, and the
    /// process_exception method returns the exception processing time.
    pub(super) fn execute_divs(&mut self, memory: &mut impl MemoryAccess, reg: u8, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = CPU::DIVS;

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
        let mut exec_time = CPU::DIVU;

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
                exec_time = if am.is_drd() { CPU::EOR_REG_BW } else { CPU::EOR_MEM_BW };
                let src = self.regs.d[reg as usize] as u8;
                let dst = self.get_byte(memory, &mut ea, &mut exec_time)?;

                let res = src ^ dst;

                self.regs.sr.n = res & SIGN_BIT_8 != 0;
                self.regs.sr.z = res == 0;

                self.set_byte(memory, &mut ea, &mut exec_time, res)?;
            },
            Size::Word => {
                exec_time = if am.is_drd() { CPU::EOR_REG_BW } else { CPU::EOR_MEM_BW };
                let src = self.regs.d[reg as usize] as u16;
                let dst = self.get_word(memory, &mut ea, &mut exec_time)?;

                let res = src ^ dst;

                self.regs.sr.n = res & SIGN_BIT_16 != 0;
                self.regs.sr.z = res == 0;

                self.set_word(memory, &mut ea, &mut exec_time, res)?;
            },
            Size::Long => {
                exec_time = if am.is_drd() { CPU::EOR_REG_L } else { CPU::EOR_MEM_L };
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
                exec_time = if am.is_drd() { CPU::EORI_REG_BW } else { CPU::EORI_MEM_BW };
                let data = self.get_byte(memory, &mut ea, &mut exec_time)? ^ imm as u8;
                self.set_byte(memory, &mut ea, &mut exec_time, data)?;

                self.regs.sr.n = data & SIGN_BIT_8 != 0;
                self.regs.sr.z = data == 0;
            },
            Size::Word => {
                exec_time = if am.is_drd() { CPU::EORI_REG_BW } else { CPU::EORI_MEM_BW };
                let data = self.get_word(memory, &mut ea, &mut exec_time)? ^ imm as u16;
                self.set_word(memory, &mut ea, &mut exec_time, data)?;

                self.regs.sr.n = data & SIGN_BIT_16 != 0;
                self.regs.sr.z = data == 0;
            },
            Size::Long => {
                exec_time = if am.is_drd() { CPU::EORI_REG_L } else { CPU::EORI_MEM_L };
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

        Ok(CPU::EORICCR)
    }

    pub(super) fn execute_eorisr(&mut self, imm: u16) -> InterpreterResult {
        self.check_supervisor()?;

        self.regs.sr ^= imm;
        Ok(CPU::EORISR)
    }

    pub(super) fn execute_exg(&mut self, rx: u8, mode: Direction, ry: u8) -> InterpreterResult {
        if mode == Direction::ExchangeData {
            self.regs.d.swap(rx as usize, ry as usize);
        } else if mode == Direction::ExchangeAddress {
            // TODO: change to std::mem::swap when new borrow checker is available
            let y = self.regs.a(ry);
            *self.regs.a_mut(ry) = self.regs.a(rx);
            *self.regs.a_mut(rx) = y;
        } else {
            let y = self.regs.a(ry);
            *self.regs.a_mut(ry) = self.regs.d[rx as usize];
            self.regs.d[rx as usize] = y;
        }

        Ok(CPU::EXG)
    }

    pub(super) fn execute_ext(&mut self, mode: u8, reg: u8) -> InterpreterResult {
        if mode == 0b010 {
            let d = self.regs.d[reg as usize] as i8 as u16;
            self.regs.d_word(reg, d);
        } else {
            self.regs.d[reg as usize] = self.regs.d[reg as usize] as i16 as u32;
        }

        self.regs.sr.n = self.regs.d[reg as usize] & SIGN_BIT_32 != 0;
        self.regs.sr.z = self.regs.d[reg as usize] == 0;
        self.regs.sr.v = false;
        self.regs.sr.c = false;

        Ok(CPU::EXT)
    }

    pub(super) fn execute_illegal(&self) -> InterpreterResult {
        Err(Vector::IllegalInstruction as u8)
    }

    pub(super) fn execute_jmp(&mut self, am: AddressingMode) -> InterpreterResult {
        let mut ea = EffectiveAddress::new(am, None);

        let mut exec_time = 0;
        self.regs.pc = self.get_effective_address(&mut ea, &mut exec_time);

        Ok(match am {
            AddressingMode::Ari(_) => CPU::JMP_ARI,
            AddressingMode::Ariwd(..) => CPU::JMP_ARIWD,
            AddressingMode::Ariwi8(..) => CPU::JMP_ARIWI8,
            AddressingMode::AbsShort(_) => CPU::JMP_ABSSHORT,
            AddressingMode::AbsLong(_) => CPU::JMP_ABSLONG,
            AddressingMode::Pciwd(..) => CPU::JMP_PCIWD,
            AddressingMode::Pciwi8(..) => CPU::JMP_PCIWI8,
            _ => panic!("Wrong addressing mode in JMP."),
        })
    }

    pub(super) fn execute_jsr(&mut self, memory: &mut impl MemoryAccess, am: AddressingMode) -> InterpreterResult {
        let mut ea = EffectiveAddress::new(am, None);

        let mut exec_time = 0;
        self.push_long(memory, self.regs.pc)?;
        self.regs.pc = self.get_effective_address(&mut ea, &mut exec_time);

        Ok(match am {
            AddressingMode::Ari(_) => CPU::JSR_ARI,
            AddressingMode::Ariwd(..) => CPU::JSR_ARIWD,
            AddressingMode::Ariwi8(..) => CPU::JSR_ARIWI8,
            AddressingMode::AbsShort(_) => CPU::JSR_ABSSHORT,
            AddressingMode::AbsLong(_) => CPU::JSR_ABSLONG,
            AddressingMode::Pciwd(..) => CPU::JSR_PCIWD,
            AddressingMode::Pciwi8(..) => CPU::JSR_PCIWI8,
            _ => panic!("Wrong addressing mode in JSR."),
        })
    }

    pub(super) fn execute_lea(&mut self, reg: u8, am: AddressingMode) -> InterpreterResult {
        let mut ea = EffectiveAddress::new(am, None);

        let mut exec_time = 0;
        *self.regs.a_mut(reg) = self.get_effective_address(&mut ea, &mut exec_time);

        Ok(match am {
            AddressingMode::Ari(_) => CPU::LEA_ARI,
            AddressingMode::Ariwd(..) => CPU::LEA_ARIWD,
            AddressingMode::Ariwi8(..) => CPU::LEA_ARIWI8,
            AddressingMode::AbsShort(_) => CPU::LEA_ABSSHORT,
            AddressingMode::AbsLong(_) => CPU::LEA_ABSLONG,
            AddressingMode::Pciwd(..) => CPU::LEA_PCIWD,
            AddressingMode::Pciwi8(..) => CPU::LEA_PCIWI8,
            _ => panic!("Wrong addressing mode in LEA."),
        })
    }

    pub(super) fn execute_link(&mut self, memory: &mut impl MemoryAccess, reg: u8, disp: i16) -> InterpreterResult {
        self.push_long(memory, self.regs.a(reg))?;
        *self.regs.a_mut(reg) = self.regs.sp();
        *self.regs.sp_mut() += disp as u32;

        Ok(CPU::LINK)
    }

    pub(super) fn execute_lsm(&mut self, memory: &mut impl MemoryAccess, dir: Direction, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = CPU::LSM;

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
                self.regs.d_byte(reg, data as u8);
                self.regs.sr.z = data & 0x0000_00FF == 0;
                CPU::LSR_BW + CPU::LSR_COUNT * shift_count as usize
            },
            Size::Word => {
                self.regs.d_word(reg, data as u16);
                self.regs.sr.z = data & 0x0000_FFFF == 0;
                CPU::LSR_BW + CPU::LSR_COUNT * shift_count as usize
            },
            Size::Long => {
                self.regs.d[reg as usize] = data;
                self.regs.sr.z = data == 0;
                CPU::LSR_L + CPU::LSR_COUNT * shift_count as usize
            },
        })
    }

    pub(super) fn execute_move(&mut self, memory: &mut impl MemoryAccess, size: Size, amdst: AddressingMode, amsrc: AddressingMode) -> InterpreterResult {
        let mut exec_time = if amdst.is_ariwpr() { CPU::MOVE_DST_ARIWPR } else { CPU::MOVE_OTHER };

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
        let mut exec_time = CPU::MOVEA;

        let mut ea = EffectiveAddress::new(am, Some(size));

        *self.regs.a_mut(reg) = if size.is_word() {
            self.get_word(memory, &mut ea, &mut exec_time)? as i16 as u32
        } else {
            self.get_long(memory, &mut ea, &mut exec_time)?
        };

        Ok(exec_time)
    }

    pub(super) fn execute_moveccr(&mut self, memory: &mut impl MemoryAccess, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = CPU::MOVECCR;

        let mut ea = EffectiveAddress::new(am, Some(Size::Word));

        let ccr = self.get_word(memory, &mut ea, &mut exec_time)?;
        self.regs.sr.set_ccr(ccr);

        Ok(exec_time)
    }

    pub(super) fn execute_movefsr(&mut self, memory: &mut impl MemoryAccess, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = if am.is_drd() { CPU::MOVEFSR_REG } else { CPU::MOVEFSR_MEM };

        let mut ea = EffectiveAddress::new(am, Some(Size::Word));

        self.set_word(memory, &mut ea, &mut exec_time, self.regs.sr.into())?;

        Ok(exec_time)
    }

    pub(super) fn execute_movesr(&mut self, memory: &mut impl MemoryAccess, am: AddressingMode) -> InterpreterResult {
        self.check_supervisor()?;

        let mut ea = EffectiveAddress::new(am, Some(Size::Word));
        let mut exec_time = CPU::MOVESR;

        let sr = self.get_word(memory, &mut ea, &mut exec_time)?;
        self.regs.sr = sr.into();
        Ok(exec_time)
    }

    pub(super) fn execute_moveusp(&mut self, dir: Direction, reg: u8) -> InterpreterResult {
        self.check_supervisor()?;

        if dir == Direction::UspToRegister {
            *self.regs.a_mut(reg) = self.regs.usp;
        } else {
            self.regs.usp = self.regs.a(reg);
        }
        Ok(CPU::MOVEUSP)
    }

    pub(super) fn execute_movem(&mut self, memory: &mut impl MemoryAccess, dir: Direction, size: Size, am: AddressingMode, mut list: u16) -> InterpreterResult {
        let count = list.count_ones() as usize;
        let mut exec_time = 0;

        let mut ea = EffectiveAddress::new(am, Some(size));

        let gap = size as u32;
        let eareg = ea.mode.register().unwrap_or(u8::MAX);

        if ea.mode.is_ariwpr() {
            let mut addr = self.regs.a(eareg);

            for reg in (0..8).rev() {
                if list & 1 != 0 {
                    addr -= gap;
                    if size.is_word() { memory.set_word(addr.even()?, self.regs.a(reg) as u16).ok_or(ACCESS_ERROR)?; }
                        else { memory.set_long(addr.even()?, self.regs.a(reg)).ok_or(ACCESS_ERROR)?; }
                }

                list >>= 1;
            }

            for reg in (0..8).rev() {
                if list & 1 != 0 {
                    addr -= gap;
                    if size.is_word() { memory.set_word(addr.even()?, self.regs.d[reg] as u16).ok_or(ACCESS_ERROR)?; }
                        else { memory.set_long(addr.even()?, self.regs.d[reg]).ok_or(ACCESS_ERROR)?; }
                }

                list >>= 1;
            }

            *self.regs.a_mut(eareg) = addr;
        } else {
            let mut addr = if ea.mode.is_ariwpo() {
                self.regs.a(eareg)
            } else {
                self.get_effective_address(&mut ea, &mut exec_time)
            };

            for reg in 0..8 {
                if list & 1 != 0 {
                    if dir == Direction::MemoryToRegister {
                        let value = if size.is_word() { memory.get_word(addr.even()?).ok_or(ACCESS_ERROR)? as i16 as u32 }
                            else { memory.get_long(addr.even()?).ok_or(ACCESS_ERROR)? };
                        self.regs.d[reg] = value;
                    } else {
                        if size.is_word() { memory.set_word(addr.even()?, self.regs.d[reg] as u16).ok_or(ACCESS_ERROR)?; }
                            else { memory.set_long(addr.even()?, self.regs.d[reg]).ok_or(ACCESS_ERROR)?; }
                    }

                    addr += gap;
                }

                list >>= 1;
            }

            for reg in 0..8 {
                if list & 1 != 0 {
                    if dir == Direction::MemoryToRegister {
                        let value = if size.is_word() { memory.get_word(addr.even()?).ok_or(ACCESS_ERROR)? as i16 as u32 }
                            else { memory.get_long(addr.even()?).ok_or(ACCESS_ERROR)? };
                        *self.regs.a_mut(reg) = value;
                    } else {
                        if size.is_word() { memory.set_word(addr.even()?, self.regs.a(reg as u8) as u16).ok_or(ACCESS_ERROR)?; }
                            else { memory.set_long(addr.even()?, self.regs.a(reg as u8)).ok_or(ACCESS_ERROR)?; }
                    }

                    addr += gap;
                }

                list >>= 1;
            }

            if ea.mode.is_ariwpo() {
                *self.regs.a_mut(eareg) = addr;
            }
        }

        exec_time = match am {
            AddressingMode::Ari(_) => CPU::MOVEM_ARI,
            AddressingMode::Ariwpo(_) => CPU::MOVEM_ARIWPO,
            AddressingMode::Ariwpr(_) => CPU::MOVEM_ARIWPR,
            AddressingMode::Ariwd(..) => CPU::MOVEM_ARIWD,
            AddressingMode::Ariwi8(..) => CPU::MOVEM_ARIWI8,
            AddressingMode::AbsShort(_) => CPU::MOVEM_ABSSHORT,
            AddressingMode::AbsLong(_) => CPU::MOVEM_ABSLONG,
            AddressingMode::Pciwd(..) => CPU::MOVEM_PCIWD,
            AddressingMode::Pciwi8(..) => CPU::MOVEM_PCIWI8,
            _ => panic!("Wrong addressing mode for MOVEM."),
        };
        if dir == Direction::MemoryToRegister {
            exec_time += CPU::MOVEM_MTR;
        }
        Ok(exec_time + count * if size.is_long() { CPU::MOVEM_LONG } else { CPU::MOVEM_WORD })
    }

    pub(super) fn execute_movep(&mut self, memory: &mut impl MemoryAccess, data: u8, dir: Direction, size: Size, addr: u8, disp: i16) -> InterpreterResult {
        let mut shift = if size.is_word() { 8 } else { 24 };
        let mut addr = self.regs.a(addr) + disp as u32;

        if dir == Direction::RegisterToMemory {
            while shift >= 0 {
                let d = (self.regs.d[data as usize] >> shift) as u8;
                memory.set_byte(addr, d).ok_or(ACCESS_ERROR)?;
                shift -= 8;
                addr += 2;
            }

            Ok(if size.is_long() {
                CPU::MOVEP_RTM_LONG
            } else {
                CPU::MOVEP_RTM_WORD
            })
        } else {
            if size.is_word() { self.regs.d[data as usize] &= 0xFFFF_0000 } else { self.regs.d[data as usize] = 0 }

            while shift >= 0 {
                let d = memory.get_byte(addr).ok_or(ACCESS_ERROR)? as u32;
                self.regs.d[data as usize] |= d << shift;
                shift -= 8;
                addr += 2;
            }

            Ok(if size.is_long() {
                CPU::MOVEP_MTR_LONG
            } else {
                CPU::MOVEP_MTR_WORD
            })
        }
    }

    pub(super) fn execute_moveq(&mut self, reg: u8, data: i8) -> InterpreterResult {
        self.regs.d[reg as usize] = data as u32;

        self.regs.sr.n = data <  0;
        self.regs.sr.z = data == 0;
        self.regs.sr.v = false;
        self.regs.sr.c = false;

        Ok(CPU::MOVEQ)
    }

    pub(super) fn execute_muls(&mut self, memory: &mut impl MemoryAccess, reg: u8, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = CPU::MULS;

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
        let mut exec_time = CPU::MULU;

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
        let mut exec_time = if am.is_drd() { CPU::NBCD_REG } else { CPU::NBCD_MEM };

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
        let mut exec_time = single_operands_time(size.is_long(), am.is_drd(), CPU::NEG_REG_BW, CPU::NEG_REG_L, CPU::NEG_MEM_BW, CPU::NEG_MEM_L);

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
        let mut exec_time = single_operands_time(size.is_long(), am.is_drd(), CPU::NEGX_REG_BW, CPU::NEGX_REG_L, CPU::NEGX_MEM_BW, CPU::NEGX_MEM_L);

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte => {
                let data = self.get_byte(memory, &mut ea, &mut exec_time)?;

                let (res, v) = 0i8.borrowing_sub(data as i8, self.regs.sr.x);
                let (_, c) = 0u8.borrowing_sub(data, self.regs.sr.x);
                self.set_byte(memory, &mut ea, &mut exec_time, res as u8)?;

                self.regs.sr.x = c;
                self.regs.sr.n = res < 0;
                if res != 0 { self.regs.sr.z = false }
                self.regs.sr.v = v;
                self.regs.sr.c = c;
            },
            Size::Word => {
                let data = self.get_word(memory, &mut ea, &mut exec_time)?;

                let (res, v) = 0i16.borrowing_sub(data as i16, self.regs.sr.x);
                let (_, c) = 0u16.borrowing_sub(data, self.regs.sr.x);
                self.set_word(memory, &mut ea, &mut exec_time, res as u16)?;

                self.regs.sr.x = c;
                self.regs.sr.n = res < 0;
                if res != 0 { self.regs.sr.z = false }
                self.regs.sr.v = v;
                self.regs.sr.c = c;
            },
            Size::Long => {
                let data = self.get_long(memory, &mut ea, &mut exec_time)?;

                let (res, v) = 0i32.borrowing_sub(data as i32, self.regs.sr.x);
                let (_, c) = 0u32.borrowing_sub(data, self.regs.sr.x);
                self.set_long(memory, &mut ea, &mut exec_time, res as u32)?;

                self.regs.sr.x = c;
                self.regs.sr.n = res < 0;
                if res != 0 { self.regs.sr.z = false }
                self.regs.sr.v = v;
                self.regs.sr.c = c;
            },
        }

        Ok(exec_time)
    }

    pub(super) fn execute_nop(&self) -> InterpreterResult {
        Ok(CPU::NOP)
    }

    pub(super) fn execute_not(&mut self, memory: &mut impl MemoryAccess, size: Size, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = single_operands_time(size.is_long(), am.is_drd(), CPU::NOT_REG_BW, CPU::NOT_REG_L, CPU::NOT_MEM_BW, CPU::NOT_MEM_L);

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
                    exec_time = CPU::OR_MEM_BW;
                } else {
                    exec_time = CPU::OR_REG_BW;
                }
                let src = self.regs.d[reg as usize] as u8;
                let dst = self.get_byte(memory, &mut ea, &mut exec_time)?;

                let res = src | dst;

                self.regs.sr.n = res & SIGN_BIT_8 != 0;
                self.regs.sr.z = res == 0;

                if dir == Direction::DstEa {
                    self.set_byte(memory, &mut ea, &mut exec_time, res)?;
                } else {
                    self.regs.d_byte(reg, res);
                }
            },
            Size::Word => {
                if dir == Direction::DstEa {
                    exec_time = CPU::OR_MEM_BW;
                } else {
                    exec_time = CPU::OR_REG_BW;
                }
                let src = self.regs.d[reg as usize] as u16;
                let dst = self.get_word(memory, &mut ea, &mut exec_time)?;

                let res = src | dst;

                self.regs.sr.n = res & SIGN_BIT_16 != 0;
                self.regs.sr.z = res == 0;

                if dir == Direction::DstEa {
                    self.set_word(memory, &mut ea, &mut exec_time, res)?;
                } else {
                    self.regs.d_word(reg, res);
                }
            },
            Size::Long => {
                if dir == Direction::DstEa {
                    exec_time = CPU::OR_MEM_L;
                } else {
                    exec_time = if am.is_dard() || am.is_immediate() { CPU::OR_REG_L_RDIMM } else { CPU::OR_REG_L };
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
                exec_time = if am.is_drd() { CPU::ORI_REG_BW } else { CPU::ORI_MEM_BW };
                let data = self.get_byte(memory, &mut ea, &mut exec_time)? | imm as u8;
                self.set_byte(memory, &mut ea, &mut exec_time, data)?;

                self.regs.sr.n = data & SIGN_BIT_8 != 0;
                self.regs.sr.z = data == 0;
            },
            Size::Word => {
                exec_time = if am.is_drd() { CPU::ORI_REG_BW } else { CPU::ORI_MEM_BW };
                let data = self.get_word(memory, &mut ea, &mut exec_time)? | imm as u16;
                self.set_word(memory, &mut ea, &mut exec_time, data)?;

                self.regs.sr.n = data & SIGN_BIT_16 != 0;
                self.regs.sr.z = data == 0;
            },
            Size::Long => {
                exec_time = if am.is_drd() { CPU::ORI_REG_L } else { CPU::ORI_MEM_L };
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

        Ok(CPU::ORICCR)
    }

    pub(super) fn execute_orisr(&mut self, imm: u16) -> InterpreterResult {
        self.check_supervisor()?;

        self.regs.sr |= imm;
        Ok(CPU::ORISR)
    }

    pub(super) fn execute_pea(&mut self, memory: &mut impl MemoryAccess, am: AddressingMode) -> InterpreterResult {
        let mut ea = EffectiveAddress::new(am, None);

        let mut exec_time = 0;
        let addr = self.get_effective_address(&mut ea, &mut exec_time);
        self.push_long(memory, addr)?;

        Ok(match am {
            AddressingMode::Ari(_) => CPU::PEA_ARI,
            AddressingMode::Ariwd(..) => CPU::PEA_ARIWD,
            AddressingMode::Ariwi8(..) => CPU::PEA_ARIWI8,
            AddressingMode::AbsShort(_) => CPU::PEA_ABSSHORT,
            AddressingMode::AbsLong(_) => CPU::PEA_ABSLONG,
            AddressingMode::Pciwd(..) => CPU::PEA_PCIWD,
            AddressingMode::Pciwi8(..) => CPU::PEA_PCIWI8,
            _ => panic!("Wrong addressing mode in PEA."),
        })
    }

    pub(super) fn execute_reset(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        self.check_supervisor()?;

        memory.reset_instruction();
        Ok(CPU::RESET)
    }

    pub(super) fn execute_rom(&mut self, memory: &mut impl MemoryAccess, dir: Direction, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = CPU::ROM;

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
                self.regs.d_byte(reg, data as u8);
                self.regs.sr.z = data & 0x0000_00FF == 0;
                CPU::ROR_BW + CPU::ROR_COUNT * shift_count as usize
            },
            Size::Word => {
                self.regs.d_word(reg, data as u16);
                self.regs.sr.z = data & 0x0000_FFFF == 0;
                CPU::ROR_BW + CPU::ROR_COUNT * shift_count as usize
            },
            Size::Long => {
                self.regs.d[reg as usize] = data;
                self.regs.sr.z = data == 0;
                CPU::ROR_L + CPU::ROR_COUNT * shift_count as usize
            },
        })
    }

    pub(super) fn execute_roxm(&mut self, memory: &mut impl MemoryAccess, dir: Direction, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = CPU::ROXM;

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
                self.regs.d_byte(reg, data as u8);
                self.regs.sr.z = data & 0x0000_00FF == 0;
                CPU::ROXR_BW + CPU::ROXR_COUNT * shift_count as usize
            },
            Size::Word => {
                self.regs.d_word(reg, data as u16);
                self.regs.sr.z = data & 0x0000_FFFF == 0;
                CPU::ROXR_BW + CPU::ROXR_COUNT * shift_count as usize
            },
            Size::Long => {
                self.regs.d[reg as usize] = data;
                self.regs.sr.z = data == 0;
                CPU::ROXR_L + CPU::ROXR_COUNT * shift_count as usize
            },
        })
    }

    pub(super) fn execute_rte(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        self.check_supervisor()?;

        let sr = self.pop_word(memory)?;
        self.regs.pc = self.pop_long(memory)?;
        let mut exec_time = CPU::RTE;

        if CPU::STACK_FORMAT == StackFormat::SCC68070 {
            let format = self.pop_word(memory)?;

            if format & 0xF000 == 0xF000 { // Long format
                *self.regs.sp_mut() += 26;
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

        Ok(CPU::RTR)
    }

    pub(super) fn execute_rts(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        self.regs.pc = self.pop_long(memory)?;

        Ok(CPU::RTS)
    }

    pub(super) fn execute_sbcd(&mut self, memory: &mut impl MemoryAccess, ry: u8, mode: Direction, rx: u8) -> InterpreterResult {
        let (src, dst) = if mode == Direction::MemoryToMemory {
            let src_addr = self.ariwpr(rx, Size::Byte);
            let dst_addr = self.ariwpr(ry, Size::Byte);
            (memory.get_byte(src_addr).ok_or(ACCESS_ERROR)?, memory.get_byte(dst_addr).ok_or(ACCESS_ERROR)?)
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
            memory.set_byte(self.regs.a(ry), res).ok_or(ACCESS_ERROR)?;
            Ok(CPU::SBCD_MEM)
        } else {
            self.regs.d_byte(ry, res);
            Ok(CPU::SBCD_REG)
        }
    }

    pub(super) fn execute_scc(&mut self, memory: &mut impl MemoryAccess, cc: u8, am: AddressingMode) -> InterpreterResult {
        let condition = self.regs.sr.condition(cc);
        let mut exec_time = single_operands_time(condition, am.is_drd(), CPU::SCC_REG_FALSE, CPU::SCC_REG_TRUE, CPU::SCC_MEM_FALSE, CPU::SCC_MEM_TRUE);

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
        Ok(CPU::STOP)
    }

    pub(super) fn execute_sub(&mut self, memory: &mut impl MemoryAccess, reg: u8, dir: Direction, size: Size, am: AddressingMode) -> InterpreterResult {
        let mut exec_time;

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte => {
                let (src, dst) = if dir == Direction::DstEa {
                    exec_time = CPU::SUB_MEM_BW;
                    (self.regs.d[reg as usize] as u8, self.get_byte(memory, &mut ea, &mut exec_time)?)
                } else {
                    exec_time = CPU::SUB_REG_BW;
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
                    self.regs.d_byte(reg, res as u8);
                }
            },
            Size::Word => {
                let (src, dst) = if dir == Direction::DstEa {
                    exec_time = CPU::SUB_MEM_BW;
                    (self.regs.d[reg as usize] as u16, self.get_word(memory, &mut ea, &mut exec_time)?)
                } else {
                    exec_time = CPU::SUB_REG_BW;
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
                    self.regs.d_word(reg, res as u16);
                }
            },
            Size::Long => {
                let (src, dst) = if dir == Direction::DstEa {
                    exec_time = CPU::SUB_MEM_L;
                    (self.regs.d[reg as usize] as u32, self.get_long(memory, &mut ea, &mut exec_time)?)
                } else {
                    exec_time = if am.is_dard() || am.is_immediate() { CPU::SUB_REG_L_RDIMM } else { CPU::SUB_REG_L };
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
            exec_time = CPU::SUBA_WORD;
            self.get_word(memory, &mut ea, &mut exec_time)? as i16 as u32
        } else {
            exec_time = if am.is_dard() || am.is_immediate() {
                CPU::SUBA_LONG_RDIMM
            } else {
                CPU::SUBA_LONG
            };
            self.get_long(memory, &mut ea, &mut exec_time)?
        };

        *self.regs.a_mut(reg) -= src;

        Ok(exec_time)
    }

    pub(super) fn execute_subi(&mut self, memory: &mut impl MemoryAccess, size: Size, am: AddressingMode, imm: u32) -> InterpreterResult {
        let mut exec_time;

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte => {
                exec_time = if am.is_drd() { CPU::SUBI_REG_BW } else { CPU::SUBI_MEM_BW };
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
                exec_time = if am.is_drd() { CPU::SUBI_REG_BW } else { CPU::SUBI_MEM_BW };
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
                exec_time = if am.is_drd() { CPU::SUBI_REG_L } else { CPU::SUBI_MEM_L };
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
        if am.is_ard() {
            *self.regs.a_mut(am.register().unwrap()) -= imm as u32;
            return Ok(if size.is_long() { CPU::SUBQ_REG_L } else { CPU::SUBQ_AREG_BW });
        }

        let mut exec_time;

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte => {
                exec_time = if am.is_drd() { CPU::SUBQ_DREG_BW } else { CPU::SUBQ_MEM_BW };
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
                exec_time = if am.is_drd() { CPU::SUBQ_DREG_BW } else { CPU::SUBQ_MEM_BW };
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
                exec_time = if am.is_drd() { CPU::SUBQ_REG_L } else { CPU::SUBQ_MEM_L };
                let data = self.get_long(memory, &mut ea, &mut exec_time)?;
                let (res, v) = (data as i32).overflowing_sub(imm as i32);
                let (_, c) = data.overflowing_sub(imm as u32);
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

    pub(super) fn execute_subx(&mut self, memory: &mut impl MemoryAccess, ry: u8, size: Size, mode: Direction, rx: u8) -> InterpreterResult {
        match size {
            Size::Byte => {
                let (src, dst) = if mode == Direction::MemoryToMemory {
                    let src_addr = self.ariwpr(rx, size);
                    let dst_addr = self.ariwpr(ry, size);
                    (memory.get_byte(src_addr).ok_or(ACCESS_ERROR)?, memory.get_byte(dst_addr).ok_or(ACCESS_ERROR)?)
                } else {
                    (self.regs.d[rx as usize] as u8, self.regs.d[ry as usize] as u8)
                };

                let (res, v) = (dst as i8).borrowing_sub(src as i8, self.regs.sr.x);
                let (_, c) = dst.borrowing_sub(src, self.regs.sr.x);

                self.regs.sr.n = res < 0;
                if res != 0 {
                    self.regs.sr.z = false;
                }
                self.regs.sr.v = v;
                self.regs.sr.c = c;
                self.regs.sr.x = c;

                if mode == Direction::MemoryToMemory {
                    memory.set_byte(self.regs.a(ry), res as u8).ok_or(ACCESS_ERROR)?;
                    Ok(CPU::SUBX_MEM_BW)
                } else {
                    self.regs.d_byte(ry, res as u8);
                    Ok(CPU::SUBX_REG_BW)
                }
            },
            Size::Word => {
                let (src, dst) = if mode == Direction::MemoryToMemory {
                    let src_addr = self.ariwpr(rx, size);
                    let dst_addr = self.ariwpr(ry, size);
                    (memory.get_word(src_addr.even()?).ok_or(ACCESS_ERROR)?, memory.get_word(dst_addr.even()?).ok_or(ACCESS_ERROR)?)
                } else {
                    (self.regs.d[rx as usize] as u16, self.regs.d[ry as usize] as u16)
                };

                let (res, v) = (dst as i16).borrowing_sub(src as i16, self.regs.sr.x);
                let (_, c) = dst.borrowing_sub(src, self.regs.sr.x);

                self.regs.sr.n = res < 0;
                if res != 0 {
                    self.regs.sr.z = false;
                }
                self.regs.sr.v = v;
                self.regs.sr.c = c;
                self.regs.sr.x = c;

                if mode == Direction::MemoryToMemory {
                    memory.set_word(self.regs.a(ry), res as u16).ok_or(ACCESS_ERROR)?;
                    Ok(CPU::SUBX_MEM_BW)
                } else {
                    self.regs.d_word(ry, res as u16);
                    Ok(CPU::SUBX_REG_BW)
                }
            },
            Size::Long => {
                let (src, dst) = if mode == Direction::MemoryToMemory {
                    let src_addr = self.ariwpr(rx, size);
                    let dst_addr = self.ariwpr(ry, size);
                    (memory.get_long(src_addr.even()?).ok_or(ACCESS_ERROR)?, memory.get_long(dst_addr.even()?).ok_or(ACCESS_ERROR)?)
                } else {
                    (self.regs.d[rx as usize], self.regs.d[ry as usize])
                };

                let (res, v) = (dst as i32).borrowing_sub(src as i32, self.regs.sr.x);
                let (_, c) = dst.borrowing_sub(src, self.regs.sr.x);

                self.regs.sr.n = res < 0;
                if res != 0 {
                    self.regs.sr.z = false;
                }
                self.regs.sr.v = v;
                self.regs.sr.c = c;
                self.regs.sr.x = c;

                if mode == Direction::MemoryToMemory {
                    memory.set_long(self.regs.a(ry), res as u32).ok_or(ACCESS_ERROR)?;
                    Ok(CPU::SUBX_MEM_L)
                } else {
                    self.regs.d[ry as usize] = res as u32;
                    Ok(CPU::SUBX_REG_L)
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

        Ok(CPU::SWAP)
    }

    pub(super) fn execute_tas(&mut self, memory: &mut impl MemoryAccess, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = if am.is_drd() { CPU::TAS_REG } else { CPU::TAS_MEM };

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
            Ok(CPU::TRAPV_NO_TRAP)
        }
    }

    pub(super) fn execute_tst(&mut self, memory: &mut impl MemoryAccess, size: Size, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = single_operands_time(size.is_long(), am.is_drd(), CPU::TST_REG_BW, CPU::TST_REG_L, CPU::TST_MEM_BW, CPU::TST_MEM_L);

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
        *self.regs.sp_mut() = self.regs.a(reg);
        *self.regs.a_mut(reg) = self.pop_long(memory)?;

        Ok(CPU::UNLK)
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
