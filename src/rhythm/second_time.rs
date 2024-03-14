//! Wallclock time based `Rhythm` implementation.

use crate::{
    prelude::TimeBase,
    rhythm::generic::{GenericRhythm, GenericRhythmTimeStep},
    time::SecondTimeStep,
    BeatTimeBase,
};

// -------------------------------------------------------------------------------------------------

impl GenericRhythmTimeStep for SecondTimeStep {
    fn default_offset() -> Self {
        0.0
    }

    fn default_step() -> Self {
        1.0
    }

    fn to_samples(&self, time_base: &BeatTimeBase) -> f64 {
        time_base.seconds_to_samples_exact(*self)
    }
}

// -------------------------------------------------------------------------------------------------

/// A Rhythm with a beat time offset and beat time step.
pub type SecondTimeRhythm = GenericRhythm<SecondTimeStep, SecondTimeStep>;

// -------------------------------------------------------------------------------------------------

/// Shortcuts for creating sample-time based rhythms.
impl BeatTimeBase {
    pub fn every_nth_seconds(&self, step: SecondTimeStep) -> SecondTimeRhythm {
        SecondTimeRhythm::new(*self, step)
    }
}
