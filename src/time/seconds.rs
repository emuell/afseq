use crate::{rhythm::second_time::SecondTimeRhythm, time::TimeBase};

// -------------------------------------------------------------------------------------------------

/// Second time timing base for beat based [Rhythm](`crate::Rhythm`) impls.
#[derive(Copy, Clone)]
pub struct SecondTimeBase {
    pub samples_per_sec: u32,
}

impl TimeBase for SecondTimeBase {
    fn samples_per_second(&self) -> u32 {
        self.samples_per_sec
    }
}

// -------------------------------------------------------------------------------------------------

/// Defines a number of seconds in SecondTimeBase.
pub type SecondTimeStep = f64;

/// Shortcuts for creating beat-time based patterns.
impl SecondTimeBase {
    pub fn every_nth_seconds(&self, step: SecondTimeStep) -> SecondTimeRhythm {
        SecondTimeRhythm::new(*self, step)
    }
}
