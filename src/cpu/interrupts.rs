pub enum Interrupt {
    VBlank = 0x40,
    STAT = 0x48,
    Timer = 0x50,
    Serial = 0x58,
    Joypad = 0x60,
}

pub fn get_enabled_interrupts(ie: u8) -> [Option<Interrupt>; 5] {
    let vblank = ((ie & 0b1) != 0).then_some(Interrupt::VBlank);
    let stat = ((ie & 0b10) != 0).then_some(Interrupt::STAT);
    let timer = ((ie & 0b100) != 0).then_some(Interrupt::Timer);
    let serial = ((ie & 0b1000) != 0).then_some(Interrupt::Serial);
    let joypad = ((ie & 0b10000) != 0).then_some(Interrupt::Joypad);

    [vblank, stat, timer, serial, joypad]
}

pub fn is_interrupt_requested(if_flag: u8, interrupt: &Interrupt) -> bool {
    match interrupt {
        Interrupt::VBlank => if_flag & 0b1 != 0,
        Interrupt::STAT => if_flag & 0b10 != 0,
        Interrupt::Timer => if_flag & 0b100 != 0,
        Interrupt::Serial => if_flag & 0b1000 != 0,
        Interrupt::Joypad => if_flag & 0b10000 != 0,
    }
}

pub fn reset_if(if_flag: u8, interrupt: &Interrupt) -> u8 {
    match interrupt {
        Interrupt::VBlank => if_flag & !(0b1),
        Interrupt::STAT => if_flag & !(0b10),
        Interrupt::Timer => if_flag & !(0b100),
        Interrupt::Serial => if_flag & !(0b1000),
        Interrupt::Joypad => if_flag & !(0b10000),
    }
}
