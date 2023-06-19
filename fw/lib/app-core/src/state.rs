use crate::features::charger::BatteryState;

pub struct State {
    pub rtc: RTC,
    pub ext_power: bool,
    pub bat_voltage: (f32, f32),
    pub bat_level: BatteryState,
}

impl Default for State {
    fn default() -> Self {
        Self {
            rtc: Default::default(),
            ext_power: false,
            bat_voltage: (0.0, 0.0),
            bat_level: BatteryState::AboveNominal,
        }
    }
}

#[derive(Default)]
pub struct RTC {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}
