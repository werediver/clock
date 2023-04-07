use core::ops::{Deref, DerefMut};

use crate::char7dp::Char7DP;

#[repr(transparent)]
pub struct Char7DPSeq<'a> {
    chars: &'a mut [Char7DP],
}

impl<'a> Deref for Char7DPSeq<'a> {
    type Target = [Char7DP];

    fn deref(&self) -> &Self::Target {
        self.chars
    }
}

impl<'a> DerefMut for Char7DPSeq<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.chars
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
