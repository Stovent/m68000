use super::operands::Size;

/// Returns bits [beg, end] inclusive, starting at 0.
pub const fn bits(d: u16, beg: u16, end: u16) -> u16 {
    let mask = (1 << end + 1 - beg) - 1;
    d >> beg & mask
}

/// Trait that convert the first bytes of slices to their big-endian integer value.
pub trait SliceAs {
    /// Returns self[0] and self[1] as if it is a big-endian 16 bits integer.
    fn u16_be(self) -> u16;
    /// Returns self[0], self[1], self[2] and self[3] as if it is a big-endian 32 bits integer.
    fn u32_be(self) -> u32;
    /// Returns self[1] for Byte size, self.u16_be() for Word size and self.u32_be() for Long size.
    fn i32_be_sized(self, size: Size) -> i32;
    /// Return self.u16_be() then makes self advance by two bytes in the slice.
    fn get_next_word(&mut self) -> u16;
}

impl SliceAs for &[u8] {
    fn u16_be(self) -> u16 {
        (self[0] as u16) << 8 | self[1] as u16
    }

    fn u32_be(self) -> u32 {
        (self[0] as u32) << 24 | (self[1] as u32) << 16 | (self[2] as u32) << 8 | self[3] as u32
    }

    fn i32_be_sized(self, size: Size) -> i32 {
        match size {
            Size::Byte => self[1] as i8 as i32,
            Size::Word => self.u16_be() as i16 as i32,
            Size::Long => self.u32_be() as i32,
        }
    }

    fn get_next_word(&mut self) -> u16 {
        let d = self.u16_be();
        *self = &self[2..];
        d
    }
}

/// Converts integers to their big-endian array.
pub trait AsArray<const N: usize> {
    fn as_array_be(self) -> [u8; N];
}

impl AsArray<2> for u16 {
    fn as_array_be(self) -> [u8; 2] {
        [(self >> 8) as u8, self as u8]
    }
}

impl AsArray<4> for u32 {
    fn as_array_be(self) -> [u8; 4] {
        [(self >> 24) as u8, (self >> 16) as u8, (self >> 8) as u8, self as u8]
    }
}
