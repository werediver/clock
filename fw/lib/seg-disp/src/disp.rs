use core::slice::SliceIndex;

use crate::char7dp::Char7DP;

type Duration = fugit::Duration<u64, 1, 1_000_000>;

pub struct Disp<const N: usize> {
    chars: [Char7DP; N],
    update_period: Duration,
    duty_cycle: f32,
    state: State<N>,
}

impl<const N: usize> Disp<N> {
    pub fn new(update_period: Duration, duty_cycle: f32) -> Self {
        Self {
            chars: [Default::default(); N],
            update_period,
            duty_cycle,
            state: State::default(),
        }
    }

    pub fn set_chars(&mut self, chars: [Char7DP; N]) {
        self.chars = chars;
    }

    pub fn set_chars_at<I>(&mut self, pos: I, chars: &[Char7DP])
    where
        I: SliceIndex<[Char7DP], Output = [Char7DP]>,
    {
        self.chars[pos].copy_from_slice(chars);
    }

    pub fn run(&mut self) -> (Action, Duration) {
        let (action, delay) = if self.state.is_char_active {
            let c = self.chars[self.state.char_index];
            (
                Action::Render(c, self.state.char_index),
                self.delay(self.duty_cycle),
            )
        } else {
            (
                Action::Render(Char7DP::space(), self.state.char_index),
                self.delay(1.0 - self.duty_cycle),
            )
        };

        self.state = self.state.next();

        (action, delay)
    }

    fn delay(&self, k: f32) -> Duration {
        Duration::from_ticks((self.update_period.ticks() as f32 * k) as u64)
    }
}

struct State<const N: usize> {
    char_index: usize,
    is_char_active: bool,
}

impl<const N: usize> Default for State<N> {
    fn default() -> Self {
        Self {
            char_index: 0,
            is_char_active: true,
        }
    }
}

impl<const N: usize> State<N> {
    fn next(&self) -> Self {
        if self.is_char_active {
            Self {
                char_index: self.char_index,
                is_char_active: false,
            }
        } else {
            Self {
                char_index: self.next_char_index(),
                is_char_active: true,
            }
        }
    }

    fn next_char_index(&self) -> usize {
        (self.char_index + 1) % N
    }
}

pub enum Action {
    Render(Char7DP, usize),
}
