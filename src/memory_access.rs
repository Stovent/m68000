use super::M68000;

pub trait MemoryAccess {
    fn iter(&mut self, addr: u32) -> MemoryIter;
    fn get_byte(&mut self, addr: u32) -> u8;
    fn get_word(&mut self, addr: u32) -> u16;
    fn get_long(&mut self, addr: u32) -> u32;

    fn set_byte(&mut self, addr: u32, value: u8);
    fn set_word(&mut self, addr: u32, value: u16);
    fn set_long(&mut self, addr: u32, value: u32);
}

pub struct MemoryIter<'a> {
    pub cpu: &'a mut dyn MemoryAccess,
    pub next_addr: u32,
}

impl<'a> Iterator for MemoryIter<'a> {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        let data = self.cpu.get_word(self.next_addr);
        self.next_addr += 2;
        Some(data)
    }
}

impl<M: MemoryAccess> M68000<M> {
    pub(super) fn get_next_word(&mut self) -> u16 {
        let data = self.memory.get_word(self.pc);
        // println!("[get_next_word] read {} {:#X} at {:#X}", data, data, self.pc);
        self.pc += 2;
        data
    }

    // pub(super) fn get_next_long(&mut self) -> u32 {
    //     let data = self.memory.get_long(self.pc);
    //     println!("[get_next_long] read {} {:#X} at {:#X}", data, data, self.pc);
    //     self.pc += 4;
    //     data
    // }
}
