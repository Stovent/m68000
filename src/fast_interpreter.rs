use crate::{M68000, MemoryAccess};
use crate::exception::Vector;
use crate::interpreter::InterpreterResult;
use crate::isa::Isa;

impl M68000 {
    /// Runs the CPU for `cycles` number of cycles.
    ///
    /// This function executes **at least** the given number of cycles.
    /// Returns the number of cycles actually executed.
    ///
    /// If you ask to execute 4 cycles but the next instruction takes 6 cycles to execute,
    /// it will be executed and the 2 extra cycles will be subtracted in the next call.
    pub fn cycle(&mut self, memory: &mut impl MemoryAccess, cycles: usize) -> usize {
        if self.cycles >= cycles {
            self.cycles -= cycles;
            return 0;
        }

        let initial = self.cycles;

        while self.cycles < cycles {
            self.cycles += self.interpreter(memory);
            if self.stop {
                self.cycles = cycles;
            }
        }

        let total = self.cycles - initial;
        self.cycles -= cycles;
        total
    }

    /// Runs the CPU until either an exception occurs or `cycle` cycles have been executed.
    ///
    /// This function executes **at least** the given number of cycles.
    /// Returns the number of cycles actually executed, and the exception that occured if any.
    ///
    /// If you ask to execute 4 cycles but the next instruction takes 6 cycles to execute,
    /// it will be executed and the 2 extra cycles will be subtracted in the next call.
    pub fn cycle_until_exception(&mut self, memory: &mut impl MemoryAccess, cycles: usize) -> (usize, Option<u8>) {
        if self.cycles >= cycles {
            self.cycles -= cycles;
            return (0, None);
        }

        let initial = self.cycles;
        let mut vector = None;

        while self.cycles < cycles {
            let (c, v) = self.interpreter_exception(memory);
            self.cycles += c;

            if v.is_some() {
                vector = v;
                break;
            }
            if self.stop {
                self.cycles = cycles;
            }
        }

        let total = self.cycles - initial;
        if self.cycles >= cycles {
            self.cycles -= cycles;
        } else {
            self.cycles = 0;
        }
        (total, vector)
    }

    /// Runs indefinitely until an exception or STOP instruction occurs.
    ///
    /// Returns the number of cycles executed and the exception that occured.
    /// If exception is None, this means the CPU has executed a STOP instruction.
    pub fn loop_until_exception_stop(&mut self, memory: &mut impl MemoryAccess) -> (usize, Option<u8>) {
        let mut total_cycles = self.cycles;
        self.cycles = 0;

        loop {
            let (cycles, vector) = self.interpreter_exception(memory);
            total_cycles += cycles;

            if vector.is_some() || self.stop {
                return (total_cycles, vector);
            }
        }
    }

    /// Executes the next instruction, returning the cycle count necessary to execute it.
    pub fn interpreter<M: MemoryAccess>(&mut self, memory: &mut M) -> usize {
        let (cycles, exception) = self.interpreter_exception(memory);
        if let Some(e) = exception {
            self.exception(e);
        }
        cycles
    }

    /// Executes the next instruction, returning the cycle count necessary to execute it,
    /// and the vector of the exception that occured during the execution if any.
    ///
    /// To process the returned exception, call [M68000::exception].
    pub fn interpreter_exception<M: MemoryAccess>(&mut self, memory: &mut M) -> (usize, Option<u8>) {
        let mut cycle_count = 0;

        if !self.exceptions.is_empty() {
            cycle_count += self.process_pending_exceptions(memory);
        }

        if self.stop {
            return (0, None);
        }

        let opcode = match self.get_next_word(memory) {
            Ok(op) => op,
            Err(e) => return (cycle_count, Some(e)),
        };
        self.current_opcode = opcode;
        let isa: Isa = opcode.into();

        let trace = self.regs.sr.t;
        match Execute::<M>::EXECUTE[isa as usize](self, memory) {
            Ok(cycles) => {
                cycle_count += cycles;
                if trace { self.exception(Vector::Trace as u8); }
            },
            Err(e) => return (cycle_count, Some(e)),
        }

        (cycle_count, None)
    }

    fn fast_unknown_instruction(&mut self, _: &mut impl MemoryAccess) -> InterpreterResult {
        self.execute_unknown_instruction()
    }

