// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{CpuDetails, M68000, MemoryAccess};
use crate::exception::{Exception, Vector};
use crate::instruction::Instruction;
use crate::interpreter::InterpreterResult;
use crate::isa::Isa;

impl<CPU: CpuDetails> M68000<CPU> {
    /// Returns the instruction at the current Program Counter and advances it to the next instruction.
    ///
    /// If an error occurs when reading the next instruction, the Err variant contains the exception vector.
    fn get_next_instruction<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> Result<Instruction, u8> {
        let mut iter = self.iter_from_pc(memory);
        let instruction = Instruction::from_memory(&mut iter)?;
        self.regs.pc.0 = iter.next_addr;
        Ok(instruction)
    }

    /// Runs the interpreter loop once and disassembles the next instruction if any.
    ///
    /// Returns the disassembled string and the cycle count necessary to execute it.
    ///
    /// See [Self::interpreter] for the potential caveat.
    pub fn disassembler_interpreter<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> (String, usize) {
        let (dis, cycles, exception) = self.disassembler_interpreter_exception(memory);
        if let Some(e) = exception {
            self.exception(Exception::from(e));
        }
        (dis, cycles)
    }

    /// Runs the interpreter loop once and disassembles the next instruction if any.
    ///
    /// Returns the disassembled string, the cycle count necessary to execute it, and the vector of the exception that
    /// occured during the execution if any.
    ///
    /// To process the returned exception, call [M68000::exception].
    ///
    /// See [Self::interpreter_exception] for the potential caveat.
    pub fn disassembler_interpreter_exception<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M) -> (String, usize, Option<u8>) {
        if self.stop {
            return (String::from(""), 0, None);
        }

        let mut cycle_count = 0;

        if !self.exceptions.is_empty() {
            cycle_count += self.process_pending_exceptions(memory);
        }

        let instruction = match self.get_next_instruction(memory) {
            Ok(i) => i,
            Err(e) => return (String::from(""), cycle_count, Some(e)),
        };

        self.current_opcode = instruction.opcode;
        let isa = Isa::from(instruction.opcode);

        let dis = instruction.disassemble();
        let trace = self.regs.sr.t;
        let exception = match Execute::<CPU, M>::EXECUTE[isa as usize](self, memory, &instruction) {
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

        (dis, cycle_count, exception)
    }

    fn instruction_unknown_instruction<M: MemoryAccess + ?Sized>(&mut self, _: &mut M, _: &Instruction) -> InterpreterResult {
        self.execute_unknown_instruction()
    }

    fn instruction_abcd<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (rx, _, mode, ry) = inst.operands.register_size_mode_register();
        self.execute_abcd(memory, rx, mode, ry)
    }

    fn instruction_add<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (reg, dir, size, am) = inst.operands.register_direction_size_effective_address();
        self.execute_add(memory, reg, dir, size, am)
    }

    fn instruction_adda<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (reg, size, am) = inst.operands.register_size_effective_address();
        self.execute_adda(memory, reg, size, am)
    }

    fn instruction_addi<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (size, am, imm) = inst.operands.size_effective_address_immediate();
        self.execute_addi(memory, size, am, imm)
    }

    fn instruction_addq<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (imm, size, am) = inst.operands.data_size_effective_address();
        self.execute_addq(memory, imm, size, am)
    }

    fn instruction_addx<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (rx, size, mode, ry) = inst.operands.register_size_mode_register();
        self.execute_addx(memory, rx, size, mode, ry)
    }

    fn instruction_and<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (reg, dir, size, am) = inst.operands.register_direction_size_effective_address();
        self.execute_and(memory, reg, dir, size, am)
    }

    fn instruction_andi<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (size, am, imm) = inst.operands.size_effective_address_immediate();
        self.execute_andi(memory, size, am, imm)
    }

    fn instruction_andiccr<M: MemoryAccess + ?Sized>(&mut self, _: &mut M, inst: &Instruction) -> InterpreterResult {
        let imm = inst.operands.immediate();
        self.execute_andiccr(imm)
    }

    fn instruction_andisr<M: MemoryAccess + ?Sized>(&mut self, _: &mut M, inst: &Instruction) -> InterpreterResult {
        let imm = inst.operands.immediate();
        self.execute_andisr(imm)
    }

    fn instruction_asm<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (dir, am) = inst.operands.direction_effective_address();
        self.execute_asm(memory, dir, am)
    }

    fn instruction_asr<M: MemoryAccess + ?Sized>(&mut self, _: &mut M, inst: &Instruction) -> InterpreterResult {
        let (rot, dir, size, mode, reg) = inst.operands.rotation_direction_size_mode_register();
        self.execute_asr(rot, dir, size, mode, reg)
    }

    fn instruction_bcc<M: MemoryAccess + ?Sized>(&mut self, _: &mut M, inst: &Instruction) -> InterpreterResult {
        let (condition, displacement) = inst.operands.condition_displacement();
        self.execute_bcc(inst.pc.wrapping_add(2), condition, displacement)
    }

    fn instruction_bchg<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (am, count) = inst.operands.effective_address_count();
        self.execute_bchg(memory, am, count)
    }

    fn instruction_bclr<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (am, count) = inst.operands.effective_address_count();
        self.execute_bclr(memory, am, count)
    }

    fn instruction_bra<M: MemoryAccess + ?Sized>(&mut self, _: &mut M, inst: &Instruction) -> InterpreterResult {
        let disp = inst.operands.displacement();
        self.execute_bra(inst.pc.wrapping_add(2), disp)
    }

    fn instruction_bset<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (am, count) = inst.operands.effective_address_count();
        self.execute_bset(memory, am, count)
    }

    fn instruction_bsr<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let disp = inst.operands.displacement();
        self.execute_bsr(memory, inst.pc.wrapping_add(2), disp)
    }

    fn instruction_btst<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (am, count) = inst.operands.effective_address_count();
        self.execute_btst(memory, am, count)
    }

    /// If a CHK exception occurs, this method returns the effective address calculation time, and the
    /// process_exception method returns the exception processing time.
    fn instruction_chk<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (reg, am) = inst.operands.register_effective_address();
        self.execute_chk(memory, reg, am)
    }

    fn instruction_clr<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (size, am) = inst.operands.size_effective_address();
        self.execute_clr(memory, size, am)
    }

    fn instruction_cmp<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (reg, _, size, am) = inst.operands.register_direction_size_effective_address();
        self.execute_cmp(memory, reg, size, am)
    }

    fn instruction_cmpa<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (reg, size, am) = inst.operands.register_size_effective_address();
        self.execute_cmpa(memory, reg, size, am)
    }

    fn instruction_cmpi<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (size, am, imm) = inst.operands.size_effective_address_immediate();
        self.execute_cmpi(memory, size, am, imm)
    }

    fn instruction_cmpm<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (ax, size, ay) = inst.operands.register_size_register();
        self.execute_cmpm(memory, ax, size, ay)
    }

    fn instruction_dbcc<M: MemoryAccess + ?Sized>(&mut self, _: &mut M, inst: &Instruction) -> InterpreterResult {
        let (cc, reg, disp) = inst.operands.condition_register_displacement();
        self.execute_dbcc(inst.pc.wrapping_add(2), cc, reg, disp)
    }

    /// If a zero divide exception occurs, this method returns the effective address calculation time, and the
    /// process_exception method returns the exception processing time.
    fn instruction_divs<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (reg, am) = inst.operands.register_effective_address();
        self.execute_divs(memory, reg, am)
    }

    /// If a zero divide exception occurs, this method returns the effective address calculation time, and the
    /// process_exception method returns the exception processing time.
    fn instruction_divu<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (reg, am) = inst.operands.register_effective_address();
        self.execute_divu(memory, reg, am)
    }

    fn instruction_eor<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (reg, _, size, am) = inst.operands.register_direction_size_effective_address();
        self.execute_eor(memory, reg, size, am)
    }

    fn instruction_eori<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (size, am, imm) = inst.operands.size_effective_address_immediate();
        self.execute_eori(memory, size, am, imm)
    }

    fn instruction_eoriccr<M: MemoryAccess + ?Sized>(&mut self, _: &mut M, inst: &Instruction) -> InterpreterResult {
        let imm = inst.operands.immediate();
        self.execute_eoriccr(imm)
    }

    fn instruction_eorisr<M: MemoryAccess + ?Sized>(&mut self, _: &mut M, inst: &Instruction) -> InterpreterResult {
        let imm = inst.operands.immediate();
        self.execute_eorisr(imm)
    }

    fn instruction_exg<M: MemoryAccess + ?Sized>(&mut self, _: &mut M, inst: &Instruction) -> InterpreterResult {
        let (rx, mode, ry) = inst.operands.register_opmode_register();
        self.execute_exg(rx, mode, ry)
    }

    fn instruction_ext<M: MemoryAccess + ?Sized>(&mut self, _: &mut M, inst: &Instruction) -> InterpreterResult {
        let (mode, reg) = inst.operands.opmode_register();
        self.execute_ext(mode, reg)
    }

    fn instruction_illegal<M: MemoryAccess + ?Sized>(&mut self, _: &mut M, _: &Instruction) -> InterpreterResult {
        self.execute_illegal()
    }

    fn instruction_jmp<M: MemoryAccess + ?Sized>(&mut self, _: &mut M, inst: &Instruction) -> InterpreterResult {
        let am = inst.operands.effective_address();
        self.execute_jmp(am)
    }

    fn instruction_jsr<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let am = inst.operands.effective_address();
        self.execute_jsr(memory, am)
    }

    fn instruction_lea<M: MemoryAccess + ?Sized>(&mut self, _: &mut M, inst: &Instruction) -> InterpreterResult {
        let (reg, am) = inst.operands.register_effective_address();
        self.execute_lea(reg, am)
    }

    fn instruction_link<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (reg, disp) = inst.operands.register_displacement();
        self.execute_link(memory, reg, disp)
    }

    fn instruction_lsm<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (dir, am) = inst.operands.direction_effective_address();
        self.execute_lsm(memory, dir, am)
    }

    fn instruction_lsr<M: MemoryAccess + ?Sized>(&mut self, _: &mut M, inst: &Instruction) -> InterpreterResult {
        let (rot, dir, size, mode, reg) = inst.operands.rotation_direction_size_mode_register();
        self.execute_lsr(rot, dir, size, mode, reg)
    }

    fn instruction_move<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (size, amdst, amsrc) = inst.operands.size_effective_address_effective_address();
        self.execute_move(memory, size, amdst, amsrc)
    }

    fn instruction_movea<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (size, reg, am) = inst.operands.size_register_effective_address();
        self.execute_movea(memory, size, reg, am)
    }

    fn instruction_moveccr<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let am = inst.operands.effective_address();
        self.execute_moveccr(memory, am)
    }

    fn instruction_movefsr<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let am = inst.operands.effective_address();
        self.execute_movefsr(memory, am)
    }

    fn instruction_movesr<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let am = inst.operands.effective_address();
        self.execute_movesr(memory, am)
    }

    fn instruction_moveusp<M: MemoryAccess + ?Sized>(&mut self, _: &mut M, inst: &Instruction) -> InterpreterResult {
        let (dir, reg) = inst.operands.direction_register();
        self.execute_moveusp(dir, reg)
    }

    fn instruction_movem<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (dir, size, am, list) = inst.operands.direction_size_effective_address_list();
        self.execute_movem(memory, dir, size, am, list)
    }

    fn instruction_movep<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (data, dir, size, addr, disp) = inst.operands.register_direction_size_register_displacement();
        self.execute_movep(memory, data, dir, size, addr, disp)
    }

    fn instruction_moveq<M: MemoryAccess + ?Sized>(&mut self, _: &mut M, inst: &Instruction) -> InterpreterResult {
        let (reg, data) = inst.operands.register_data();
        self.execute_moveq(reg, data)
    }

    fn instruction_muls<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (reg, am) = inst.operands.register_effective_address();
        self.execute_muls(memory, reg, am)
    }

    fn instruction_mulu<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (reg, am) = inst.operands.register_effective_address();
        self.execute_mulu(memory, reg, am)
    }

    fn instruction_nbcd<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let am = inst.operands.effective_address();
        self.execute_nbcd(memory, am)
    }

    fn instruction_neg<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (size, am) = inst.operands.size_effective_address();
        self.execute_neg(memory, size, am)
    }

    fn instruction_negx<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (size, am) = inst.operands.size_effective_address();
        self.execute_negx(memory, size, am)
    }

    fn instruction_nop<M: MemoryAccess + ?Sized>(&mut self, _: &mut M, _: &Instruction) -> InterpreterResult {
        self.execute_nop()
    }

    fn instruction_not<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (size, am) = inst.operands.size_effective_address();
        self.execute_not(memory, size, am)
    }

    fn instruction_or<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (reg, dir, size, am) = inst.operands.register_direction_size_effective_address();
        self.execute_or(memory, reg, dir, size, am)
    }

    fn instruction_ori<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (size, am, imm) = inst.operands.size_effective_address_immediate();
        self.execute_ori(memory, size, am, imm)
    }

    fn instruction_oriccr<M: MemoryAccess + ?Sized>(&mut self, _: &mut M, inst: &Instruction) -> InterpreterResult {
        let imm = inst.operands.immediate();
        self.execute_oriccr(imm)
    }

    fn instruction_orisr<M: MemoryAccess + ?Sized>(&mut self, _: &mut M, inst: &Instruction) -> InterpreterResult {
        let imm = inst.operands.immediate();
        self.execute_orisr(imm)
    }

    fn instruction_pea<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let am = inst.operands.effective_address();
        self.execute_pea(memory, am)
    }

    fn instruction_reset<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, _: &Instruction) -> InterpreterResult {
        self.execute_reset(memory)
    }

    fn instruction_rom<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (dir, am) = inst.operands.direction_effective_address();
        self.execute_rom(memory, dir, am)
    }

    fn instruction_ror<M: MemoryAccess + ?Sized>(&mut self, _: &mut M, inst: &Instruction) -> InterpreterResult {
        let (rot, dir, size, mode, reg) = inst.operands.rotation_direction_size_mode_register();
        self.execute_ror(rot, dir, size, mode, reg)
    }

    fn instruction_roxm<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (dir, am) = inst.operands.direction_effective_address();
        self.execute_roxm(memory, dir, am)
    }

    fn instruction_roxr<M: MemoryAccess + ?Sized>(&mut self, _: &mut M, inst: &Instruction) -> InterpreterResult {
        let (rot, dir, size, mode, reg) = inst.operands.rotation_direction_size_mode_register();
        self.execute_roxr(rot, dir, size, mode, reg)
    }

    fn instruction_rte<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, _: &Instruction) -> InterpreterResult {
        self.execute_rte(memory)
    }

    fn instruction_rtr<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, _: &Instruction) -> InterpreterResult {
        self.execute_rtr(memory)
    }

    fn instruction_rts<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, _: &Instruction) -> InterpreterResult {
        self.execute_rts(memory)
    }

    fn instruction_sbcd<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (ry, _, mode, rx) = inst.operands.register_size_mode_register();
        self.execute_sbcd(memory, ry, mode, rx)
    }

    fn instruction_scc<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (cc, am) = inst.operands.condition_effective_address();
        self.execute_scc(memory, cc, am)
    }

    fn instruction_stop<M: MemoryAccess + ?Sized>(&mut self, _: &mut M, inst: &Instruction) -> InterpreterResult {
        let imm = inst.operands.immediate();
        self.execute_stop(imm)
    }

    fn instruction_sub<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (reg, dir, size, am) = inst.operands.register_direction_size_effective_address();
        self.execute_sub(memory, reg, dir, size, am)
    }

    fn instruction_suba<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (reg, size, am) = inst.operands.register_size_effective_address();
        self.execute_suba(memory, reg, size, am)
    }

    fn instruction_subi<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (size, am, imm) = inst.operands.size_effective_address_immediate();
        self.execute_subi(memory, size, am, imm)
    }

    fn instruction_subq<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (imm, size, am) = inst.operands.data_size_effective_address();
        self.execute_subq(memory, imm, size, am)
    }

    fn instruction_subx<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (ry, size, mode, rx) = inst.operands.register_size_mode_register();
        self.execute_subx(memory, ry, size, mode, rx)
    }

    fn instruction_swap<M: MemoryAccess + ?Sized>(&mut self, _: &mut M, inst: &Instruction) -> InterpreterResult {
        let reg = inst.operands.register();
        self.execute_swap(reg)
    }

    fn instruction_tas<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let am = inst.operands.effective_address();
        self.execute_tas(memory, am)
    }

    fn instruction_trap<M: MemoryAccess + ?Sized>(&mut self, _: &mut M, inst: &Instruction) -> InterpreterResult {
        let vector = inst.operands.vector();
        self.execute_trap(vector)
    }

    fn instruction_trapv<M: MemoryAccess + ?Sized>(&mut self, _: &mut M, _: &Instruction) -> InterpreterResult {
        self.execute_trapv()
    }

    fn instruction_tst<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let (size, am) = inst.operands.size_effective_address();
        self.execute_tst(memory, size, am)
    }

    fn instruction_unlk<M: MemoryAccess + ?Sized>(&mut self, memory: &mut M, inst: &Instruction) -> InterpreterResult {
        let reg = inst.operands.register();
        self.execute_unlk(memory, reg)
    }
}

