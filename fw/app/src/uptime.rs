use cortex_m::{
    interrupt,
    peripheral::{syst::SystClkSource, SYST},
};
use cortex_m_rt::exception;

use app_core::common::Instant;

pub struct Uptime {
    _syst: SYST,
    syst_reload_period_ms: u32,
}

impl Uptime {
    /// Blocking delays longer than SYST reload period will invoke WFI (wait for interrupt) instruction.
    pub fn new(mut syst: SYST, syst_reload_period_ms: u32) -> Self {
        syst.set_clock_source(SystClkSource::External);

        // See
        //
        // - [SysTick Reload Value Register](https://developer.arm.com/documentation/dui0497/a/cortex-m0-peripherals/optional-system-timer--systick/systick-reload-value-register)
        // - [SysTick Calibration Value Register](https://developer.arm.com/documentation/dui0497/a/cortex-m0-peripherals/optional-system-timer--systick/systick-calibration-value-register)

        let ticks_per_reload_period =
            syst_reload_period_ms * (SYST::get_ticks_per_10ms() + 1) / 10 - 1;

        syst.set_reload(ticks_per_reload_period);
        syst.clear_current();
        syst.enable_counter();
        syst.enable_interrupt();

        Uptime {
            _syst: syst,
            syst_reload_period_ms,
        }
    }

    /// Return the "uptime" in microseconds.
    pub fn get_us(&self) -> u64 {
        /// Return the uptime, if [`SYST::get_current()`] and [`SYST_RELOAD_COUNT`]
        /// are captured within the same [`SYST`] run.
        ///
        /// A [`SYST`] overflow occurring between capturing [`SYST::get_current()`]
        /// and [`SYST_RELOAD_COUNT`] would result in a large error in the calculated
        /// uptime.
        fn try_get_us() -> Option<u64> {
            let syst_current1 = SYST::get_current();
            interrupt::free(|_| {
                let syst_current2 = SYST::get_current();
                if syst_current2 <= syst_current1 {
                    let syst_reload_count = unsafe { SYST_RELOAD_COUNT };

                    let syst_reload = SYST::get_reload();

                    let syst_ticks_per_10ms = (SYST::get_ticks_per_10ms() + 1) as u64;

                    let syst_base_us = {
                        let syst_reload_period_us =
                            1000 * (syst_reload as u64 + 1) * 10 / syst_ticks_per_10ms;

                        syst_reload_count as u64 * syst_reload_period_us
                    };

                    let syst_current_us =
                        1000 * (syst_reload - syst_current1) as u64 * 10 / syst_ticks_per_10ms;

                    Some(syst_base_us + syst_current_us)
                } else {
                    None
                }
            })
        }

        loop {
            if let Some(uptime) = try_get_us() {
                return uptime;
            }
        }
    }

    pub fn get_instant(&self) -> Instant {
        Instant::from_ticks(self.get_us())
    }

    pub fn delay_us(&self, us: u64) {
        let start = self.get_instant();
        let delay = fugit::Duration::<u64, 1, 1_000_000>::from_ticks(us);
        let end = start
            .checked_add_duration(delay)
            .expect("uptime must not overflow during the delay");
        loop {
            let now = self.get_instant();
            if let Some(left) = end.checked_duration_since(now) {
                if left.ticks() == 0 {
                    break;
                } else if left.to_millis() >= self.syst_reload_period_ms as u64 {
                    cortex_m::asm::wfi();
                }
            } else {
                // let overshot = now - end;
                // if overshot > Duration::from_ticks(20u64) {
                //     rprintln!("Delay overshot by {}", overshot);
                // }
                break;
            }
        }
    }

    pub fn delay_ms(&self, ms: u64) {
        let start = self.get_instant();
        let delay = fugit::Duration::<u64, 1, 1_000>::from_ticks(ms);
        let end = start
            .checked_add_duration(delay)
            .expect("uptime must not overflow during the delay");
        loop {
            let now = self.get_instant();
            if let Some(left) = end.checked_duration_since(now) {
                if left.to_micros() == 0 {
                    break;
                } else if left.to_millis() >= self.syst_reload_period_ms as u64 {
                    cortex_m::asm::wfi();
                }
            } else {
                // let overshot = now - end;
                // if overshot > Duration::from_ticks(20u64) {
                //     rprintln!("Delay overshot by {}", overshot);
                // }
                break;
            }
        }
    }
}

#[exception]
#[allow(non_snake_case)]
fn SysTick() {
    unsafe {
        SYST_RELOAD_COUNT += 1;
    }
}

static mut SYST_RELOAD_COUNT: u32 = 0;
