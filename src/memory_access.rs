use super::M68000;

pub trait MemoryAccess {
    fn get_slice(&mut self, addr: u32) -> &[u8];
    fn get_byte(&mut self, addr: u32) -> u8;
    fn get_word(&mut self, addr: u32) -> u16;
    fn get_long(&mut self, addr: u32) -> u32;

    fn set_byte(&mut self, addr: u32, value: u8);
    fn set_word(&mut self, addr: u32, value: u16);
    fn set_long(&mut self, addr: u32, value: u32);
}

impl<M: MemoryAccess> M68000<M> {
    pub(super) fn get_next_word(&mut self) -> u16 {
        let data = self.memory.get_word(self.pc);
        self.pc += 2;
        data
    }

    pub(super) fn get_next_long(&mut self) -> u32 {
        let data = self.memory.get_long(self.pc);
        self.pc += 4;
        data
    }
}