    fn fast_abcd(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (rx, _, mode, ry) = self.register_size_mode_register();
        self.execute_abcd(memory, rx, mode, ry)
    }

    fn fast_add(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (reg, dir, size, am) = self.register_direction_size_effective_address(memory);
        self.execute_add(memory, reg, dir, size, am)
    }

    fn fast_adda(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (reg, size, am) = self.register_size_effective_address(memory);
        self.execute_adda(memory, reg, size, am)
    }

    fn fast_addi(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (size, am, imm) = self.size_effective_address_immediate(memory);
        self.execute_addi(memory, size, am, imm)
    }

    fn fast_addq(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (imm, size, am) = self.data_size_effective_address(memory);
        self.execute_addq(memory, imm, size, am)
    }

    fn fast_addx(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (rx, size, mode, ry) = self.register_size_mode_register();
        self.execute_addx(memory, rx, size, mode, ry)
    }

    fn fast_and(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (reg, dir, size, am) = self.register_direction_size_effective_address(memory);
        self.execute_and(memory, reg, dir, size, am)
    }

    fn fast_andi(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (size, am, imm) = self.size_effective_address_immediate(memory);
        self.execute_andi(memory, size, am, imm)
    }

    fn fast_andiccr(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let imm = self.immediate(memory);
        self.execute_andiccr(imm)
    }

    fn fast_andisr(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let imm = self.immediate(memory);
        self.execute_andisr(imm)
    }

    fn fast_asm(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (dir, am) = self.direction_effective_address(memory);
        self.execute_asm(memory, dir, am)
    }

    fn fast_asr(&mut self, _: &mut impl MemoryAccess) -> InterpreterResult {
        let (rot, dir, size, mode, reg) = self.rotation_direction_size_mode_register();
        self.execute_asr(rot, dir, size, mode, reg)
    }

    fn fast_bcc(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let pc = self.regs.pc;
        let (condition, displacement) = self.condition_displacement(memory);
        self.execute_bcc(pc, condition, displacement)
    }

    fn fast_bchg(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (am, count) = self.effective_address_count(memory);
        self.execute_bchg(memory, am, count)
    }

    fn fast_bclr(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (am, count) = self.effective_address_count(memory);
        self.execute_bclr(memory, am, count)
    }

    fn fast_bra(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let pc = self.regs.pc;
        let disp = self.displacement(memory);
        self.execute_bra(pc, disp)
    }

    fn fast_bset(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (am, count) = self.effective_address_count(memory);
        self.execute_bset(memory, am, count)
    }

    fn fast_bsr(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let pc = self.regs.pc;
        let disp = self.displacement(memory);
        self.execute_bsr(memory, pc, disp)
    }

    fn fast_btst(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (am, count) = self.effective_address_count(memory);
        self.execute_btst(memory, am, count)
    }

    /// If a CHK exception occurs, this method returns the effective address calculation time, and the
    /// process_exception method returns the exception processing time.
    fn fast_chk(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (reg, am) = self.register_effective_address(memory);
        self.execute_chk(memory, reg, am)
    }

    fn fast_clr(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (size, am) = self.size_effective_address(memory);
        self.execute_clr(memory, size, am)
    }

    fn fast_cmp(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (reg, _, size, am) = self.register_direction_size_effective_address(memory);
        self.execute_cmp(memory, reg, size, am)
    }

    fn fast_cmpa(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (reg, size, am) = self.register_size_effective_address(memory);
        self.execute_cmpa(memory, reg, size, am)
    }

    fn fast_cmpi(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (size, am, imm) = self.size_effective_address_immediate(memory);
        self.execute_cmpi(memory, size, am, imm)
    }

    fn fast_cmpm(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (ax, size, ay) = self.register_size_register();
        self.execute_cmpm(memory, ax, size, ay)
    }

    fn fast_dbcc(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let pc = self.regs.pc;
        let (cc, reg, disp) = self.condition_register_displacement(memory);
        self.execute_dbcc(pc, cc, reg, disp)
    }

    /// If a zero divide exception occurs, this method returns the effective address calculation time, and the
    /// process_exception method returns the exception processing time.
    fn fast_divs(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (reg, am) = self.register_effective_address(memory);
        self.execute_divs(memory, reg, am)
    }

