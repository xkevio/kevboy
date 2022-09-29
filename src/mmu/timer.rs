pub struct Timers {
    pub div: u16,
    pub tima: u8,
    pub tma: u8,
    pub tac: u8,
    pub if_fired: u8,
}

impl Timers {
    pub fn new() -> Self {
        Self {
            div: 0xABCC,
            tima: 0,
            tma: 0,
            tac: 0xF8,
            if_fired: 0,
        }
    }

    pub fn tick(&mut self, m_cycles: u64) {
        if m_cycles % 64 == 0 {
            self.tick_div();
        }

        if m_cycles % self.get_tima_frequency() as u64 == 0 {
            self.tick_tima();
        }
    }

    /// Should be called every 64 m-cycles
    fn tick_div(&mut self) {
        self.div += 1;
    }

    /// Increased at frequency specified by TAC
    fn tick_tima(&mut self) {
        if self.is_timer_enabled() {
            let (result, overflow) = self.tima.overflowing_add(1);

            if overflow {
                self.tima = self.tma;
                self.if_fired = 0b100;
            } else {
                self.tima = result;
            }
        }
    }

    fn is_timer_enabled(&self) -> bool {
        (self.tac & 0b100) != 0
    }

    // get frequency in m-cycles
    fn get_tima_frequency(&self) -> u16 {
        match self.tac & 0b11 {
            0 => 256,
            1 => 4,
            2 => 16,
            3 => 64,
            _ => panic!("Greater value than two bits should be able to store!"),
        }
    }
}
