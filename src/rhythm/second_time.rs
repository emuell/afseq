//! Wallclock time based `Rhythm` implementation.

use crate::{
    prelude::TimeBase,
    rhythm::generic::{GenericRhythm, GenericRhythmTimeStep},
    time::SecondTimeStep,
    BeatTimeBase,
};

// -------------------------------------------------------------------------------------------------

impl GenericRhythmTimeStep for SecondTimeStep {
    #[inline]
    fn default_offset() -> Self {
        0.0
    }

    #[inline]
    fn default_step() -> Self {
        1.0
    }

    #[inline]
    fn to_samples(&self, time_base: &BeatTimeBase) -> f64 {
        *self * time_base.samples_per_second() as f64
    }
}

// -------------------------------------------------------------------------------------------------

/// A Rhythm with a beat time offset and beat time step.
pub type SecondTimeRhythm = GenericRhythm<SecondTimeStep, SecondTimeStep>;

// -------------------------------------------------------------------------------------------------

/// Shortcuts for creating second-time based rhythms.
impl BeatTimeBase {
    pub fn every_nth_seconds(&self, step: SecondTimeStep) -> SecondTimeRhythm {
        SecondTimeRhythm::new(*self, step)
    }
}