    /// If a zero divide exception occurs, this method returns the effective address calculation time, and the
    /// process_exception method returns the exception processing time.
    fn fast_divu(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (reg, am) = self.register_effective_address(memory);
        self.execute_divu(memory, reg, am)
    }

    fn fast_eor(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (reg, _, size, am) = self.register_direction_size_effective_address(memory);
        self.execute_eor(memory, reg, size, am)
    }

    fn fast_eori(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (size, am, imm) = self.size_effective_address_immediate(memory);
        self.execute_eori(memory, size, am, imm)
    }

    fn fast_eoriccr(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let imm = self.immediate(memory);
        self.execute_eoriccr(imm)
    }

    fn fast_eorisr(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let imm = self.immediate(memory);
        self.execute_eorisr(imm)
    }

    fn fast_exg(&mut self, _: &mut impl MemoryAccess) -> InterpreterResult {
        let (rx, mode, ry) = self.register_opmode_register();
        self.execute_exg(rx, mode, ry)
    }

    fn fast_ext(&mut self, _: &mut impl MemoryAccess) -> InterpreterResult {
        let (mode, reg) = self.opmode_register();
        self.execute_ext(mode, reg)
    }

    fn fast_illegal(&mut self, _: &mut impl MemoryAccess) -> InterpreterResult {
        self.execute_illegal()
    }

    fn fast_jmp(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let am = self.effective_address(memory);
        self.execute_jmp(am)
    }

    fn fast_jsr(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let am = self.effective_address(memory);
        self.execute_jsr(memory, am)
    }

    fn fast_lea(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (reg, am) = self.register_effective_address(memory);
        self.execute_lea(reg, am)
    }

    fn fast_link(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (reg, disp) = self.register_displacement(memory);
        self.execute_link(memory, reg, disp)
    }

    fn fast_lsm(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (dir, am) = self.direction_effective_address(memory);
        self.execute_lsm(memory, dir, am)
    }

    fn fast_lsr(&mut self, _: &mut impl MemoryAccess) -> InterpreterResult {
        let (rot, dir, size, mode, reg) = self.rotation_direction_size_mode_register();
        self.execute_lsr(rot, dir, size, mode, reg)
    }

    fn fast_move(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (size, amdst, amsrc) = self.size_effective_address_effective_address(memory);
        self.execute_move(memory, size, amdst, amsrc)
    }

    fn fast_movea(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (size, reg, am) = self.size_register_effective_address(memory);
        self.execute_movea(memory, size, reg, am)
    }

    fn fast_moveccr(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let am = self.effective_address(memory);
        self.execute_moveccr(memory, am)
    }

    fn fast_movefsr(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let am = self.effective_address(memory);
        self.execute_movefsr(memory, am)
    }

    fn fast_movesr(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let am = self.effective_address(memory);
        self.execute_movesr(memory, am)
    }

    fn fast_moveusp(&mut self, _: &mut impl MemoryAccess) -> InterpreterResult {
        let (dir, reg) = self.direction_register();
        self.execute_moveusp(dir, reg)
    }

    fn fast_movem(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (dir, size, am, list) = self.direction_size_effective_address_list(memory);
        self.execute_movem(memory, dir, size, am, list)
    }

    fn fast_movep(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (data, dir, size, addr, disp) = self.register_direction_size_register_displacement(memory);
        self.execute_movep(memory, data, dir, size, addr, disp)
    }

    fn fast_moveq(&mut self, _: &mut impl MemoryAccess) -> InterpreterResult {
        let (reg, data) = self.register_data();
        self.execute_moveq(reg, data)
    }

    fn fast_muls(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (reg, am) = self.register_effective_address(memory);
        self.execute_muls(memory, reg, am)
    }

    fn fast_mulu(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (reg, am) = self.register_effective_address(memory);
        self.execute_mulu(memory, reg, am)
    }

    fn fast_nbcd(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let am = self.effective_address(memory);
        self.execute_nbcd(memory, am)
    }

    fn fast_neg(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (size, am) = self.size_effective_address(memory);
        self.execute_neg(memory, size, am)
    }

    fn fast_negx(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (size, am) = self.size_effective_address(memory);
        self.execute_negx(memory, size, am)
    }