struct Execute<E: CpuDetails, M: MemoryAccess + ?Sized> {
    _e: E,
    _m: M,
}

impl<E: CpuDetails, M: MemoryAccess + ?Sized> Execute<E, M> {
    /// Function used to execute the instruction.
    const EXECUTE: [fn(&mut M68000<E>, &mut M, &Instruction) -> InterpreterResult; Isa::_Size as usize] = [
        M68000::instruction_unknown_instruction,
        M68000::instruction_abcd,
        M68000::instruction_add,
        M68000::instruction_adda,
        M68000::instruction_addi,
        M68000::instruction_addq,
        M68000::instruction_addx,
        M68000::instruction_and,
        M68000::instruction_andi,
        M68000::instruction_andiccr,
        M68000::instruction_andisr,
        M68000::instruction_asm,
        M68000::instruction_asr,
        M68000::instruction_bcc,
        M68000::instruction_bchg,
        M68000::instruction_bclr,
        M68000::instruction_bra,
        M68000::instruction_bset,
        M68000::instruction_bsr,
        M68000::instruction_btst,
        M68000::instruction_chk,
        M68000::instruction_clr,
        M68000::instruction_cmp,
        M68000::instruction_cmpa,
        M68000::instruction_cmpi,
        M68000::instruction_cmpm,
        M68000::instruction_dbcc,
        M68000::instruction_divs,
        M68000::instruction_divu,
        M68000::instruction_eor,
        M68000::instruction_eori,
        M68000::instruction_eoriccr,
        M68000::instruction_eorisr,
        M68000::instruction_exg,
        M68000::instruction_ext,
        M68000::instruction_illegal,
        M68000::instruction_jmp,
        M68000::instruction_jsr,
        M68000::instruction_lea,
        M68000::instruction_link,
        M68000::instruction_lsm,
        M68000::instruction_lsr,
        M68000::instruction_move,
        M68000::instruction_movea,
        M68000::instruction_moveccr,
        M68000::instruction_movefsr,
        M68000::instruction_movesr,
        M68000::instruction_moveusp,
        M68000::instruction_movem,
        M68000::instruction_movep,
        M68000::instruction_moveq,
        M68000::instruction_muls,
        M68000::instruction_mulu,
        M68000::instruction_nbcd,
        M68000::instruction_neg,
        M68000::instruction_negx,
        M68000::instruction_nop,
        M68000::instruction_not,
        M68000::instruction_or,
        M68000::instruction_ori,
        M68000::instruction_oriccr,
        M68000::instruction_orisr,
        M68000::instruction_pea,
        M68000::instruction_reset,
        M68000::instruction_rom,
        M68000::instruction_ror,
        M68000::instruction_roxm,
        M68000::instruction_roxr,
        M68000::instruction_rte,
        M68000::instruction_rtr,
        M68000::instruction_rts,
        M68000::instruction_sbcd,
        M68000::instruction_scc,
        M68000::instruction_stop,
        M68000::instruction_sub,
        M68000::instruction_suba,
        M68000::instruction_subi,
        M68000::instruction_subq,
        M68000::instruction_subx,
        M68000::instruction_swap,
        M68000::instruction_tas,
        M68000::instruction_trap,
        M68000::instruction_trapv,
        M68000::instruction_tst,
        M68000::instruction_unlk,
    ];
}
