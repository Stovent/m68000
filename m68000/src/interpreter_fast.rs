// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{CpuDetails, M68000, MemoryAccess};
use crate::exception::{Exception, Vector};
use crate::instruction::*;
use crate::interpreter::InterpreterResult;
use crate::isa::Isa;

impl<CPU: CpuDetails> M68000<CPU> {
    /// Runs the CPU for **at least** the given number of cycles.
    ///
    /// Returns the number of cycles actually executed.
    ///
    /// If you ask to execute 4 cycles but the next instruction takes 6 cycles to execute, it will be executed
    /// and 6 is returned. It is the caller's responsibility to handle the extra cycles.
    pub fn cycle<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, cycles: usize) -> usize {
        let mut total = 0;

        while total < cycles {
            total += self.interpreter(memory);

            if self.stop {
                return cycles;
            }
        }

        total
    }

    /// Runs the CPU until either an exception occurs or **at least** the given number cycles have been executed.
    ///
    /// Returns the number of cycles actually executed, and the exception that occured if any.
    ///
    /// If you ask to execute 4 cycles but the next instruction takes 6 cycles to execute, it will be executed
    /// and 6 is returned, along with the vector that occured if any.
    /// It is the caller's responsibility to handle the extra cycles.
    pub fn cycle_until_exception<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, cycles: usize) -> (usize, Option<u8>) {
        let mut total = 0;

        while total < cycles {
            let (c, v) = self.interpreter_exception(memory);
            total += c;

            if v.is_some() || self.stop {
                return (total, v);
            }
        }

        (total, None)
    }

    /// Runs indefinitely until an exception or STOP instruction occurs.
    ///
    /// Returns the number of cycles executed and the exception that occured.
    /// If exception is None, this means the CPU has executed a STOP instruction.
    pub fn loop_until_exception_stop<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> (usize, Option<u8>) {
        let mut total_cycles = 0;

        loop {
            let (cycles, vector) = self.interpreter_exception(memory);
            total_cycles += cycles;

            if vector.is_some() || self.stop {
                return (total_cycles, vector);
            }
        }
    }

    /// Runs the interpreter loop once, returning the cycle count necessary to execute it.
    ///
    /// If the CPU is stopped, returns 0.
    ///
    /// This method may or may not execute any instruction.
    /// For example, if an Access Error occurs during instruction fetch or if the CPU is stopped, it returns 0 and no instruction is executed.
    pub fn interpreter<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> usize {
        let (cycles, exception) = self.interpreter_exception(memory);
        if let Some(e) = exception {
            self.exception(Exception::from(e));
        }
        cycles
    }

    /// Runs the interpreter loop once, returning the cycle count necessary to execute it
    /// and the vector of the exception that occured during the execution if any.
    ///
    /// To process the returned exception, call [M68000::exception].
    ///
    /// If the CPU is stopped, returns (0, None).
    ///
    /// This method may or may not execute any instruction.
    /// For example, if an Access Error occurs during instruction fetch, the exception is returned and no instruction is executed.
    pub fn interpreter_exception<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> (usize, Option<u8>) {
        if self.stop {
            return (0, None);
        }

        let mut cycle_count = 0;

        if !self.exceptions.is_empty() {
            cycle_count += self.process_pending_exceptions(memory);
        }

        let opcode = match self.get_next_word(memory) {
            Ok(op) => op,
            Err(e) => return (cycle_count, Some(e)),
        };
        self.current_opcode = opcode;
        let isa = Isa::from(opcode);

        let trace = self.regs.sr.t;
        let exception = match Execute::<CPU, M>::EXECUTE[isa as usize](self, memory) {
            Ok(cycles) => {
                cycle_count += cycles;
                if trace && !isa.is_privileged() {
                    Some(Vector::Trace as u8)
                } else {
                    None
                }
            },
            Err(e) => Some(e),
        };

        (cycle_count, exception)
    }

    fn fast_unknown_instruction<M: MemoryAccess + ?Sized>(&mut self, _: &mut M) -> InterpreterResult {
        self.execute_unknown_instruction()
    }

    fn fast_abcd<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let (rx, _, mode, ry) = register_size_mode_register(self.current_opcode);
        self.execute_abcd(memory, rx, mode, ry)
    }

    fn fast_add<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (reg, dir, size, am) = register_direction_size_effective_address(opcode, &mut iter);
        self.execute_add(memory, reg, dir, size, am)
    }

    fn fast_adda<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (reg, size, am) = register_size_effective_address(opcode, &mut iter);
        self.execute_adda(memory, reg, size, am)
    }

    fn fast_addi<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (size, am, imm) = size_effective_address_immediate(opcode, &mut iter);
        self.execute_addi(memory, size, am, imm)
    }

    fn fast_addq<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (imm, size, am) = data_size_effective_address(opcode, &mut iter);
        self.execute_addq(memory, imm, size, am)
    }

    fn fast_addx<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let (rx, size, mode, ry) = register_size_mode_register(self.current_opcode);
        self.execute_addx(memory, rx, size, mode, ry)
    }

    fn fast_and<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (reg, dir, size, am) = register_direction_size_effective_address(opcode, &mut iter);
        self.execute_and(memory, reg, dir, size, am)
    }

    fn fast_andi<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (size, am, imm) = size_effective_address_immediate(opcode, &mut iter);
        self.execute_andi(memory, size, am, imm)
    }

    fn fast_andiccr<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let mut iter = self.iter_from_pc(memory)?;
        let imm = immediate(&mut iter);
        self.execute_andiccr(imm)
    }

    fn fast_andisr<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let mut iter = self.iter_from_pc(memory)?;
        let imm = immediate(&mut iter);
        self.execute_andisr(imm)
    }

    fn fast_asm<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (dir, am) = direction_effective_address(opcode, &mut iter);
        self.execute_asm(memory, dir, am)
    }

    fn fast_asr<M: MemoryAccess + ?Sized>(&mut self, _: &mut M) -> InterpreterResult {
        let (rot, dir, size, ir, reg) = rotation_direction_size_mode_register(self.current_opcode);
        self.execute_asr(rot, dir, size, ir, reg)
    }

    fn fast_bcc<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let pc = self.regs.pc.0;
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (condition, displacement) = condition_displacement(opcode, &mut iter);
        self.execute_bcc(pc, condition, displacement)
    }

    fn fast_bchg<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (am, count) = effective_address_count(opcode, &mut iter);
        self.execute_bchg(memory, am, count)
    }

    fn fast_bclr<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (am, count) = effective_address_count(opcode, &mut iter);
        self.execute_bclr(memory, am, count)
    }

    fn fast_bra<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let pc = self.regs.pc.0;
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let disp = displacement(opcode, &mut iter);
        self.execute_bra(pc, disp)
    }

    fn fast_bset<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (am, count) = effective_address_count(opcode, &mut iter);
        self.execute_bset(memory, am, count)
    }

    fn fast_bsr<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let pc = self.regs.pc.0;
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let disp = displacement(opcode, &mut iter);
        self.execute_bsr(memory, pc, disp)
    }

    fn fast_btst<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (am, count) = effective_address_count(opcode, &mut iter);
        self.execute_btst(memory, am, count)
    }

    /// If a CHK exception occurs, this method returns the effective address calculation time, and the
    /// process_exception method returns the exception processing time.
    fn fast_chk<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (reg, am) = register_effective_address(opcode, &mut iter);
        self.execute_chk(memory, reg, am)
    }

    fn fast_clr<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (size, am) = size_effective_address(opcode, &mut iter);
        self.execute_clr(memory, size, am)
    }

    fn fast_cmp<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (reg, _, size, am) = register_direction_size_effective_address(opcode, &mut iter);
        self.execute_cmp(memory, reg, size, am)
    }

    fn fast_cmpa<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (reg, size, am) = register_size_effective_address(opcode, &mut iter);
        self.execute_cmpa(memory, reg, size, am)
    }

    fn fast_cmpi<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (size, am, imm) = size_effective_address_immediate(opcode, &mut iter);
        self.execute_cmpi(memory, size, am, imm)
    }

    fn fast_cmpm<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let (ax, size, ay) = register_size_register(self.current_opcode);
        self.execute_cmpm(memory, ax, size, ay)
    }

    fn fast_dbcc<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let pc = self.regs.pc.0;
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (cc, reg, disp) = condition_register_displacement(opcode, &mut iter);
        self.execute_dbcc(pc, cc, reg, disp)
    }

    /// If a zero divide exception occurs, this method returns the effective address calculation time, and the
    /// process_exception method returns the exception processing time.
    fn fast_divs<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (reg, am) = register_effective_address(opcode, &mut iter);
        self.execute_divs(memory, reg, am)
    }

    /// If a zero divide exception occurs, this method returns the effective address calculation time, and the
    /// process_exception method returns the exception processing time.
    fn fast_divu<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (reg, am) = register_effective_address(opcode, &mut iter);
        self.execute_divu(memory, reg, am)
    }

    fn fast_eor<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (reg, _, size, am) = register_direction_size_effective_address(opcode, &mut iter);
        self.execute_eor(memory, reg, size, am)
    }

    fn fast_eori<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (size, am, imm) = size_effective_address_immediate(opcode, &mut iter);
        self.execute_eori(memory, size, am, imm)
    }

    fn fast_eoriccr<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let mut iter = self.iter_from_pc(memory)?;
        let imm = immediate(&mut iter);
        self.execute_eoriccr(imm)
    }

    fn fast_eorisr<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let mut iter = self.iter_from_pc(memory)?;
        let imm = immediate(&mut iter);
        self.execute_eorisr(imm)
    }

    fn fast_exg<M: MemoryAccess + ?Sized>(&mut self, _: &mut M) -> InterpreterResult {
        let (rx, mode, ry) = register_opmode_register(self.current_opcode);
        self.execute_exg(rx, mode, ry)
    }

    fn fast_ext<M: MemoryAccess + ?Sized>(&mut self, _: &mut M) -> InterpreterResult {
        let (mode, reg) = opmode_register(self.current_opcode);
        self.execute_ext(mode, reg)
    }

    fn fast_illegal<M: MemoryAccess + ?Sized>(&mut self, _: &mut M) -> InterpreterResult {
        self.execute_illegal()
    }

    fn fast_jmp<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let am = effective_address(opcode, &mut iter);
        self.execute_jmp(am)
    }

    fn fast_jsr<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let am = effective_address(opcode, &mut iter);
        self.execute_jsr(memory, am)
    }

    fn fast_lea<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (reg, am) = register_effective_address(opcode, &mut iter);
        self.execute_lea(reg, am)
    }

    fn fast_link<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (reg, disp) = register_displacement(opcode, &mut iter);
        self.execute_link(memory, reg, disp)
    }

    fn fast_lsm<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (dir, am) = direction_effective_address(opcode, &mut iter);
        self.execute_lsm(memory, dir, am)
    }

    fn fast_lsr<M: MemoryAccess + ?Sized>(&mut self, _: &mut M) -> InterpreterResult {
        let (rot, dir, size, ir, reg) = rotation_direction_size_mode_register(self.current_opcode);
        self.execute_lsr(rot, dir, size, ir, reg)
    }

    fn fast_move<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (size, amdst, amsrc) = size_effective_address_effective_address(opcode, &mut iter);
        self.execute_move(memory, size, amdst, amsrc)
    }

    fn fast_movea<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (size, reg, am) = size_register_effective_address(opcode, &mut iter);
        self.execute_movea(memory, size, reg, am)
    }

    fn fast_moveccr<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let am = effective_address(opcode, &mut iter);
        self.execute_moveccr(memory, am)
    }

    fn fast_movefsr<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let am = effective_address(opcode, &mut iter);
        self.execute_movefsr(memory, am)
    }

    fn fast_movesr<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let am = effective_address(opcode, &mut iter);
        self.execute_movesr(memory, am)
    }

    fn fast_moveusp<M: MemoryAccess + ?Sized>(&mut self, _: &mut M) -> InterpreterResult {
        let (dir, reg) = direction_register(self.current_opcode);
        self.execute_moveusp(dir, reg)
    }

    fn fast_movem<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (dir, size, am, list) = direction_size_effective_address_list(opcode, &mut iter);
        self.execute_movem(memory, dir, size, am, list)
    }

    fn fast_movep<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (data, dir, size, addr, disp) = register_direction_size_register_displacement(opcode, &mut iter);
        self.execute_movep(memory, data, dir, size, addr, disp)
    }

    fn fast_moveq<M: MemoryAccess + ?Sized>(&mut self, _: &mut M) -> InterpreterResult {
        let (reg, data) = register_data(self.current_opcode);
        self.execute_moveq(reg, data)
    }

    fn fast_muls<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (reg, am) = register_effective_address(opcode, &mut iter);
        self.execute_muls(memory, reg, am)
    }

    fn fast_mulu<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (reg, am) = register_effective_address(opcode, &mut iter);
        self.execute_mulu(memory, reg, am)
    }

    fn fast_nbcd<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let am = effective_address(opcode, &mut iter);
        self.execute_nbcd(memory, am)
    }

    fn fast_neg<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (size, am) = size_effective_address(opcode, &mut iter);
        self.execute_neg(memory, size, am)
    }

    fn fast_negx<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (size, am) = size_effective_address(opcode, &mut iter);
        self.execute_negx(memory, size, am)
    }

    fn fast_nop<M: MemoryAccess + ?Sized>(&mut self, _: &mut M) -> InterpreterResult {
        self.execute_nop()
    }

    fn fast_not<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (size, am) = size_effective_address(opcode, &mut iter);
        self.execute_not(memory, size, am)
    }

    fn fast_or<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (reg, dir, size, am) = register_direction_size_effective_address(opcode, &mut iter);
        self.execute_or(memory, reg, dir, size, am)
    }

    fn fast_ori<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (size, am, imm) = size_effective_address_immediate(opcode, &mut iter);
        self.execute_ori(memory, size, am, imm)
    }

    fn fast_oriccr<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let mut iter = self.iter_from_pc(memory)?;
        let imm = immediate(&mut iter);
        self.execute_oriccr(imm)
    }

    fn fast_orisr<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let mut iter = self.iter_from_pc(memory)?;
        let imm = immediate(&mut iter);
        self.execute_orisr(imm)
    }

    fn fast_pea<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let am = effective_address(opcode, &mut iter);
        self.execute_pea(memory, am)
    }

    fn fast_reset<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        self.execute_reset(memory)
    }

    fn fast_rom<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (dir, am) = direction_effective_address(opcode, &mut iter);
        self.execute_rom(memory, dir, am)
    }

    fn fast_ror<M: MemoryAccess + ?Sized>(&mut self, _: &mut M) -> InterpreterResult {
        let (rot, dir, size, ir, reg) = rotation_direction_size_mode_register(self.current_opcode);
        self.execute_ror(rot, dir, size, ir, reg)
    }

    fn fast_roxm<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (dir, am) = direction_effective_address(opcode, &mut iter);
        self.execute_roxm(memory, dir, am)
    }

    fn fast_roxr<M: MemoryAccess + ?Sized>(&mut self, _: &mut M) -> InterpreterResult {
        let (rot, dir, size, ir, reg) = rotation_direction_size_mode_register(self.current_opcode);
        self.execute_roxr(rot, dir, size, ir, reg)
    }

    fn fast_rte<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        self.execute_rte(memory)
    }

    fn fast_rtr<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        self.execute_rtr(memory)
    }

    fn fast_rts<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        self.execute_rts(memory)
    }

    fn fast_sbcd<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let (ry, _, mode, rx) = register_size_mode_register(self.current_opcode);
        self.execute_sbcd(memory, ry, mode, rx)
    }

    fn fast_scc<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (cc, am) = condition_effective_address(opcode, &mut iter);
        self.execute_scc(memory, cc, am)
    }

    fn fast_stop<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let mut iter = self.iter_from_pc(memory)?;
        let imm = immediate(&mut iter);
        self.execute_stop(imm)
    }

    fn fast_sub<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (reg, dir, size, am) = register_direction_size_effective_address(opcode, &mut iter);
        self.execute_sub(memory, reg, dir, size, am)
    }

    fn fast_suba<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (reg, size, am) = register_size_effective_address(opcode, &mut iter);
        self.execute_suba(memory, reg, size, am)
    }

    fn fast_subi<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (size, am, imm) = size_effective_address_immediate(opcode, &mut iter);
        self.execute_subi(memory, size, am, imm)
    }

    fn fast_subq<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (imm, size, am) = data_size_effective_address(opcode, &mut iter);
        self.execute_subq(memory, imm, size, am)
    }

    fn fast_subx<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let (ry, size, mode, rx) = register_size_mode_register(self.current_opcode);
        self.execute_subx(memory, ry, size, mode, rx)
    }

    fn fast_swap<M: MemoryAccess + ?Sized>(&mut self, _: &mut M) -> InterpreterResult {
        let reg = register(self.current_opcode);
        self.execute_swap(reg)
    }

    fn fast_tas<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let am = effective_address(opcode, &mut iter);
        self.execute_tas(memory, am)
    }

    fn fast_trap<M: MemoryAccess + ?Sized>(&mut self, _: &mut M) -> InterpreterResult {
        let vector = vector(self.current_opcode);
        self.execute_trap(vector)
    }

    fn fast_trapv<M: MemoryAccess + ?Sized>(&mut self, _: &mut M) -> InterpreterResult {
        self.execute_trapv()
    }

    fn fast_tst<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let opcode = self.current_opcode;
        let mut iter = self.iter_from_pc(memory)?;
        let (size, am) = size_effective_address(opcode, &mut iter);
        self.execute_tst(memory, size, am)
    }

    fn fast_unlk<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> InterpreterResult {
        let reg = register(self.current_opcode);
        self.execute_unlk(memory, reg)
    }
}

