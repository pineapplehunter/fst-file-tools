use std::fmt;

use nom::{combinator::map, number::complete::be_i8};
use serde::Serialize;

use crate::FstParsable;

/// Time scale of the waveform
#[derive(Clone, Copy, Serialize)]
#[repr(transparent)]
pub struct TimeScale(
    /// The time scale where the number N corresponds to 10^N s
    pub i8,
);

impl fmt::Debug for TimeScale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for TimeScale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "10^{}s", self.0)
    }
}

impl FstParsable for TimeScale {
    fn parse(input: &[u8]) -> crate::error::FstFileResult<'_, Self> {
        map(be_i8, |v| TimeScale(v))(input)
    }
}