    fn fast_nop(&mut self, _: &mut impl MemoryAccess) -> InterpreterResult {
        self.execute_nop()
    }

    fn fast_not(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (size, am) = self.size_effective_address(memory);
        self.execute_not(memory, size, am)
    }

    fn fast_or(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (reg, dir, size, am) = self.register_direction_size_effective_address(memory);
        self.execute_or(memory, reg, dir, size, am)
    }

    fn fast_ori(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (size, am, imm) = self.size_effective_address_immediate(memory);
        self.execute_ori(memory, size, am, imm)
    }

    fn fast_oriccr(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let imm = self.immediate(memory);
        self.execute_oriccr(imm)
    }

    fn fast_orisr(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let imm = self.immediate(memory);
        self.execute_orisr(imm)
    }

    fn fast_pea(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let am = self.effective_address(memory);
        self.execute_pea(memory, am)
    }

    fn fast_reset(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        self.execute_reset(memory)
    }

    fn fast_rom(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (dir, am) = self.direction_effective_address(memory);
        self.execute_rom(memory, dir, am)
    }

    fn fast_ror(&mut self, _: &mut impl MemoryAccess) -> InterpreterResult {
        let (rot, dir, size, mode, reg) = self.rotation_direction_size_mode_register();
        self.execute_ror(rot, dir, size, mode, reg)
    }

    fn fast_roxm(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (dir, am) = self.direction_effective_address(memory);
        self.execute_roxm(memory, dir, am)
    }

    fn fast_roxr(&mut self, _: &mut impl MemoryAccess) -> InterpreterResult {
        let (rot, dir, size, mode, reg) = self.rotation_direction_size_mode_register();
        self.execute_roxr(rot, dir, size, mode, reg)
    }

    fn fast_rte(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        self.execute_rte(memory)
    }

    fn fast_rtr(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        self.execute_rtr(memory)
    }

    fn fast_rts(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        self.execute_rts(memory)
    }

    fn fast_sbcd(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (ry, _, mode, rx) = self.register_size_mode_register();
        self.execute_sbcd(memory, ry, mode, rx)
    }

    fn fast_scc(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (cc, am) = self.condition_effective_address(memory);
        self.execute_scc(memory, cc, am)
    }

    fn fast_stop(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let imm = self.immediate(memory);
        self.execute_stop(imm)
    }

    fn fast_sub(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (reg, dir, size, am) = self.register_direction_size_effective_address(memory);
        self.execute_sub(memory, reg, dir, size, am)
    }

    fn fast_suba(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (reg, size, am) = self.register_size_effective_address(memory);
        self.execute_suba(memory, reg, size, am)
    }

    fn fast_subi(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (size, am, imm) = self.size_effective_address_immediate(memory);
        self.execute_subi(memory, size, am, imm)
    }

    fn fast_subq(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (imm, size, am) = self.data_size_effective_address(memory);
        self.execute_subq(memory, imm, size, am)
    }

    fn fast_subx(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (ry, size, mode, rx) = self.register_size_mode_register();
        self.execute_subx(memory, ry, size, mode, rx)
    }

    fn fast_swap(&mut self, _: &mut impl MemoryAccess) -> InterpreterResult {
        let reg = self.register();
        self.execute_swap(reg)
    }

    fn fast_tas(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let am = self.effective_address(memory);
        self.execute_tas(memory, am)
    }

    fn fast_trap(&mut self, _: &mut impl MemoryAccess) -> InterpreterResult {
        let vector = self.vector();
        self.execute_trap(vector)
    }

    fn fast_trapv(&mut self, _: &mut impl MemoryAccess) -> InterpreterResult {
        self.execute_trapv()
    }

    fn fast_tst(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let (size, am) = self.size_effective_address(memory);
        self.execute_tst(memory, size, am)
    }

    fn fast_unlk(&mut self, memory: &mut impl MemoryAccess) -> InterpreterResult {
        let reg = self.register();
        self.execute_unlk(memory, reg)
    }
}

struct Execute<M: MemoryAccess> {
    _m: M,
}

impl<M: MemoryAccess> Execute<M> {
    /// Function used to execute the instruction.
    const EXECUTE: [fn(&mut M68000, &mut M) -> InterpreterResult; Isa::_Size as usize] = [
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
