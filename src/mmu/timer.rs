use crate::mmu::mmio::MMIO;

pub struct Timers {
    pub div: u16,
    pub tima: u8,
    pub tma: u8,
    pub tac: u8,

    pub irq: bool,
    and_result_falling_edge: bool,
}

impl MMIO for Timers {
    fn read(&mut self, address: u16) -> u8 {
        match address {
            0xFF04 => (self.div >> 8) as u8,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac,
            _ => unreachable!("Unreachable timer register read"),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0xFF04 => self.div = 0,
            0xFF05 => self.tima = value,
            0xFF06 => self.tma = value,
            0xFF07 => self.tac = value,
            _ => unreachable!("Unreachable timer register write"),
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
            and_result_falling_edge: false,
        }
    }

    pub fn tick(&mut self, cycles_tima: u16) {
        // increase each clock (t-cycle)
        for _ in 0..(cycles_tima * 4) {
            self.tick_div();
            self.tick_tima();

            self.and_result_falling_edge = self.get_timer_falling_edge();
        }
    }

    /// Get bit of DIV in position specified by the lower 2 bits of the TAC register
    fn get_sys_counter_bit(&self) -> bool {
        match self.tac & 0b11 {
            0 => (self.div & (1 << 9)) != 0,
            1 => (self.div & (1 << 3)) != 0,
            2 => (self.div & (1 << 5)) != 0,
            3 => (self.div & (1 << 7)) != 0,
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
            if self.and_result_falling_edge && !self.get_timer_falling_edge() {
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

    /// TIMA increase is based on TAC frequency and DIV
    fn get_timer_falling_edge(&self) -> bool {
        self.get_sys_counter_bit() & self.is_timer_enabled()
    }

    fn is_timer_enabled(&self) -> bool {
        (self.tac & 0b100) != 0
    }
}
