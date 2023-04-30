use core::ops::{Index, IndexMut};

use crate::char7dp::Char7DP;

#[repr(transparent)]
pub struct Char7DPSeq<'a> {
    chars: &'a mut [Char7DP],
}

impl<'a> Index<usize> for Char7DPSeq<'a> {
    type Output = Char7DP;

    fn index(&self, index: usize) -> &Self::Output {
        &self.chars[index]
    }
}

impl<'a> IndexMut<usize> for Char7DPSeq<'a> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.chars[index]
    }
}

impl<'a> Char7DPSeq<'a> {
    pub fn new(chars: &'a mut [Char7DP]) -> Self {
        Self { chars }
    }

    pub fn set_dec(&mut self, n: usize, leading_zeros: bool) -> &mut Self {
        let mut p = n;
        for i in 0..self.chars.len() {
            self.chars[i] = if p > 0 || i == 0 || leading_zeros {
                let q = (p % 10) as u8;
                p /= 10;
                Char7DP::try_from_u8(q).unwrap()
            } else {
                Char7DP::space()
            }
        }

        self
    }
}
