use embedded_hal::blocking::delay::{DelayMs, DelayUs};

use crate::uptime::Uptime;

impl DelayUs<u64> for Uptime {
    fn delay_us(&mut self, us: u64) {
        Self::delay_us(self, us)
    }
}

impl DelayMs<u8> for Uptime {
    fn delay_ms(&mut self, ms: u8) {
        Self::delay_ms(self, ms as u64)
    }
}

impl DelayMs<u64> for Uptime {
    fn delay_ms(&mut self, ms: u64) {
        Self::delay_ms(self, ms)
    }
}
