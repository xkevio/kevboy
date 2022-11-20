use crate::mmu::mmio::MMIO;

pub struct Timers {
    pub div: u16,
    pub tima: u8,
    pub tma: u8,
    pub tac: u8,

    pub irq: bool,
    and_result_falling_edge: u8,
}

impl MMIO for Timers {
    fn read(&mut self, address: u16) -> u8 {
        match address {
            0xFF04 => (self.div >> 8) as u8,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac,
            _ => unreachable!("Unreachable Timer register read"),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0xFF04 => self.div = 0,
            0xFF05 => self.tima = value,
            0xFF06 => self.tma = value,
            0xFF07 => self.tac = value,
            _ => unreachable!("Unreachable Timer register write"),
        }
    }
}

impl Timers {
    pub fn new() -> Self {
        Self {
            div: 0xABCC,
            tima: 0,
            tma: 0,
            tac: 0xF8,

            irq: false,
            and_result_falling_edge: 0,
        }
    }

    pub fn tick(&mut self, cycles_tima: u16) {
        // increase each clock (t-cycle)
        for _ in 0..(cycles_tima * 4) {
            self.tick_div();
            self.tick_tima();

            self.and_result_falling_edge =
                self.get_sys_counter_bit(self.tac & 0b11) & ((self.tac & 0b100) >> 2);
        }
    }

    /// Get bit of DIV in position specified by the lower 2 bits of the TAC register
    fn get_sys_counter_bit(&self, tac_frequency: u8) -> u8 {
        match tac_frequency {
            0 => ((self.div & (1 << 9)) >> 9) as u8,
            1 => ((self.div & (1 << 3)) >> 3) as u8,
            2 => ((self.div & (1 << 5)) >> 5) as u8,
            3 => ((self.div & (1 << 7)) >> 7) as u8,
            _ => panic!("Invalid TAC frequency!"),
        }
    }

    /// Upper bits increase every 64 m-cycles
    fn tick_div(&mut self) {
        self.div += 1;
    }

    /// Increased at frequency specified by TAC
    /// TODO: tima and tma write while reload
    fn tick_tima(&mut self) {
        if !self.irq {
            // check for falling edge of "AND Result" -- bit of DIV & timer enable bit -- only then increase TIMA
            // obscure behavior, not necessary for most games but more accurate
            if self.and_result_falling_edge == 1
                && self.get_sys_counter_bit(self.tac & 0b11) & ((self.tac & 0b100) >> 2) == 0
            {
                let (result, overflow) = self.tima.overflowing_add(1);

                if overflow {
                    self.tima = 0x00; // set to tma next cycle
                    self.irq = true;
                } else {
                    self.tima = result;
                }
            }
        }
    }

    fn is_timer_enabled(&self) -> bool {
        (self.tac & 0b100) != 0
    }
}
