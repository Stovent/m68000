use super::M68000;
use super::addressing_modes::{AddressingMode, EffectiveAddress};
use super::memory_access::MemoryAccess;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Operand {
    /// The value itself.
    pub value: i32,
    /// The effective address of the data.
    pub effective_address: EffectiveAddress,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Size {
    Byte = 1,
    Word = 2,
    Long = 4,
}

impl Size {
    /// returns Word when self is Byte, self otherwise.
    ///
    /// This is used in addressing modes, where byte post/pre increment
    /// increments the register by 2 instead of 1.
    pub(super) fn as_word_long(self) -> Self {
        if self == Self::Byte {
            Self::Word
        } else {
            self
        }
    }

    /// Creates a new size from a single size bit of the operand (like MOVEM).
    ///
    /// Size bit means:
    /// - 0 => Word
    /// - 1 => Long
    pub fn from_bit(d: u16) -> Self {
        if d == 0 {
            Self::Word
        } else if d == 1 {
            Self::Long
        } else {
            panic!("[Size::from_bit] Wrong size {}", d)
        }
    }

    /// Creates a new size from the size bits of a MOVE or MOVEA instruction.
    ///
    /// - 1 => Byte
    /// - 3 => Word
    /// - 2 => Long
    pub fn from_move(d: u16) -> Self {
        if d == 1 {
            Self::Byte
        } else if d == 3 {
            Self::Word
        } else if d == 2 {
            Self::Long
        } else {
            panic!("[Size::from_move] Wrong Size {}", d)
        }
    }
}

impl From<u16> for Size {
    /// Creates a new size from the primary size bits.
    ///
    /// Size bits must be:
    /// - 0 => Byte
    /// - 1 => Word
    /// - 2 => Long
    fn from(d: u16) -> Self {
        if d == 0 {
            Self::Byte
        } else if d == 1 {
            Self::Word
        } else if d == 2 {
            Self::Long
        } else {
            panic!("[Size::from<u16>] Wrong size {}", d)
        }
    }
}

impl<M: MemoryAccess> M68000<M> {
    /// Gets an operand.
    pub fn get_operand(&mut self, mode: AddressingMode, reg: usize, size: Size) -> Operand {
        let effective_address = self.get_effective_address(mode, reg, size);
        let value = match mode {
            AddressingMode::Drd => {
                self.d[reg] as i32
            },
            AddressingMode::Ard => {
                self.a(reg) as i32
            },
            AddressingMode::Mode7 => match reg {
                (0..=3) => self.get_value(effective_address.address.unwrap(), size),
                4 => {
                    if size == Size::Byte {
                        self.get_next_word() as i8 as i32
                    } else if size == Size::Word {
                        self.get_next_word() as i16 as i32
                    } else {
                        self.get_next_long() as i32
                    }
                }
                _ => panic!("[M68000::get_operand] Wrong addressing mode {:?} reg {}", mode, reg),
            },
            _ => self.get_value(effective_address.address.unwrap(), size),
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