struct Execute<E: CpuDetails, M: MemoryAccess + ?Sized> {
    _e: E,
    _m: M,
}

impl<E: CpuDetails, M: MemoryAccess + ?Sized> Execute<E, M> {
    /// Function used to execute the instruction.
    const EXECUTE: [fn(&mut M68000<E>, &mut M) -> InterpreterResult; Isa::_Size as usize] = [
        M68000::fast_unknown_instruction,
        M68000::fast_abcd,
        M68000::fast_add,
        M68000::fast_adda,
        M68000::fast_addi,
        M68000::fast_addq,
        M68000::fast_addx,
        M68000::fast_and,
        M68000::fast_andi,
        M68000::fast_andiccr,
        M68000::fast_andisr,
        M68000::fast_asm,
        M68000::fast_asr,
        M68000::fast_bcc,
        M68000::fast_bchg,
        M68000::fast_bclr,
        M68000::fast_bra,
        M68000::fast_bset,
        M68000::fast_bsr,
        M68000::fast_btst,
        M68000::fast_chk,
        M68000::fast_clr,
        M68000::fast_cmp,
        M68000::fast_cmpa,
        M68000::fast_cmpi,
        M68000::fast_cmpm,
        M68000::fast_dbcc,
        M68000::fast_divs,
        M68000::fast_divu,
        M68000::fast_eor,
        M68000::fast_eori,
        M68000::fast_eoriccr,
        M68000::fast_eorisr,
        M68000::fast_exg,
        M68000::fast_ext,
        M68000::fast_illegal,
        M68000::fast_jmp,
        M68000::fast_jsr,
        M68000::fast_lea,
        M68000::fast_link,
        M68000::fast_lsm,
        M68000::fast_lsr,
        M68000::fast_move,
        M68000::fast_movea,
        M68000::fast_moveccr,
        M68000::fast_movefsr,
        M68000::fast_movesr,
        M68000::fast_moveusp,
        M68000::fast_movem,
        M68000::fast_movep,
        M68000::fast_moveq,
        M68000::fast_muls,
        M68000::fast_mulu,
        M68000::fast_nbcd,
        M68000::fast_neg,
        M68000::fast_negx,
        M68000::fast_nop,
        M68000::fast_not,
        M68000::fast_or,
        M68000::fast_ori,
        M68000::fast_oriccr,
        M68000::fast_orisr,
        M68000::fast_pea,
        M68000::fast_reset,
        M68000::fast_rom,
        M68000::fast_ror,
        M68000::fast_roxm,
        M68000::fast_roxr,
        M68000::fast_rte,
        M68000::fast_rtr,
        M68000::fast_rts,
        M68000::fast_sbcd,
        M68000::fast_scc,
        M68000::fast_stop,
        M68000::fast_sub,
        M68000::fast_suba,
        M68000::fast_subi,
        M68000::fast_subq,
        M68000::fast_subx,
        M68000::fast_swap,
        M68000::fast_tas,
        M68000::fast_trap,
        M68000::fast_trapv,
        M68000::fast_tst,
        M68000::fast_unlk,
    ];
}
