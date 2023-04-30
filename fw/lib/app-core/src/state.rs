#[derive(Default)]
pub struct State {
    pub rtc: RTC,
}

#[derive(Default)]
pub struct RTC {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}
