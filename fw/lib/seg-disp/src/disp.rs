use core::slice::SliceIndex;

use crate::char7dp::Char7DP;

pub struct Disp<const N: usize> {
    chars: [Char7DP; N],
    char_index: usize,
}

impl<const N: usize> Default for Disp<N> {
    fn default() -> Self {
        Self {
            chars: [Default::default(); N],
            char_index: 0,
        }
    }
}

impl<const N: usize> Disp<N> {
    pub fn set_chars(&mut self, chars: [Char7DP; N]) {
        self.chars = chars;
    }

    pub fn set_chars_at<I>(&mut self, pos: I, chars: &[Char7DP])
    where
        I: SliceIndex<[Char7DP], Output = [Char7DP]>,
    {
        self.chars[pos].copy_from_slice(chars);
    }

    pub fn run(&mut self) -> Action {
        let c = self.chars[self.char_index];
        let index = self.char_index;

        self.char_index = (self.char_index + 1) % self.chars.len();

        Action::Render(c, index)
    }
}

pub enum Action {
    Render(Char7DP, usize),
}
