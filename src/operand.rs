use super::M68000;
use super::addressing_modes::{AddressingMode, EffectiveAddress};
use super::instruction::Size;
use super::memory_access::MemoryAccess;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Operand {
    /// The value itself.
    pub value: i32,
    /// The effective address of the data.
    pub effective_address: EffectiveAddress,
}

impl<M: MemoryAccess> M68000<M> {
    /// Gets an operand.
    pub(super) fn get_operand(&mut self, mode: AddressingMode, reg: usize, size: Option<Size>) -> Operand {
        let effective_address = self.get_effective_address(mode, reg, size);
        let value = match mode {
            AddressingMode::Drd => {
                self.d[reg] as i32
            },
            AddressingMode::Ard => {
                self.a(reg) as i32
            },
            AddressingMode::Mode7 => match reg {
                (0..=3) => self.get_value(effective_address.address.unwrap(), size.unwrap()),
                4 => {
                    match size.unwrap() {
                        Size::Byte => self.get_next_word() as i8 as i32,
                        Size::Word => self.get_next_word() as i16 as i32,
                        Size::Long => self.get_next_long() as i32,
                    }
                }
                _ => panic!("[M68000::get_operand] Wrong addressing mode {:?} reg {}", mode, reg),
            },
            _ => self.get_value(effective_address.address.unwrap(), size.unwrap()),
        };
        Operand {
            value,
            effective_address,
        }
    }

    fn get_value(&mut self, addr: u32, size: Size) -> i32 {
        match size {
            Size::Byte => self.memory.get_byte(addr) as i8 as i32,
            Size::Word => self.memory.get_word(addr) as i16 as i32,
            Size::Long => self.memory.get_long(addr) as i32,
        }
    }
}
