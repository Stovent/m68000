// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{CpuDetails, M68000, MemoryAccess, StackFormat};
use crate::addressing_modes::{EffectiveAddress, AddressingMode};
use crate::exception::{ACCESS_ERROR, Vector};
use crate::instruction::{Direction, Size};
use crate::utils::{bits, CarryingOps, Integer, IsEven};

use std::num::Wrapping;

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

    pub(super) fn execute_abcd<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, rx: u8, mode: Direction, ry: u8) -> InterpreterResult {
        let (src, dst) = if mode == Direction::MemoryToMemory {
            let src_addr = self.ariwpr(ry, Size::Byte);
            let dst_addr = self.ariwpr(rx, Size::Byte);
            (memory.get_byte(src_addr).ok_or(ACCESS_ERROR)? as u16, memory.get_byte(dst_addr).ok_or(ACCESS_ERROR)? as u16)
        } else {
            (self.regs.d[ry as usize].0 as u8 as u16, self.regs.d[rx as usize].0 as u8 as u16)
        };
        let src = src + self.regs.sr.x as u16;
        let bin_res = src + dst;

        let mut res = (src & 0x0F) + (dst & 0x0F);
        if res >= 0x0A {
            res += 0x06;
        }

        res += (src & 0xF0) + (dst & 0xF0);
        if res >= 0xA0 {
            res += 0x60;
        }

        self.regs.sr.n = res & 0x80 != 0;
        if res != 0 { self.regs.sr.z = false; }
        self.regs.sr.v = src > (0x79 - dst) && bin_res < 0x80;
        self.regs.sr.c = res >= 0x0100;
        self.regs.sr.x = self.regs.sr.c;

        if mode == Direction::MemoryToMemory {
            memory.set_byte(self.regs.a(rx), res as u8).ok_or(ACCESS_ERROR)?;
            Ok(CPU::ABCD_MEM)
        } else {
            self.regs.d_byte(rx, res as u8);
            Ok(CPU::ABCD_REG)
        }
    }

    fn add<UT, ST, const ADDX: bool>(&mut self, dst: UT, src: UT) -> UT
    where
        UT: CarryingOps<ST, UT>,
        ST: Integer,
    {
        let (res, v) = src.signed_carrying_add(dst, ADDX && self.regs.sr.x);
        let (ures, c) = src.unsigned_carrying_add(dst, ADDX && self.regs.sr.x);

        self.regs.sr.x = c;
        self.regs.sr.n = res < ST::ZERO;
        if ADDX {
            if res != ST::ZERO {
                self.regs.sr.z = false;
            }
        } else {
            self.regs.sr.z = res == ST::ZERO;
        }
        self.regs.sr.v = v;
        self.regs.sr.c = c;

        ures
    }

    pub(super) fn execute_add<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, reg: u8, dir: Direction, size: Size, am: AddressingMode) -> InterpreterResult {
        let mut exec_time;

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte => {
                let (src, dst) = if dir == Direction::DstEa {
                    exec_time = CPU::ADD_MEM_BW;
                    (self.regs.d[reg as usize].0 as u8, self.get_byte(memory, &mut ea, &mut exec_time)?)
                } else {
                    exec_time = CPU::ADD_REG_BW;
                    (self.get_byte(memory, &mut ea, &mut exec_time)?, self.regs.d[reg as usize].0 as u8)
                };

                let res = self.add::<u8, i8, false>(dst, src);

                if dir == Direction::DstEa {
                    self.set_byte(memory, &mut ea, &mut exec_time, res)?;
                } else {
                    self.regs.d_byte(reg, res);
                }
            },
            Size::Word => {
                let (src, dst) = if dir == Direction::DstEa {
                    exec_time = CPU::ADD_MEM_BW;
                    (self.regs.d[reg as usize].0 as u16, self.get_word(memory, &mut ea, &mut exec_time)?)
                } else {
                    exec_time = CPU::ADD_REG_BW;
                    (self.get_word(memory, &mut ea, &mut exec_time)?, self.regs.d[reg as usize].0 as u16)
                };

                let res = self.add::<u16, i16, false>(dst, src);

                if dir == Direction::DstEa {
                    self.set_word(memory, &mut ea, &mut exec_time, res)?;
                } else {
                    self.regs.d_word(reg, res);
                }
            },
            Size::Long => {
                let (src, dst) = if dir == Direction::DstEa {
                    exec_time = CPU::ADD_MEM_L;
                    (self.regs.d[reg as usize].0, self.get_long(memory, &mut ea, &mut exec_time)?)
                } else {
                    exec_time = if am.is_dard() || am.is_immediate() { CPU::ADD_REG_L_RDIMM } else { CPU::ADD_REG_L };
                    (self.get_long(memory, &mut ea, &mut exec_time)?, self.regs.d[reg as usize].0)
                };

                let res = self.add::<u32, i32, false>(dst, src);

                if dir == Direction::DstEa {
                    self.set_long(memory, &mut ea, &mut exec_time, res)?;
                } else {
                    self.regs.d[reg as usize].0 = res;
                }
            },
        }

        Ok(exec_time)
    }

    pub(super) fn execute_adda<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, reg: u8, size: Size, am: AddressingMode) -> InterpreterResult {
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

    pub(super) fn execute_addi<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, size: Size, am: AddressingMode, imm: u32) -> InterpreterResult {
        let mut exec_time;

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte =>  {
                exec_time = if am.is_drd() { CPU::ADDI_REG_BW } else { CPU::ADDI_MEM_BW };
                let data = self.get_byte(memory, &mut ea, &mut exec_time)?;

                let res = self.add::<u8, i8, false>(data, imm as u8);

                self.set_byte(memory, &mut ea, &mut exec_time, res)?;
            },
            Size::Word => {
                exec_time = if am.is_drd() { CPU::ADDI_REG_BW } else { CPU::ADDI_MEM_BW };
                let data = self.get_word(memory, &mut ea, &mut exec_time)?;

                let res = self.add::<u16, i16, false>(data, imm as u16);

                self.set_word(memory, &mut ea, &mut exec_time, res)?;
            },
            Size::Long => {
                exec_time = if am.is_drd() { CPU::ADDI_REG_L } else { CPU::ADDI_MEM_L };
                let data = self.get_long(memory, &mut ea, &mut exec_time)?;

                let res = self.add::<u32, i32, false>(data, imm);

                self.set_long(memory, &mut ea, &mut exec_time, res)?;
            },
        }

        Ok(exec_time)
    }

    pub(super) fn execute_addq<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, imm: u8, size: Size, am: AddressingMode) -> InterpreterResult {
        let imm = if imm == 0 { 8 } else { imm };

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

                let res = self.add::<u8, i8, false>(data, imm);

                self.set_byte(memory, &mut ea, &mut exec_time, res)?;
            },
            Size::Word => {
                exec_time = if am.is_drd() { CPU::ADDQ_REG_BW } else { CPU::ADDQ_MEM_BW };
                let data = self.get_word(memory, &mut ea, &mut exec_time)?;

                let res = self.add::<u16, i16, false>(data, imm as u16);

                self.set_word(memory, &mut ea, &mut exec_time, res)?;
            },
            Size::Long => {
                exec_time = if am.is_drd() { CPU::ADDQ_REG_L } else { CPU::ADDQ_MEM_L };
                let data = self.get_long(memory, &mut ea, &mut exec_time)?;

                let res = self.add::<u32, i32, false>(data, imm as u32);

                self.set_long(memory, &mut ea, &mut exec_time, res)?;
            },
        }

        Ok(exec_time)
    }

    pub(super) fn execute_addx<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, rx: u8, size: Size, mode: Direction, ry: u8) -> InterpreterResult {
        match size {
            Size::Byte => {
                let (src, dst) = if mode == Direction::MemoryToMemory {
                    let src_addr = self.ariwpr(ry, size);
                    let dst_addr = self.ariwpr(rx, size);
                    (memory.get_byte(src_addr).ok_or(ACCESS_ERROR)?, memory.get_byte(dst_addr).ok_or(ACCESS_ERROR)?)
                } else {
                    (self.regs.d[ry as usize].0 as u8, self.regs.d[rx as usize].0 as u8)
                };

                let res = self.add::<u8, i8, true>(dst, src);

                if mode == Direction::MemoryToMemory {
                    memory.set_byte(self.regs.a(rx), res).ok_or(ACCESS_ERROR)?;
                    Ok(CPU::ADDX_MEM_BW)
                } else {
                    self.regs.d_byte(rx, res);
                    Ok(CPU::ADDX_REG_BW)
                }
            },
            Size::Word => {
                let (src, dst) = if mode == Direction::MemoryToMemory {
                    let src_addr = self.ariwpr(ry, size);
                    let dst_addr = self.ariwpr(rx, size);
                    (memory.get_word(src_addr.even()?).ok_or(ACCESS_ERROR)?, memory.get_word(dst_addr.even()?).ok_or(ACCESS_ERROR)?)
                } else {
                    (self.regs.d[ry as usize].0 as u16, self.regs.d[rx as usize].0 as u16)
                };

                let res = self.add::<u16, i16, true>(dst, src);

                if mode == Direction::MemoryToMemory {
                    memory.set_word(self.regs.a(rx), res).ok_or(ACCESS_ERROR)?;
                    Ok(CPU::ADDX_MEM_BW)
                } else {
                    self.regs.d_word(rx, res);
                    Ok(CPU::ADDX_REG_BW)
                }
            },
            Size::Long => {
                let (src, dst) = if mode == Direction::MemoryToMemory {
                    let src_addr = self.ariwpr(ry, size);
                    let dst_addr = self.ariwpr(rx, size);
                    (memory.get_long(src_addr.even()?).ok_or(ACCESS_ERROR)?, memory.get_long(dst_addr.even()?).ok_or(ACCESS_ERROR)?)
                } else {
                    (self.regs.d[ry as usize].0, self.regs.d[rx as usize].0)
                };

                let res = self.add::<u32, i32, true>(dst, src);

                if mode == Direction::MemoryToMemory {
                    memory.set_long(self.regs.a(rx), res).ok_or(ACCESS_ERROR)?;
                    Ok(CPU::ADDX_MEM_L)
                } else {
                    self.regs.d[rx as usize].0 = res;
                    Ok(CPU::ADDX_REG_L)
                }
            },
        }
    }

    fn and<UT>(&mut self, dst: UT, src: UT) -> UT
    where
        UT: Integer,
    {
        let res = src & dst;

        self.regs.sr.n = res & UT::SIGN_BIT_MASK != UT::ZERO;
        self.regs.sr.z = res == UT::ZERO;
        self.regs.sr.v = false;
        self.regs.sr.c = false;

        res
    }

    pub(super) fn execute_and<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, reg: u8, dir: Direction, size: Size, am: AddressingMode) -> InterpreterResult {
        let mut exec_time;

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte => {
                if dir == Direction::DstEa {
                    exec_time = CPU::AND_MEM_BW;
                } else {
                    exec_time = CPU::AND_REG_BW;
                }
                let src = self.regs.d[reg as usize].0 as u8;
                let dst = self.get_byte(memory, &mut ea, &mut exec_time)?;

                let res = self.and(dst, src);

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
                let src = self.regs.d[reg as usize].0 as u16;
                let dst = self.get_word(memory, &mut ea, &mut exec_time)?;

                let res = self.and(dst, src);

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
                let src = self.regs.d[reg as usize].0;
                let dst = self.get_long(memory, &mut ea, &mut exec_time)?;

                let res = self.and(dst, src);

                if dir == Direction::DstEa {
                    self.set_long(memory, &mut ea, &mut exec_time, res)?;
                } else {
                    self.regs.d[reg as usize].0 = res;
                }
            },
        }

        Ok(exec_time)
    }

    pub(super) fn execute_andi<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, size: Size, am: AddressingMode, imm: u32) -> InterpreterResult {
        let mut exec_time;

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte => {
                exec_time = if am.is_drd() { CPU::ANDI_REG_BW } else { CPU::ANDI_MEM_BW };
                let data = self.get_byte(memory, &mut ea, &mut exec_time)?;

                let res = self.and(data, imm as u8);

                self.set_byte(memory, &mut ea, &mut exec_time, res)?;
            },
            Size::Word => {
                exec_time = if am.is_drd() { CPU::ANDI_REG_BW } else { CPU::ANDI_MEM_BW };
                let data = self.get_word(memory, &mut ea, &mut exec_time)?;

                let res = self.and(data, imm as u16);

                self.set_word(memory, &mut ea, &mut exec_time, res)?;
            },
            Size::Long => {
                exec_time = if am.is_drd() { CPU::ANDI_REG_L } else { CPU::ANDI_MEM_L };
                let data = self.get_long(memory, &mut ea, &mut exec_time)?;

                let res = self.and(data, imm);

                self.set_long(memory, &mut ea, &mut exec_time, res)?;
            },
        }

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

    pub(super) fn execute_asm<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, dir: Direction, am: AddressingMode) -> InterpreterResult {
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
            (self.regs.d[rot as usize].0 % 64) as u8
        } else {
            if rot == 0 { 8 } else { rot }
        };

        let (mut data, mask) = match size {
            Size::Byte => (self.regs.d[reg as usize].0 & 0x0000_00FF, SIGN_BIT_8 as u32),
            Size::Word => (self.regs.d[reg as usize].0 & 0x0000_FFFF, SIGN_BIT_16 as u32),
            Size::Long => (self.regs.d[reg as usize].0, SIGN_BIT_32),
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
                self.regs.d[reg as usize].0 = data;
                self.regs.sr.z = data == 0;
                CPU::ASR_L + CPU::ASR_COUNT * shift_count as usize
            }
        })
    }

    pub(super) fn execute_bcc(&mut self, pc: u32, condition: u8, displacement: i16) -> InterpreterResult {
        if self.regs.sr.condition(condition) {
            self.regs.pc.0 = pc.wrapping_add(displacement as u32);
            Ok(CPU::BCC_BRANCH)
        } else {
            Ok(if self.current_opcode as u8 == 0 {
                CPU::BCC_NO_BRANCH_WORD
            } else {
                CPU::BCC_NO_BRANCH_BYTE
            })
        }
    }

    pub(super) fn execute_bchg<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, am: AddressingMode, mut count: u8) -> InterpreterResult {
        let mut exec_time = if bits(self.current_opcode, 8, 8) != 0 {
            count = self.regs.d[count as usize].0 as u8;
            if am.is_drd() { CPU::BCHG_DYN_REG } else { CPU::BCHG_DYN_MEM }
        } else {
            if am.is_drd() { CPU::BCHG_STA_REG } else { CPU::BCHG_STA_MEM }
        };

        if am.is_drd() {
            count %= 32;
            let reg = am.register().unwrap() as usize;
            self.regs.sr.z = self.regs.d[reg].0 & 1 << count == 0;
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

    pub(super) fn execute_bclr<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, am: AddressingMode, mut count: u8) -> InterpreterResult {
        let mut exec_time = if bits(self.current_opcode, 8, 8) != 0 {
            count = self.regs.d[count as usize].0 as u8;
            if am.is_drd() { CPU::BCLR_DYN_REG } else { CPU::BCLR_DYN_MEM }
        } else {
            if am.is_drd() { CPU::BCLR_STA_REG } else { CPU::BCLR_STA_MEM }
        };

        if am.is_drd() {
            count %= 32;
            let reg = am.register().unwrap() as usize;
            self.regs.sr.z = self.regs.d[reg].0 & 1 << count == 0;
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
        self.regs.pc.0 = pc.wrapping_add(disp as u32);

        Ok(if self.current_opcode as u8 == 0 {
            CPU::BRA_WORD
        } else {
            CPU::BRA_BYTE
        })
    }

    pub(super) fn execute_bset<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, am: AddressingMode, mut count: u8) -> InterpreterResult {
        let mut exec_time = if bits(self.current_opcode, 8, 8) != 0 {
            count = self.regs.d[count as usize].0 as u8;
            if am.is_drd() { CPU::BSET_DYN_REG } else { CPU::BSET_DYN_MEM }
        } else {
            if am.is_drd() { CPU::BSET_STA_REG } else { CPU::BSET_STA_MEM }
        };

        if am.is_drd() {
            count %= 32;
            let reg = am.register().unwrap() as usize;
            self.regs.sr.z = self.regs.d[reg].0 & 1 << count == 0;
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

    pub(super) fn execute_bsr<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, pc: u32, disp: i16) -> InterpreterResult {
        self.push_long(memory, self.regs.pc.0)?;
        self.regs.pc.0 = pc.wrapping_add(disp as u32);

        Ok(if self.current_opcode as u8 == 0 {
            CPU::BSR_WORD
        } else {
            CPU::BSR_BYTE
        })
    }

    pub(super) fn execute_btst<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, am: AddressingMode, mut count: u8) -> InterpreterResult {
        let mut exec_time = if bits(self.current_opcode, 8, 8) != 0 {
            count = self.regs.d[count as usize].0 as u8;
            if am.is_drd() { CPU::BTST_DYN_REG } else { CPU::BTST_DYN_MEM }
        } else {
            if am.is_drd() { CPU::BTST_STA_REG } else { CPU::BTST_STA_MEM }
        };

        if am.is_drd() {
            count %= 32;
            let reg = am.register().unwrap() as usize;
            self.regs.sr.z = self.regs.d[reg].0 & 1 << count == 0;
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
    pub(super) fn execute_chk<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, reg: u8, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = 0;

        let mut ea = EffectiveAddress::new(am, Some(Size::Word));

        let src = self.get_word(memory, &mut ea, &mut exec_time)? as i16;
        let data = self.regs.d[reg as usize].0 as i16;

        if data < 0 || data > src {
            Err(Vector::ChkInstruction as u8)
        } else {
            Ok(CPU::CHK_NO_TRAP + exec_time)
        }
    }

    pub(super) fn execute_clr<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, size: Size, am: AddressingMode) -> InterpreterResult {
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

    pub(super) fn execute_cmp<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, reg: u8, size: Size, am: AddressingMode) -> InterpreterResult {
        let mut exec_time;

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte => {
                exec_time = CPU::CMP_BW;
                let src = self.get_byte(memory, &mut ea, &mut exec_time)?;
                let dst = self.regs.d[reg as usize].0 as u8;

                self.sub::<u8, i8, false, true>(dst, src);
            },
            Size::Word => {
                exec_time = CPU::CMP_BW;
                let src = self.get_word(memory, &mut ea, &mut exec_time)?;
                let dst = self.regs.d[reg as usize].0 as u16;

                self.sub::<u16, i16, false, true>(dst, src);
            },
            Size::Long => {
                exec_time = CPU::CMP_L;
                let src = self.get_long(memory, &mut ea, &mut exec_time)?;
                let dst = self.regs.d[reg as usize].0;

                self.sub::<u32, i32, false, true>(dst, src);
            },
        }

        Ok(exec_time)
    }

    pub(super) fn execute_cmpa<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, reg: u8, size: Size, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = CPU::CMPA;

        let mut ea = EffectiveAddress::new(am, Some(size));

        let src = if size.is_word() {
            self.get_word(memory, &mut ea, &mut exec_time)? as i16 as u32
        } else {
            self.get_long(memory, &mut ea, &mut exec_time)?
        };

        self.sub::<u32, i32, false, true>(self.regs.a(reg), src);

        Ok(exec_time)
    }

    pub(super) fn execute_cmpi<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, size: Size, am: AddressingMode, imm: u32) -> InterpreterResult {
        let mut exec_time;

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte => {
                exec_time = if am.is_drd() { CPU::CMPI_REG_BW } else { CPU::CMPI_MEM_BW };
                let data = self.get_byte(memory, &mut ea, &mut exec_time)?;

                self.sub::<u8, i8, false, true>(data, imm as u8);
            },
            Size::Word => {
                exec_time = if am.is_drd() { CPU::CMPI_REG_BW } else { CPU::CMPI_MEM_BW };
                let data = self.get_word(memory, &mut ea, &mut exec_time)?;

                self.sub::<u16, i16, false, true>(data, imm as u16);
            },
            Size::Long => {
                exec_time = if am.is_drd() { CPU::CMPI_REG_L } else { CPU::CMPI_MEM_L };
                let data = self.get_long(memory, &mut ea, &mut exec_time)?;

                self.sub::<u32, i32, false, true>(data, imm);
            },
        }

        Ok(exec_time)
    }

    pub(super) fn execute_cmpm<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, ax: u8, size: Size, ay: u8) -> InterpreterResult {
        let addry = self.ariwpo(ay, size);
        let addrx = self.ariwpo(ax, size);

        match size {
            Size::Byte => {
                let src = memory.get_byte(addry).ok_or(ACCESS_ERROR)?;
                let dst = memory.get_byte(addrx).ok_or(ACCESS_ERROR)?;

                self.sub::<u8, i8, false, true>(dst, src);

                Ok(CPU::CMPM_BW)
            },
            Size::Word => {
                let src = memory.get_word(addry.even()?).ok_or(ACCESS_ERROR)?;
                let dst = memory.get_word(addrx.even()?).ok_or(ACCESS_ERROR)?;

                self.sub::<u16, i16, false, true>(dst, src);

                Ok(CPU::CMPM_BW)
            },
            Size::Long => {
                let src = memory.get_long(addry.even()?).ok_or(ACCESS_ERROR)?;
                let dst = memory.get_long(addrx.even()?).ok_or(ACCESS_ERROR)?;

                self.sub::<u32, i32, false, true>(dst, src);

                Ok(CPU::CMPM_L)
            },
        }
    }

    pub(super) fn execute_dbcc(&mut self, pc: u32, cc: u8, reg: u8, disp: i16) -> InterpreterResult {
        if !self.regs.sr.condition(cc) {
            let counter = (self.regs.d[reg as usize].0 as i16).wrapping_sub(1);
            self.regs.d_word(reg, counter as u16);

            if counter != -1 {
                self.regs.pc.0 = pc.wrapping_add(disp as u32);
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
    pub(super) fn execute_divs<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, reg: u8, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = CPU::DIVS;

        let mut ea = EffectiveAddress::new(am, Some(Size::Word));

        let src = self.get_word(memory, &mut ea, &mut exec_time)? as i16 as i32;
        let dst = self.regs.d[reg as usize].0 as i32;

        if src == 0 {
            Err(Vector::ZeroDivide as u8)
        } else {
            let quot = dst / src;
            let rem = dst % src;
            self.regs.d[reg as usize].0 = (rem as u16 as u32) << 16 | (quot as u16 as u32);

            self.regs.sr.n = quot < 0;
            self.regs.sr.z = quot == 0;
            self.regs.sr.v = quot < i16::MIN as i32 || quot > i16::MAX as i32;
            self.regs.sr.c = false;

            Ok(exec_time)
        }
    }

    /// If a zero divide exception occurs, this method returns the effective address calculation time, and the
    /// process_exception method returns the exception processing time.
    pub(super) fn execute_divu<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, reg: u8, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = CPU::DIVU;

        let mut ea = EffectiveAddress::new(am, Some(Size::Word));

        let src = self.get_word(memory, &mut ea, &mut exec_time)? as u32;
        let dst = self.regs.d[reg as usize].0;

        if src == 0 {
            Err(Vector::ZeroDivide as u8)
        } else {
            let quot = dst / src;
            let rem = dst % src;
            self.regs.d[reg as usize].0 = (rem as u16 as u32) << 16 | (quot as u16 as u32);

            self.regs.sr.n = quot & 0x0000_8000 != 0;
            self.regs.sr.z = quot == 0;
            self.regs.sr.v = (quot as i32) < i16::MIN as i32 || quot > i16::MAX as u32;
            self.regs.sr.c = false;

            Ok(exec_time)
        }
    }

    fn eor<UT>(&mut self, dst: UT, src: UT) -> UT
    where
        UT: Integer,
    {
        let res = src ^ dst;

        self.regs.sr.n = res & UT::SIGN_BIT_MASK != UT::ZERO;
        self.regs.sr.z = res == UT::ZERO;
        self.regs.sr.v = false;
        self.regs.sr.c = false;

        res
    }

    pub(super) fn execute_eor<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, reg: u8, size: Size, am: AddressingMode) -> InterpreterResult {
        let mut exec_time;

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte => {
                exec_time = if am.is_drd() { CPU::EOR_REG_BW } else { CPU::EOR_MEM_BW };
                let src = self.regs.d[reg as usize].0 as u8;
                let dst = self.get_byte(memory, &mut ea, &mut exec_time)?;

                let res = self.eor(dst, src);

                self.set_byte(memory, &mut ea, &mut exec_time, res)?;
            },
            Size::Word => {
                exec_time = if am.is_drd() { CPU::EOR_REG_BW } else { CPU::EOR_MEM_BW };
                let src = self.regs.d[reg as usize].0 as u16;
                let dst = self.get_word(memory, &mut ea, &mut exec_time)?;

                let res = self.eor(dst, src);

                self.set_word(memory, &mut ea, &mut exec_time, res)?;
            },
            Size::Long => {
                exec_time = if am.is_drd() { CPU::EOR_REG_L } else { CPU::EOR_MEM_L };
                let src = self.regs.d[reg as usize].0;
                let dst = self.get_long(memory, &mut ea, &mut exec_time)?;

                let res = self.eor(dst, src);

                self.set_long(memory, &mut ea, &mut exec_time, res)?;
            },
        }

        Ok(exec_time)
    }

    pub(super) fn execute_eori<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, size: Size, am: AddressingMode, imm: u32) -> InterpreterResult {
        let mut exec_time;

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte => {
                exec_time = if am.is_drd() { CPU::EORI_REG_BW } else { CPU::EORI_MEM_BW };
                let data = self.get_byte(memory, &mut ea, &mut exec_time)?;

                let res = self.eor(data, imm as u8);

                self.set_byte(memory, &mut ea, &mut exec_time, res)?;
            },
            Size::Word => {
                exec_time = if am.is_drd() { CPU::EORI_REG_BW } else { CPU::EORI_MEM_BW };
                let data = self.get_word(memory, &mut ea, &mut exec_time)?;

                let res = self.eor(data, imm as u16);

                self.set_word(memory, &mut ea, &mut exec_time, res)?;
            },
            Size::Long => {
                exec_time = if am.is_drd() { CPU::EORI_REG_L } else { CPU::EORI_MEM_L };
                let data = self.get_long(memory, &mut ea, &mut exec_time)?;

                let res = self.eor(data, imm);

                self.set_long(memory, &mut ea, &mut exec_time, res)?;
            },
        }

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
            self.regs.a_mut(ry).0 = self.regs.a(rx);
            self.regs.a_mut(rx).0 = y;
        } else {
            let y = self.regs.a(ry);
            self.regs.a_mut(ry).0 = self.regs.d[rx as usize].0;
            self.regs.d[rx as usize].0 = y;
        }

        Ok(CPU::EXG)
    }

    pub(super) fn execute_ext(&mut self, mode: u8, reg: u8) -> InterpreterResult {
        if mode == 0b010 {
            let d = self.regs.d[reg as usize].0 as i8 as u16;
            self.regs.d_word(reg, d);
        } else {
            self.regs.d[reg as usize].0 = self.regs.d[reg as usize].0 as i16 as u32;
        }

        self.regs.sr.n = self.regs.d[reg as usize].0 & SIGN_BIT_32 != 0;
        self.regs.sr.z = self.regs.d[reg as usize].0 == 0;
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
        self.regs.pc.0 = self.get_effective_address(&mut ea, &mut exec_time);

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

    pub(super) fn execute_jsr<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, am: AddressingMode) -> InterpreterResult {
        let mut ea = EffectiveAddress::new(am, None);

        let mut exec_time = 0;
        self.push_long(memory, self.regs.pc.0)?;
        self.regs.pc.0 = self.get_effective_address(&mut ea, &mut exec_time);

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
        self.regs.a_mut(reg).0 = self.get_effective_address(&mut ea, &mut exec_time);

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

    pub(super) fn execute_link<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, reg: u8, disp: i16) -> InterpreterResult {
        self.push_long(memory, self.regs.a(reg))?;
        self.regs.a_mut(reg).0 = self.regs.sp();
        *self.regs.sp_mut() += disp as u32;

        Ok(CPU::LINK)
    }

    pub(super) fn execute_lsm<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, dir: Direction, am: AddressingMode) -> InterpreterResult {
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
            (self.regs.d[rot as usize].0 % 64) as u8
        } else {
            if rot == 0 { 8 } else { rot }
        };

        let (mut data, mask) = match size {
            Size::Byte => (self.regs.d[reg as usize].0 & 0x0000_00FF, SIGN_BIT_8 as u32),
            Size::Word => (self.regs.d[reg as usize].0 & 0x0000_FFFF, SIGN_BIT_16 as u32),
            Size::Long => (self.regs.d[reg as usize].0, SIGN_BIT_32),
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
                self.regs.d[reg as usize].0 = data;
                self.regs.sr.z = data == 0;
                CPU::LSR_L + CPU::LSR_COUNT * shift_count as usize
            },
        })
    }

    pub(super) fn execute_move<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, size: Size, amdst: AddressingMode, amsrc: AddressingMode) -> InterpreterResult {
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

    pub(super) fn execute_movea<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, size: Size, reg: u8, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = CPU::MOVEA;

        let mut ea = EffectiveAddress::new(am, Some(size));

        self.regs.a_mut(reg).0 = if size.is_word() {
            self.get_word(memory, &mut ea, &mut exec_time)? as i16 as u32
        } else {
            self.get_long(memory, &mut ea, &mut exec_time)?
        };

        Ok(exec_time)
    }

    pub(super) fn execute_moveccr<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = CPU::MOVECCR;

        let mut ea = EffectiveAddress::new(am, Some(Size::Word));

        let ccr = self.get_word(memory, &mut ea, &mut exec_time)?;
        self.regs.sr.set_ccr(ccr);

        Ok(exec_time)
    }

    pub(super) fn execute_movefsr<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = if am.is_drd() { CPU::MOVEFSR_REG } else { CPU::MOVEFSR_MEM };

        let mut ea = EffectiveAddress::new(am, Some(Size::Word));

        self.set_word(memory, &mut ea, &mut exec_time, self.regs.sr.into())?;

        Ok(exec_time)
    }

    pub(super) fn execute_movesr<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, am: AddressingMode) -> InterpreterResult {
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
            self.regs.usp.0 = self.regs.a(reg);
        }
        Ok(CPU::MOVEUSP)
    }

    pub(super) fn execute_movem<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, dir: Direction, size: Size, am: AddressingMode, mut list: u16) -> InterpreterResult {
        let count = list.count_ones() as usize;
        let mut exec_time = 0;

        let mut ea = EffectiveAddress::new(am, Some(size));

        let gap = size as u32;
        let eareg = ea.mode.register().unwrap_or(u8::MAX);

        if ea.mode.is_ariwpr() {
            let mut addr = self.regs.a(eareg);

            for reg in (0..8).rev() {
                if list & 1 != 0 {
                    addr = addr.wrapping_sub(gap);
                    if size.is_word() { memory.set_word(addr.even()?, self.regs.a(reg) as u16).ok_or(ACCESS_ERROR)?; }
                        else { memory.set_long(addr.even()?, self.regs.a(reg)).ok_or(ACCESS_ERROR)?; }
                }

                list >>= 1;
            }

            for reg in (0..8).rev() {
                if list & 1 != 0 {
                    addr = addr.wrapping_sub(gap);
                    if size.is_word() { memory.set_word(addr.even()?, self.regs.d[reg].0 as u16).ok_or(ACCESS_ERROR)?; }
                        else { memory.set_long(addr.even()?, self.regs.d[reg].0).ok_or(ACCESS_ERROR)?; }
                }

                list >>= 1;
            }

            self.regs.a_mut(eareg).0 = addr;
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
                        self.regs.d[reg].0 = value;
                    } else {
                        if size.is_word() { memory.set_word(addr.even()?, self.regs.d[reg].0 as u16).ok_or(ACCESS_ERROR)?; }
                            else { memory.set_long(addr.even()?, self.regs.d[reg].0).ok_or(ACCESS_ERROR)?; }
                    }

                    addr = addr.wrapping_add(gap);
                }

                list >>= 1;
            }

            for reg in 0..8 {
                if list & 1 != 0 {
                    if dir == Direction::MemoryToRegister {
                        let value = if size.is_word() { memory.get_word(addr.even()?).ok_or(ACCESS_ERROR)? as i16 as u32 }
                            else { memory.get_long(addr.even()?).ok_or(ACCESS_ERROR)? };
                        self.regs.a_mut(reg).0 = value;
                    } else {
                        if size.is_word() { memory.set_word(addr.even()?, self.regs.a(reg as u8) as u16).ok_or(ACCESS_ERROR)?; }
                            else { memory.set_long(addr.even()?, self.regs.a(reg as u8)).ok_or(ACCESS_ERROR)?; }
                    }

                    addr = addr.wrapping_add(gap);
                }

                list >>= 1;
            }

            if ea.mode.is_ariwpo() {
                self.regs.a_mut(eareg).0 = addr;
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

    pub(super) fn execute_movep<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, data: u8, dir: Direction, size: Size, addr: u8, disp: i16) -> InterpreterResult {
        let mut shift = if size.is_word() { 8 } else { 24 };
        let mut addr = Wrapping(self.regs.a(addr).wrapping_add(disp as u32));

        if dir == Direction::RegisterToMemory {
            while shift >= 0 {
                let d = (self.regs.d[data as usize].0 >> shift) as u8;
                memory.set_byte(addr.0, d).ok_or(ACCESS_ERROR)?;
                shift -= 8;
                addr += 2;
            }

            Ok(if size.is_long() {
                CPU::MOVEP_RTM_LONG
            } else {
                CPU::MOVEP_RTM_WORD
            })
        } else {
            if size.is_word() { self.regs.d[data as usize] &= 0xFFFF_0000 } else { self.regs.d[data as usize].0 = 0 }

            while shift >= 0 {
                let d = memory.get_byte(addr.0).ok_or(ACCESS_ERROR)? as u32;
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
        self.regs.d[reg as usize].0 = data as u32;

        self.regs.sr.n = data <  0;
        self.regs.sr.z = data == 0;
        self.regs.sr.v = false;
        self.regs.sr.c = false;

        Ok(CPU::MOVEQ)
    }

    pub(super) fn execute_muls<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, reg: u8, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = CPU::MULS;

        let mut ea = EffectiveAddress::new(am, Some(Size::Word));

        let src = self.get_word(memory, &mut ea, &mut exec_time)? as i16 as i32;
        let dst = self.regs.d[reg as usize].0 as i16 as i32;

        let res = src * dst;
        self.regs.d[reg as usize].0 = res as u32;

        self.regs.sr.n = res < 0;
        self.regs.sr.z = res == 0;
        self.regs.sr.v = false;
        self.regs.sr.c = false;

        Ok(exec_time)
    }

    pub(super) fn execute_mulu<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, reg: u8, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = CPU::MULU;

        let mut ea = EffectiveAddress::new(am, Some(Size::Word));

        let src = self.get_word(memory, &mut ea, &mut exec_time)? as u32;
        let dst = self.regs.d[reg as usize].0 as u16 as u32;

        let res = src * dst;
        self.regs.d[reg as usize].0 = res;

        self.regs.sr.n = res & SIGN_BIT_32 != 0;
        self.regs.sr.z = res == 0;
        self.regs.sr.v = false;
        self.regs.sr.c = false;

        Ok(exec_time)
    }

    pub(super) fn execute_nbcd<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = if am.is_drd() { CPU::NBCD_REG } else { CPU::NBCD_MEM };

        let mut ea = EffectiveAddress::new(am, Some(Size::Byte));

        let data = self.get_byte(memory, &mut ea, &mut exec_time)?;

        let mut res = 0 - data - self.regs.sr.x as u8;
        if res != 0 {
            res -= 0x60;
        }
        if (res & 0x0F) != 0 {
            res -= 0x06;
        }

        self.regs.sr.n = res & 0x80 != 0;
        if res != 0 { self.regs.sr.z = false; }
        self.regs.sr.v = res != 0 && (res & 0x80) == 0 && data <= 0x80;
        self.regs.sr.c = res != 0;
        self.regs.sr.x = self.regs.sr.c;

        self.set_byte(memory, &mut ea, &mut exec_time, res)?;

        Ok(exec_time)
    }

    pub(super) fn execute_neg<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, size: Size, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = single_operands_time(size.is_long(), am.is_drd(), CPU::NEG_REG_BW, CPU::NEG_REG_L, CPU::NEG_MEM_BW, CPU::NEG_MEM_L);

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte => {
                let data = self.get_byte(memory, &mut ea, &mut exec_time)?;

                let res = self.sub::<u8, i8, false, false>(0, data);

                self.set_byte(memory, &mut ea, &mut exec_time, res)?;
            },
            Size::Word => {
                let data = self.get_word(memory, &mut ea, &mut exec_time)?;

                let res = self.sub::<u16, i16, false, false>(0, data);

                self.set_word(memory, &mut ea, &mut exec_time, res)?;
            },
            Size::Long => {
                let data = self.get_long(memory, &mut ea, &mut exec_time)?;

                let res = self.sub::<u32, i32, false, false>(0, data);

                self.set_long(memory, &mut ea, &mut exec_time, res)?;
            },
        }

        Ok(exec_time)
    }

    pub(super) fn execute_negx<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, size: Size, am: AddressingMode) -> InterpreterResult {
        let mut exec_time = single_operands_time(size.is_long(), am.is_drd(), CPU::NEGX_REG_BW, CPU::NEGX_REG_L, CPU::NEGX_MEM_BW, CPU::NEGX_MEM_L);

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte => {
                let data = self.get_byte(memory, &mut ea, &mut exec_time)?;

                let res = self.sub::<u8, i8, true, false>(0, data);

                self.set_byte(memory, &mut ea, &mut exec_time, res)?;
            },
            Size::Word => {
                let data = self.get_word(memory, &mut ea, &mut exec_time)?;

                let res = self.sub::<u16, i16, true, false>(0, data);

                self.set_word(memory, &mut ea, &mut exec_time, res)?;
            },
            Size::Long => {
                let data = self.get_long(memory, &mut ea, &mut exec_time)?;

                let res = self.sub::<u32, i32, true, false>(0, data);

                self.set_long(memory, &mut ea, &mut exec_time, res)?;
            },
        }

        Ok(exec_time)
    }

    pub(super) fn execute_nop(&self) -> InterpreterResult {
        Ok(CPU::NOP)
    }

    pub(super) fn execute_not<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, size: Size, am: AddressingMode) -> InterpreterResult {
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

    fn or<UT>(&mut self, dst: UT, src: UT) -> UT
    where
        UT: Integer,
    {
        let res = src | dst;

        self.regs.sr.n = res & UT::SIGN_BIT_MASK != UT::ZERO;
        self.regs.sr.z = res == UT::ZERO;
        self.regs.sr.v = false;
        self.regs.sr.c = false;

        res
    }

    pub(super) fn execute_or<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, reg: u8, dir: Direction, size: Size, am: AddressingMode) -> InterpreterResult {
        let mut exec_time;

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte => {
                if dir == Direction::DstEa {
                    exec_time = CPU::OR_MEM_BW;
                } else {
                    exec_time = CPU::OR_REG_BW;
                }
                let src = self.regs.d[reg as usize].0 as u8;
                let dst = self.get_byte(memory, &mut ea, &mut exec_time)?;

                let res = self.or(dst, src);

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
                let src = self.regs.d[reg as usize].0 as u16;
                let dst = self.get_word(memory, &mut ea, &mut exec_time)?;

                let res = self.or(dst, src);

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
                let src = self.regs.d[reg as usize].0;
                let dst = self.get_long(memory, &mut ea, &mut exec_time)?;

                let res = self.or(dst, src);

                if dir == Direction::DstEa {
                    self.set_long(memory, &mut ea, &mut exec_time, res)?;
                } else {
                    self.regs.d[reg as usize].0 = res;
                }
            },
        }

        Ok(exec_time)
    }

    pub(super) fn execute_ori<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, size: Size, am: AddressingMode, imm: u32) -> InterpreterResult {
        let mut exec_time;

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte => {
                exec_time = if am.is_drd() { CPU::ORI_REG_BW } else { CPU::ORI_MEM_BW };
                let data = self.get_byte(memory, &mut ea, &mut exec_time)?;

                let res = self.or(data, imm as u8);

                self.set_byte(memory, &mut ea, &mut exec_time, res)?;
            },
            Size::Word => {
                exec_time = if am.is_drd() { CPU::ORI_REG_BW } else { CPU::ORI_MEM_BW };
                let data = self.get_word(memory, &mut ea, &mut exec_time)?;

                let res = self.or(data, imm as u16);

                self.set_word(memory, &mut ea, &mut exec_time, res)?;
            },
            Size::Long => {
                exec_time = if am.is_drd() { CPU::ORI_REG_L } else { CPU::ORI_MEM_L };
                let data = self.get_long(memory, &mut ea, &mut exec_time)?;

                let res = self.or(data, imm);

                self.set_long(memory, &mut ea, &mut exec_time, res)?;
            },
        }

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

    pub(super) fn execute_pea<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, am: AddressingMode) -> InterpreterResult {
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

    pub(super) fn execute_reset<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        self.check_supervisor()?;

        memory.reset_instruction();
        Ok(CPU::RESET)
    }

    pub(super) fn execute_rom<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, dir: Direction, am: AddressingMode) -> InterpreterResult {
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
            (self.regs.d[rot as usize].0 % 64) as u8
        } else {
            if rot == 0 { 8 } else { rot }
        };

        let (mut data, mask) = match size {
            Size::Byte => (self.regs.d[reg as usize].0 & 0x0000_00FF, SIGN_BIT_8 as u32),
            Size::Word => (self.regs.d[reg as usize].0 & 0x0000_FFFF, SIGN_BIT_16 as u32),
            Size::Long => (self.regs.d[reg as usize].0, SIGN_BIT_32),
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
                self.regs.d[reg as usize].0 = data;
                self.regs.sr.z = data == 0;
                CPU::ROR_L + CPU::ROR_COUNT * shift_count as usize
            },
        })
    }

    pub(super) fn execute_roxm<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, dir: Direction, am: AddressingMode) -> InterpreterResult {
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
            (self.regs.d[rot as usize].0 % 64) as u8
        } else {
            if rot == 0 { 8 } else { rot }
        };

        let (mut data, mask) = match size {
            Size::Byte => (self.regs.d[reg as usize].0 & 0x0000_00FF, SIGN_BIT_8 as u32),
            Size::Word => (self.regs.d[reg as usize].0 & 0x0000_FFFF, SIGN_BIT_16 as u32),
            Size::Long => (self.regs.d[reg as usize].0, SIGN_BIT_32),
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
                self.regs.d[reg as usize].0 = data;
                self.regs.sr.z = data == 0;
                CPU::ROXR_L + CPU::ROXR_COUNT * shift_count as usize
            },
        })
    }

    pub(super) fn execute_rte<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        self.check_supervisor()?;

        let sr = self.pop_word(memory)?;
        self.regs.pc.0 = self.pop_long(memory)?;
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

    pub(super) fn execute_rtr<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let ccr = self.pop_word(memory)?;
        self.regs.sr &= SR_UPPER_MASK;
        self.regs.sr |= ccr & CCR_MASK;
        self.regs.pc.0 = self.pop_long(memory)?;

        Ok(CPU::RTR)
    }

    pub(super) fn execute_rts<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        self.regs.pc.0 = self.pop_long(memory)?;

        Ok(CPU::RTS)
    }

    pub(super) fn execute_sbcd<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, ry: u8, mode: Direction, rx: u8) -> InterpreterResult {
        let (src, dst) = if mode == Direction::MemoryToMemory {
            let src_addr = self.ariwpr(rx, Size::Byte);
            let dst_addr = self.ariwpr(ry, Size::Byte);
            (memory.get_byte(src_addr).ok_or(ACCESS_ERROR)?, memory.get_byte(dst_addr).ok_or(ACCESS_ERROR)?)
        } else {
            (self.regs.d[rx as usize].0 as u8, self.regs.d[ry as usize].0 as u8)
        };
        let src = src + self.regs.sr.x as u8;

        let bin_res = dst as u16 - src as u16;

        let mut res = (dst & 0x0F) - (src & 0x0F);
        if res >= 0x0A {
            res -= 0x06;
        }

        res += (dst & 0xF0) - (src & 0xF0);
        if res >= 0xA0 || bin_res > 0x99 {
            res -= 0x60;
        }

        res &= 0x00FF;

        self.regs.sr.n = res & 0x80 != 0;
        if res != 0 { self.regs.sr.z = false; }
        self.regs.sr.v = res < 0x80 && bin_res > 0x99;
        self.regs.sr.c = src > dst;
        self.regs.sr.x = self.regs.sr.c;

        if mode == Direction::MemoryToMemory {
            memory.set_byte(self.regs.a(ry), res).ok_or(ACCESS_ERROR)?;
            Ok(CPU::SBCD_MEM)
        } else {
            self.regs.d_byte(ry, res);
            Ok(CPU::SBCD_REG)
        }
    }

    pub(super) fn execute_scc<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, cc: u8, am: AddressingMode) -> InterpreterResult {
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

    /// Performs dst - src.
    fn sub<UT, ST, const SUBX: bool, const CMP: bool>(&mut self, dst: UT, src: UT) -> UT
    where
        UT: CarryingOps<ST, UT>,
        ST: Integer,
    {
        let (res, v) = dst.signed_borrowing_sub(src, SUBX && self.regs.sr.x);
        let (ures, c) = dst.unsigned_borrowing_sub(src, SUBX && self.regs.sr.x);

        if !CMP {
            self.regs.sr.x = c;
        }
        self.regs.sr.n = res < ST::ZERO;
        if SUBX {
            if res != ST::ZERO {
                self.regs.sr.z = false;
            }
        } else {
            self.regs.sr.z = res == ST::ZERO;
        }
        self.regs.sr.v = v;
        self.regs.sr.c = c;

        ures
    }

    pub(super) fn execute_sub<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, reg: u8, dir: Direction, size: Size, am: AddressingMode) -> InterpreterResult {
        let mut exec_time;

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte => {
                let (src, dst) = if dir == Direction::DstEa {
                    exec_time = CPU::SUB_MEM_BW;
                    (self.regs.d[reg as usize].0 as u8, self.get_byte(memory, &mut ea, &mut exec_time)?)
                } else {
                    exec_time = CPU::SUB_REG_BW;
                    (self.get_byte(memory, &mut ea, &mut exec_time)?, self.regs.d[reg as usize].0 as u8)
                };

                let res = self.sub::<u8, i8, false, false>(dst, src);

                if dir == Direction::DstEa {
                    self.set_byte(memory, &mut ea, &mut exec_time, res)?;
                } else {
                    self.regs.d_byte(reg, res);
                }
            },
            Size::Word => {
                let (src, dst) = if dir == Direction::DstEa {
                    exec_time = CPU::SUB_MEM_BW;
                    (self.regs.d[reg as usize].0 as u16, self.get_word(memory, &mut ea, &mut exec_time)?)
                } else {
                    exec_time = CPU::SUB_REG_BW;
                    (self.get_word(memory, &mut ea, &mut exec_time)?, self.regs.d[reg as usize].0 as u16)
                };

                let res = self.sub::<u16, i16, false, false>(dst, src);

                if dir == Direction::DstEa {
                    self.set_word(memory, &mut ea, &mut exec_time, res)?;
                } else {
                    self.regs.d_word(reg, res);
                }
            },
            Size::Long => {
                let (src, dst) = if dir == Direction::DstEa {
                    exec_time = CPU::SUB_MEM_L;
                    (self.regs.d[reg as usize].0, self.get_long(memory, &mut ea, &mut exec_time)?)
                } else {
                    exec_time = if am.is_dard() || am.is_immediate() { CPU::SUB_REG_L_RDIMM } else { CPU::SUB_REG_L };
                    (self.get_long(memory, &mut ea, &mut exec_time)?, self.regs.d[reg as usize].0)
                };

                let res = self.sub::<u32, i32, false, false>(dst, src);

                if dir == Direction::DstEa {
                    self.set_long(memory, &mut ea, &mut exec_time, res)?;
                } else {
                    self.regs.d[reg as usize].0 = res;
                }
            },
        }

        Ok(exec_time)
    }

    pub(super) fn execute_suba<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, reg: u8, size: Size, am: AddressingMode) -> InterpreterResult {
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

    pub(super) fn execute_subi<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, size: Size, am: AddressingMode, imm: u32) -> InterpreterResult {
        let mut exec_time;

        let mut ea = EffectiveAddress::new(am, Some(size));

        match size {
            Size::Byte => {
                exec_time = if am.is_drd() { CPU::SUBI_REG_BW } else { CPU::SUBI_MEM_BW };
                let data = self.get_byte(memory, &mut ea, &mut exec_time)?;

                let res = self.sub::<u8, i8, false, false>(data, imm as u8);

                self.set_byte(memory, &mut ea, &mut exec_time, res)?;
            },
            Size::Word => {
                exec_time = if am.is_drd() { CPU::SUBI_REG_BW } else { CPU::SUBI_MEM_BW };
                let data = self.get_word(memory, &mut ea, &mut exec_time)?;

                let res = self.sub::<u16, i16, false, false>(data, imm as u16);

                self.set_word(memory, &mut ea, &mut exec_time, res)?;
            },
            Size::Long => {
                exec_time = if am.is_drd() { CPU::SUBI_REG_L } else { CPU::SUBI_MEM_L };
                let data = self.get_long(memory, &mut ea, &mut exec_time)?;

                let res = self.sub::<u32, i32, false, false>(data, imm);

                self.set_long(memory, &mut ea, &mut exec_time, res)?;
            },
        }

        Ok(exec_time)
    }

    pub(super) fn execute_subq<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, imm: u8, size: Size, am: AddressingMode) -> InterpreterResult {
        let imm = if imm == 0 { 8 } else { imm };

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

                let res = self.sub::<u8, i8, false, false>(data, imm);

                self.set_byte(memory, &mut ea, &mut exec_time, res)?;
            },
            Size::Word => {
                exec_time = if am.is_drd() { CPU::SUBQ_DREG_BW } else { CPU::SUBQ_MEM_BW };
                let data = self.get_word(memory, &mut ea, &mut exec_time)?;

                let res = self.sub::<u16, i16, false, false>(data, imm as u16);

                self.set_word(memory, &mut ea, &mut exec_time, res)?;
            },
            Size::Long => {
                exec_time = if am.is_drd() { CPU::SUBQ_REG_L } else { CPU::SUBQ_MEM_L };
                let data = self.get_long(memory, &mut ea, &mut exec_time)?;

                let res = self.sub::<u32, i32, false, false>(data, imm as u32);

                self.set_long(memory, &mut ea, &mut exec_time, res)?;
            },
        }

        Ok(exec_time)
    }

    pub(super) fn execute_subx<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, ry: u8, size: Size, mode: Direction, rx: u8) -> InterpreterResult {
        match size {
            Size::Byte => {
                let (src, dst) = if mode == Direction::MemoryToMemory {
                    let src_addr = self.ariwpr(rx, size);
                    let dst_addr = self.ariwpr(ry, size);
                    (memory.get_byte(src_addr).ok_or(ACCESS_ERROR)?, memory.get_byte(dst_addr).ok_or(ACCESS_ERROR)?)
                } else {
                    (self.regs.d[rx as usize].0 as u8, self.regs.d[ry as usize].0 as u8)
                };

                let res = self.sub::<u8, i8, true, false>(dst, src);

                if mode == Direction::MemoryToMemory {
                    memory.set_byte(self.regs.a(ry), res).ok_or(ACCESS_ERROR)?;
                    Ok(CPU::SUBX_MEM_BW)
                } else {
                    self.regs.d_byte(ry, res);
                    Ok(CPU::SUBX_REG_BW)
                }
            },
            Size::Word => {
                let (src, dst) = if mode == Direction::MemoryToMemory {
                    let src_addr = self.ariwpr(rx, size);
                    let dst_addr = self.ariwpr(ry, size);
                    (memory.get_word(src_addr.even()?).ok_or(ACCESS_ERROR)?, memory.get_word(dst_addr.even()?).ok_or(ACCESS_ERROR)?)
                } else {
                    (self.regs.d[rx as usize].0 as u16, self.regs.d[ry as usize].0 as u16)
                };

                let res = self.sub::<u16, i16, true, false>(dst, src);

                if mode == Direction::MemoryToMemory {
                    memory.set_word(self.regs.a(ry), res).ok_or(ACCESS_ERROR)?;
                    Ok(CPU::SUBX_MEM_BW)
                } else {
                    self.regs.d_word(ry, res);
                    Ok(CPU::SUBX_REG_BW)
                }
            },
            Size::Long => {
                let (src, dst) = if mode == Direction::MemoryToMemory {
                    let src_addr = self.ariwpr(rx, size);
                    let dst_addr = self.ariwpr(ry, size);
                    (memory.get_long(src_addr.even()?).ok_or(ACCESS_ERROR)?, memory.get_long(dst_addr.even()?).ok_or(ACCESS_ERROR)?)
                } else {
                    (self.regs.d[rx as usize].0, self.regs.d[ry as usize].0)
                };

                let res = self.sub::<u32, i32, true, false>(dst, src);

                if mode == Direction::MemoryToMemory {
                    memory.set_long(self.regs.a(ry), res).ok_or(ACCESS_ERROR)?;
                    Ok(CPU::SUBX_MEM_L)
                } else {
                    self.regs.d[ry as usize].0 = res;
                    Ok(CPU::SUBX_REG_L)
                }
            },
        }
    }

    pub(super) fn execute_swap(&mut self, reg: u8) -> InterpreterResult {
        let high = self.regs.d[reg as usize] >> 16;
        self.regs.d[reg as usize] <<= 16;
        self.regs.d[reg as usize] |= high;

        self.regs.sr.n = self.regs.d[reg as usize].0 & SIGN_BIT_32 != 0;
        self.regs.sr.z = self.regs.d[reg as usize].0 == 0;
        self.regs.sr.v = false;
        self.regs.sr.c = false;

        Ok(CPU::SWAP)
    }

    pub(super) fn execute_tas<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, am: AddressingMode) -> InterpreterResult {
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

    pub(super) fn execute_tst<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, size: Size, am: AddressingMode) -> InterpreterResult {
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

    pub(super) fn execute_unlk<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, reg: u8) -> InterpreterResult {
        self.regs.sp_mut().0 = self.regs.a(reg);
        self.regs.a_mut(reg).0 = self.pop_long(memory)?;

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
