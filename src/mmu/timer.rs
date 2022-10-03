pub struct Timers {
    pub div: u16,
    pub tima: u8,
    pub tma: u8,
    pub tac: u8,
    pub if_fired: u8,
    cycles_tima: u16,
}

impl Timers {
    pub fn new() -> Self {
        Self {
            div: 0xABCC,
            tima: 0,
            tma: 0,
            tac: 0xF8,
            if_fired: 0,
            cycles_tima: 0,
        }
    }

    pub fn tick(&mut self, cycles_tima: u16) {
        self.cycles_tima += cycles_tima;

        // increase each clock (t-cycle)
        for _ in 0..(cycles_tima * 4) {
            self.tick_div();
        }

        while self.cycles_tima >= self.get_tima_frequency() {
            self.tick_tima();
            self.cycles_tima -= self.get_tima_frequency();
        }
    }

    pub fn reset_div(&mut self) {
        self.div = 0;

        // write to DIV can increase TIMA (doesn't work yet)
        // if self.get_sys_counter_bit(self.tac & 0b11) == 0 {
        //     self.tick_tima();
        // }
    }

    fn get_sys_counter_bit(&self, tac_frequency: u8) -> u16 {
        match tac_frequency {
            0 => self.div & (1 << 9),
            1 => self.div & (1 << 3),
            2 => self.div & (1 << 5),
            3 => self.div & (1 << 7),
            _ => panic!("Invalid TAC frequency!"),
        }
    }

    /// Upper bits increase every 64 m-cycles
    fn tick_div(&mut self) {
        self.div += 1;
    }

    /// Increased at frequency specified by TAC
    fn tick_tima(&mut self) {
        if self.is_timer_enabled() {
            let (result, overflow) = self.tima.overflowing_add(1);

            if overflow {
                self.tima = 0x00; // set to tma next cycle
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
