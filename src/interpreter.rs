use super::{M68000, MemoryAccess};
use super::decoder::DECODER;
use super::instruction::Instruction;
use super::operands::Operands;
use super::status_register::StatusRegister;

impl<M: MemoryAccess> M68000<M> {
    pub fn interpreter(&mut self) {
        let pc = self.pc;
        let opcode = self.get_next_word();
        let isa = DECODER[opcode as usize];
        let entry = &Self::ISA_ENTRY[isa as usize];

        let mut iter = self.memory.iter(self.pc);
        let (operands, len) = (entry.decode)(opcode, &mut iter);
        self.pc += len as u32;

        let instruction = Instruction {
            isa,
            opcode,
            pc,
            operands,
        };

        #[cfg(debug_assertions)]
        println!("{}", (entry.disassemble)(&instruction));

        (entry.execute)(self, &instruction);
    }

    pub(super) fn unknown_instruction(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn abcd(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn add(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn adda(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn addi(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn addq(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn addx(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn and(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn andi(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn andiccr(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn andisr(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn asm(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn asr(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn bcc(&mut self, inst: &Instruction) -> usize {
        let (condition, mut displacement) = match inst.operands {
            Operands::ConditionDisplacement(c, d) => (c, d as i16),
            _ => panic!("Wrong operands enum for Bcc"),
        };
        if displacement == 0 {
            displacement = self.get_next_word() as i16;
        }
        if StatusRegister::CONDITIONS[condition as usize](&self.sr) {
            self.pc = inst.pc + 2 + displacement as u32;
        }
        1
    }

    pub(super) fn bchg(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn bclr(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn bra(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn bset(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn bsr(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn btst(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn chk(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn clr(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn cmp(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn cmpa(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn cmpi(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn cmpm(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn dbcc(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn divs(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn divu(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn eor(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn eori(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn eoriccr(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn eorisr(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn exg(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn ext(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn illegal(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn jmp(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn jsr(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn lea(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn link(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn lsm(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn lsr(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn r#move(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn movea(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn moveccr(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn movefsr(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn movesr(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn moveusp(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn movem(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn movep(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn moveq(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn muls(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn mulu(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn nbcd(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn neg(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn negx(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn nop(&mut self, _: &Instruction) -> usize {
        1
    }

    pub(super) fn not(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn or(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn ori(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn oriccr(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn orisr(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn pea(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn reset(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn rom(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn ror(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn roxm(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn roxr(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn rte(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn rtr(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn rts(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn sbcd(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn scc(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn stop(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn sub(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn suba(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn subi(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn subq(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn subx(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn swap(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn tas(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn trap(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn trapv(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn tst(&mut self, inst: &Instruction) -> usize {
        0
    }

    pub(super) fn unlk(&mut self, inst: &Instruction) -> usize {
        0
    }
}
