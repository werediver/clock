use crate::{
    action::Action,
    common::Duration,
    state::State,
    task::{NextRun, Task},
};
use seg_disp::{char7dp::Char7DP, char7dp_seq::Char7DPSeq};

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
        let time = hhmm_to_char7dp_array(state.rtc.hour, state.rtc.minute, state.rtc.second);
        self.disp.set_chars(time);

        let (action, delay) = self.disp.run();

        (Some(Action::Display(action)), NextRun::After(delay))
    }
}

fn hhmm_to_char7dp_array(hour: u8, minute: u8, second: u8) -> [Char7DP; 4] {
    let mut time = [Char7DP::space(); 4];

    Char7DPSeq::new(&mut time[0..2]).set_dec(minute as usize, true);
    Char7DPSeq::new(&mut time[2..4]).set_dec(hour as usize, false)[0].set_dp(second & 1 == 0);

    time
}
