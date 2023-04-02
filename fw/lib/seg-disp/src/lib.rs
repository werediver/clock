#![cfg_attr(not(test), no_std)]

extern crate alloc;

use alloc::{borrow::ToOwned, string::String, vec::Vec};
use snafu::Snafu;

/// Individual segments of a seven-segment indicator with a decimal point.
///
/// ```text
///    -A-
/// F |   | B
///    -G-
/// E |   | C
///    -D-  o DP
/// ```
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[repr(u8)]
pub enum Segment7DP {
    /// The top segment.
    A = 0b00000001,
    /// The top-right segment.
    B = 0b00000010,
    /// The bottom-right segment.
    C = 0b00000100,
    /// The bottom segment.
    D = 0b00001000,
    /// The bottom-left segment.
    E = 0b00010000,
    /// The top-left segment.
    F = 0b00100000,
    /// The middle segment.
    G = 0b01000000,
    /// The decimal point.
    DP = 0b10000000,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[repr(transparent)]
pub struct Char7DP {
    state: u8,
}

impl Char7DP {
    pub const fn space() -> Self {
        Self { state: 0 }
    }

    pub const fn new(segments: &[Segment7DP]) -> Self {
        let mut state = 0u8;

        let mut i = 0usize;
        while i < segments.len() {
            let seg = segments[i] as u8;

            #[cfg(debug_assertions)]
            if state & seg != 0 {
                panic!("A duplicate segment is encountered");
            }

            state |= seg;

            i += 1;
        }

        Self { state }
    }

    pub const fn try_from_u8(value: u8) -> Result<Self, Char7DPTryFromError> {
        use Segment7DP::*;

        match value {
            0 => Ok(Self::new(&[A, B, C, D, E, F])),
            1 => Ok(Self::new(&[B, C])),
            2 => Ok(Self::new(&[A, B, G, E, D])),
            3 => Ok(Self::new(&[A, B, G, C, D])),
            4 => Ok(Self::new(&[F, G, B, C])),
            5 => Ok(Self::new(&[A, F, G, C, D])),
            6 => Ok(Self::new(&[A, F, E, D, C, G])),
            7 => Ok(Self::new(&[A, B, C])),
            8 => Ok(Self::new(&[A, B, C, D, E, F, G])),
            9 => Ok(Self::new(&[G, F, A, B, C, D])),
            _ => Err(Char7DPTryFromError::UnsupportedValue),
        }
    }

    pub const fn try_from_char(value: char) -> Result<Self, Char7DPTryFromError> {
        use Segment7DP::*;

        match value {
            ' ' => Ok(Self::space()),
            '0' => Ok(Self::new(&[A, B, C, D, E, F])),
            '1' => Ok(Self::new(&[B, C])),
            '2' => Ok(Self::new(&[A, B, G, E, D])),
            '3' => Ok(Self::new(&[A, B, G, C, D])),
            '4' => Ok(Self::new(&[F, G, B, C])),
            '5' => Ok(Self::new(&[A, F, G, C, D])),
            '6' => Ok(Self::new(&[A, F, E, D, C, G])),
            '7' => Ok(Self::new(&[A, B, C])),
            '8' => Ok(Self::new(&[A, B, C, D, E, F, G])),
            '9' => Ok(Self::new(&[G, F, A, B, C, D])),
            '.' => Ok(Self::new(&[DP])),
            '-' => Ok(Self::new(&[G])),
            '_' => Ok(Self::new(&[D])),
            '=' => Ok(Self::new(&[D, G])),
            _ => Err(Char7DPTryFromError::UnsupportedValue),
        }
    }

    pub const fn try_from_chars<const N: usize>(
        value: &[char; N],
    ) -> Result<[Self; N], Char7DPTryFromError> {
        let mut s = [Self::space(); N];

        let mut i = 0usize;
        while i < N {
            if let Ok(c) = Self::try_from_char(value[i]) {
                s[i] = c;
            } else {
                return Err(Char7DPTryFromError::UnsupportedValue);
            }

            i += 1;
        }

        Ok(s)
    }

    pub fn try_from_str(value: &str) -> Result<Vec<Char7DP>, Char7DPTryFromError> {
        let mut s = Vec::with_capacity(value.len());

        let mut chars = value.chars().peekable();
        while let Some(c) = chars.next() {
            if let Ok(c) = Self::try_from_char(c) {
                if let Some('.') = chars.peek() {
                    _ = chars.next();
                    s.push(c.with_dp());
                } else {
                    s.push(c);
                }
            } else {
                return Err(Char7DPTryFromError::UnsupportedValue);
            }
        }

        Ok(s)
    }

    pub const fn with_dp(&self) -> Self {
        Self {
            state: self.state | Segment7DP::DP as u8,
        }
    }

    #[inline]
    pub fn state(&self) -> u8 {
        self.state
    }

    pub fn is_set(&self, seg: Segment7DP) -> bool {
        let seg = seg as u8;
        self.state & seg == seg
    }

    pub fn render(value: &[Self]) -> String {
        use Segment7DP::*;

        let mut s = String::default();

        fn sep(i: usize, len: usize) -> String {
            if i + 1 < len {
                " ".to_owned()
            } else {
                "".to_owned()
            }
        }

        for i in 0..value.len() {
            s += if value[i].is_set(A) { " -- " } else { "    " };
            s += &sep(i, value.len());
        }
        s += "\n";

        for i in 0..value.len() {
            s += if value[i].is_set(F) { "|" } else { " " };
            s += "  ";
            s += if value[i].is_set(B) { "|" } else { " " };
            s += &sep(i, value.len());
        }
        s += "\n";

        for i in 0..value.len() {
            s += if value[i].is_set(G) { " -- " } else { "    " };
            s += &sep(i, value.len());
        }
        s += "\n";

        for i in 0..value.len() {
            s += if value[i].is_set(E) { "|" } else { " " };
            s += "  ";
            s += if value[i].is_set(C) { "|" } else { " " };
            s += &sep(i, value.len());
        }
        s += "\n";

        for i in 0..value.len() {
            s += if value[i].is_set(D) { " -- " } else { "    " };
            s += &if value[i].is_set(DP) {
                ".".to_owned()
            } else {
                sep(i, value.len())
            };
        }
        s += "\n";

        s
    }
}

impl TryFrom<u8> for Char7DP {
    type Error = Char7DPTryFromError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::try_from_u8(value)
    }
}
impl TryFrom<char> for Char7DP {
    type Error = Char7DPTryFromError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        Self::try_from_char(value)
    }
}

#[derive(Snafu, Debug)]
pub enum Char7DPTryFromError {
    #[snafu(display("Unsupported value"))]
    UnsupportedValue,
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;

    #[test]
    fn render_digits_dp() -> Result<(), Box<dyn Error>> {
        let seg_chars: Vec<Char7DP> = Char7DP::try_from_str("0.123456789")?;
        let s = Char7DP::render(&seg_chars);

        let s_ref = r#"
 --        --   --        --   --   --   --   -- 
|  |    |    |    | |  | |    |       | |  | |  |
           --   --   --   --   --        --   -- 
|  |    | |       |    |    | |  |    | |  |    |
 -- .      --   --        --   --        --   -- 
"#
        .trim_start_matches('\n');

        assert_eq!(s, s_ref);

        Ok(())
    }
}
