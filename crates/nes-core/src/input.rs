//! Standard serial controller; buttons use A/B/Select/Start/Up/Down/Left/Right.
#[derive(Default)]
pub struct Controller {
    pub buttons: u8,
    shift: u8,
    strobe: bool,
}
impl Controller {
    pub fn set_buttons(&mut self, buttons: u8) {
        self.buttons = buttons;
        if self.strobe {
            self.shift = buttons;
        }
    }
    pub fn write(&mut self, v: u8) {
        self.strobe = v & 1 != 0;
        if self.strobe {
            self.shift = self.buttons;
        }
    }
    pub fn read(&mut self) -> u8 {
        if self.strobe {
            return self.buttons & 1;
        }
        let bit = self.shift & 1;
        self.shift = (self.shift >> 1) | 0x80;
        bit
    }
}
