#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy)]
pub enum Interrupt {
    VBlank = 0x40,
    STAT = 0x48,
    Timer = 0x50,
    Serial = 0x58,
    Joypad = 0x60,
}

#[derive(Default)]
pub struct InterruptHandler {
    pub inte: u8,
    pub intf: u8,
}

impl InterruptHandler {
    pub fn get_enabled_interrupts(&self) -> [Option<Interrupt>; 5] {
        let vblank = ((self.inte & 0b1) != 0).then_some(Interrupt::VBlank);
        let stat = ((self.inte & 0b10) != 0).then_some(Interrupt::STAT);
        let timer = ((self.inte & 0b100) != 0).then_some(Interrupt::Timer);
        let serial = ((self.inte & 0b1000) != 0).then_some(Interrupt::Serial);
        let joypad = ((self.inte & 0b10000) != 0).then_some(Interrupt::Joypad);

        [vblank, stat, timer, serial, joypad]
    }

    pub fn is_interrupt_requested(&self, interrupt: Interrupt) -> bool {
        match interrupt {
            Interrupt::VBlank => self.intf & 0b1 != 0,
            Interrupt::STAT => self.intf & 0b10 != 0,
            Interrupt::Timer => self.intf & 0b100 != 0,
            Interrupt::Serial => self.intf & 0b1000 != 0,
            Interrupt::Joypad => self.intf & 0b10000 != 0,
        }
    }

    pub fn reset_if(&mut self, interrupt: Interrupt) {
        match interrupt {
            Interrupt::VBlank => self.intf &= !(0b1),
            Interrupt::STAT => self.intf &= !(0b10),
            Interrupt::Timer => self.intf &= !(0b100),
            Interrupt::Serial => self.intf &= !(0b1000),
            Interrupt::Joypad => self.intf &= !(0b10000),
        }
    }

    pub fn request_interrupt(&mut self, interrupt: Interrupt) {
        match interrupt {
            Interrupt::VBlank => self.intf |= 0b1,
            Interrupt::STAT => self.intf |= 0b10,
            Interrupt::Timer => self.intf |= 0b100,
            Interrupt::Serial => self.intf |= 0b1000,
            Interrupt::Joypad => self.intf |= 0b10000,
        }
    }
}
