use crate::{
    action::Action,
    common::Duration,
    state::State,
    task::{NextRun, Task},
};
use seg_disp::{char7dp::Char7DP, char7dp_seq::Char7DPSeq};

use super::charger::BatteryState;

pub struct Display {
    disp: seg_disp::disp::Disp<4>,
}

impl Default for Display {
    fn default() -> Self {
        Self {
            disp: seg_disp::disp::Disp::new(Duration::from_ticks(2_000), 1.0),
        }
    }
}

impl Task<State, Action> for Display {
    fn run(&mut self, state: &mut State) -> (Option<Action>, NextRun) {
        let mut time = [Char7DP::space(); 4];

        match state.bat_level {
            BatteryState::Critical => {
                time[0].set_dp(state.rtc.second & 1 == 0);
            }
            BatteryState::BelowNominal | BatteryState::AboveNominal | BatteryState::Charged => {
                let (hour, minute, second) = (state.rtc.hour, state.rtc.minute, state.rtc.second);
                Char7DPSeq::new(&mut time[0..2]).set_dec(minute as usize, true);
                Char7DPSeq::new(&mut time[2..4]).set_dec(hour as usize, false);
                time[2].set_dp(second & 1 == 0);
            }
            BatteryState::Charging => {
                let (hour, minute, second) = (state.rtc.hour, state.rtc.minute, state.rtc.second);
                Char7DPSeq::new(&mut time[0..2]).set_dec(minute as usize, true);
                Char7DPSeq::new(&mut time[2..4]).set_dec(hour as usize, false);
                time[second as usize % 4].set_dp(true);
            }
        }

        self.disp.set_chars(time);

        let (action, delay) = self.disp.run();

        (Some(Action::Display(action)), NextRun::After(delay))
    }
}
