use std::fmt::Display;

pub struct Bindings<const SIZE: usize> {
    pub buttons: [u8; SIZE],
}
impl<const SIZE: usize> Bindings<SIZE> {
    pub fn new() -> Self {
        Self { buttons: [0; SIZE] }
    }

    pub fn set(&mut self, key: usize) {
        let i = key / 8;
        let byte = key % 8;
        self.buttons[i] |= 1 << byte;
    }

    pub fn unset(&mut self, key: usize) {
        let i = key / 8;
        let byte = key % 8;
        self.buttons[i] &= !(1 << byte);
    }

    pub fn reset(&mut self) {
        self.buttons = [0; SIZE];
    }

    pub fn down(&self, key: usize) -> bool {
        let i = key / 8;
        let byte = key % 8;
        self.buttons[i] & 1 << byte > 0
    }
}
impl<const SIZE: usize> Display for Bindings<SIZE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "|")?;
        for i in 0..SIZE {
            write!(f, "{:08b}|", self.buttons[i].reverse_bits())?;
        }
        Ok(())
    }
}

pub type KeyboardBinding = Bindings<25>;
pub type MouseBinding = Bindings<1>;
