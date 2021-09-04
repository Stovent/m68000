use super::{M68000, MemoryAccess};
use super::addressing_modes::AddressingMode;
use super::disassembler::DISASSEMBLE;
use super::isa::ISA::Size_;
use super::instruction::Instruction;
use super::operand::Size;
use super::status_register::StatusRegister;
use super::utils::Bits;

impl<M: MemoryAccess> M68000<M> {
    pub fn interpreter(&mut self) {
        // self.current_pc = self.pc;
        // self.current_opcode = self.get_next_word();

        let (inst, width) = Instruction::new::<M>(self.pc, self.memory.get_slice(self.pc));
        self.pc += width;

        #[cfg(debug_assertions)]
        println!("{}", DISASSEMBLE[inst.isa as usize](&inst));
        Self::EXECUTE[inst.isa as usize](self);
    }

    fn unknown_instruction(&mut self) -> usize {
        0
    }

    fn abcd(&mut self) -> usize {
        0
    }

    fn add(&mut self) -> usize {
        let size = self.current_opcode.bits::<6, 7>();
        let eamode = self.current_opcode.bits::<3, 5>();
        let eareg = self.current_opcode.bits::<0, 2>() as usize;
        // if size == Size::Byte {}
        let _operand = self.get_operand(AddressingMode::from(eamode), eareg, Size::from(size));
        0
    }

    fn adda(&mut self) -> usize {
        0
    }

    fn addi(&mut self) -> usize {
        0
    }

    fn addq(&mut self) -> usize {
        0
    }

    fn addx(&mut self) -> usize {
        0
    }

    fn and(&mut self) -> usize {
        0
    }

    fn andi(&mut self) -> usize {
        0
    }

    fn andiccr(&mut self) -> usize {
        0
    }

    fn andisr(&mut self) -> usize {
        0
    }

    fn asm(&mut self) -> usize {
        0
    }

    fn asr(&mut self) -> usize {
        0
    }

    fn bcc(&mut self) -> usize {
        let condition = self.current_opcode.bits::<8, 11>() as usize;
        let mut disp = self.current_opcode as i8 as i16;
        if disp == 0 {
            disp = self.get_next_word() as i16;
        }
        if StatusRegister::CONDITIONS[condition](&self.sr) {
            self.pc = self.current_pc + 2 + disp as u32;
        }
        1
    }

    fn bchg(&mut self) -> usize {
        0
    }

    fn bclr(&mut self) -> usize {
        0
    }

    fn bra(&mut self) -> usize {
        0
    }

    fn bset(&mut self) -> usize {
        0
    }

    fn bsr(&mut self) -> usize {
        0
    }

    fn btst(&mut self) -> usize {
        0
    }

    fn chk(&mut self) -> usize {
        0
    }

    fn clr(&mut self) -> usize {
        0
    }

    fn cmp(&mut self) -> usize {
        0
    }

    fn cmpa(&mut self) -> usize {
        0
    }

    fn cmpi(&mut self) -> usize {
        0
    }

    fn cmpm(&mut self) -> usize {
        0
    }

    fn dbcc(&mut self) -> usize {
        0
    }

    fn divs(&mut self) -> usize {
        0
    }

    fn divu(&mut self) -> usize {
        0
    }

    fn eor(&mut self) -> usize {
        0
    }

    fn eori(&mut self) -> usize {
        0
    }

    fn eoriccr(&mut self) -> usize {
        0
    }

    fn eorisr(&mut self) -> usize {
        0
    }

    fn exg(&mut self) -> usize {
        0
    }

    fn ext(&mut self) -> usize {
        0
    }

    fn illegal(&mut self) -> usize {
        0
    }

    fn jmp(&mut self) -> usize {
        0
    }

    fn jsr(&mut self) -> usize {
        0
    }

    fn lea(&mut self) -> usize {
        0
    }

    fn link(&mut self) -> usize {
        0
    }

    fn lsm(&mut self) -> usize {
        0
    }

    fn lsr(&mut self) -> usize {
        0
    }

