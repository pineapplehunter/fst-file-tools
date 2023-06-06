use std::fmt;

use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
#[repr(transparent)]
pub struct TimeScale(pub i8);

impl fmt::Display for TimeScale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "10^{}s", self.0)
    }
}