    fn r#move(&mut self) -> usize {
        0
    }

    fn movea(&mut self) -> usize {
        0
    }

    fn moveccr(&mut self) -> usize {
        0
    }

    fn movefsr(&mut self) -> usize {
        0
    }

    fn movesr(&mut self) -> usize {
        0
    }

    fn moveusp(&mut self) -> usize {
        0
    }

    fn movem(&mut self) -> usize {
        0
    }

    fn movep(&mut self) -> usize {
        0
    }

    fn moveq(&mut self) -> usize {
        0
    }

    fn muls(&mut self) -> usize {
        0
    }

    fn mulu(&mut self) -> usize {
        0
    }

    fn nbcd(&mut self) -> usize {
        0
    }

    fn neg(&mut self) -> usize {
        0
    }

    fn negx(&mut self) -> usize {
        0
    }

    fn nop(&mut self) -> usize {
        1
    }

    fn not(&mut self) -> usize {
        0
    }

    fn or(&mut self) -> usize {
        0
    }

    fn ori(&mut self) -> usize {
        0
    }

    fn oriccr(&mut self) -> usize {
        0
    }

    fn orisr(&mut self) -> usize {
        0
    }

    fn pea(&mut self) -> usize {
        0
    }

    fn reset(&mut self) -> usize {
        0
    }

    fn rom(&mut self) -> usize {
        0
    }

    fn ror(&mut self) -> usize {
        0
    }

    fn roxm(&mut self) -> usize {
        0
    }

    fn roxr(&mut self) -> usize {
        0
    }

    fn rte(&mut self) -> usize {
        0
    }

    fn rtr(&mut self) -> usize {
        0
    }

    fn rts(&mut self) -> usize {
        0
    }

    fn sbcd(&mut self) -> usize {
        0
    }

    fn scc(&mut self) -> usize {
        0
    }

    fn stop(&mut self) -> usize {
        0
    }

    fn sub(&mut self) -> usize {
        0
    }

    fn suba(&mut self) -> usize {
        0
    }

    fn subi(&mut self) -> usize {
        0
    }

    fn subq(&mut self) -> usize {
        0
    }

    fn subx(&mut self) -> usize {
        0
    }

    fn swap(&mut self) -> usize {
        0
    }

    fn tas(&mut self) -> usize {
        0
    }

    fn trap(&mut self) -> usize {
        0
    }

    fn trapv(&mut self) -> usize {
        0
    }

    fn tst(&mut self) -> usize {
        0
    }

    fn unlk(&mut self) -> usize {
        0
    }

    const EXECUTE: [fn(&mut Self) -> usize; Size_ as usize] = [
        Self::unknown_instruction,
        Self::abcd,
        Self::add,
        Self::adda,
        Self::addi,
        Self::addq,
        Self::addx,
        Self::and,
        Self::andi,
        Self::andiccr,
        Self::andisr,
        Self::asm,
        Self::asr,
        Self::bcc,
        Self::bchg,
        Self::bclr,
        Self::bra,
        Self::bset,
        Self::bsr,
        Self::btst,
        Self::chk,
        Self::clr,
        Self::cmp,
        Self::cmpa,
        Self::cmpi,
        Self::cmpm,
        Self::dbcc,
        Self::divs,
        Self::divu,
        Self::eor,
        Self::eori,
        Self::eoriccr,
        Self::eorisr,
        Self::exg,
        Self::ext,
        Self::illegal,
        Self::jmp,
        Self::jsr,
        Self::lea,
        Self::link,
        Self::lsm,
        Self::lsr,
        Self::r#move,
        Self::movea,
        Self::moveccr,
        Self::movefsr,
        Self::movesr,
        Self::moveusp,
        Self::movem,
        Self::movep,
        Self::moveq,
        Self::muls,
        Self::mulu,
        Self::nbcd,
        Self::neg,
        Self::negx,
        Self::nop,
        Self::not,
        Self::or,
        Self::ori,
        Self::oriccr,
        Self::orisr,
        Self::pea,
        Self::reset,
        Self::rom,
        Self::ror,
        Self::roxm,
        Self::roxr,
        Self::rte,
        Self::rtr,
        Self::rts,
        Self::sbcd,
        Self::scc,
        Self::stop,
        Self::sub,
        Self::suba,
        Self::subi,
        Self::subq,
        Self::subx,
        Self::swap,
        Self::tas,
        Self::trap,
        Self::trapv,
        Self::tst,
        Self::unlk,
    ];
}
