//! Wallclock time based `Pattern` implementation.

use crate::{
    pattern::generic::{GenericPattern, GenericPatternTimeStep},
    prelude::SampleTimeBase,
    time::SecondTimeStep,
    BeatTimeBase, ExactSampleTime,
};

// -------------------------------------------------------------------------------------------------

impl GenericPatternTimeStep for SecondTimeStep {
    #[inline]
    fn default_offset() -> Self {
        0.0
    }

    #[inline]
    fn default_step() -> Self {
        1.0
    }

    #[inline]
    fn to_samples(&self, time_base: &BeatTimeBase) -> ExactSampleTime {
        *self as ExactSampleTime * time_base.samples_per_second() as ExactSampleTime
    }
}

// -------------------------------------------------------------------------------------------------

/// A Pattern with a beat time offset and beat time step.
pub type SecondTimePattern = GenericPattern<SecondTimeStep, SecondTimeStep>;

// -------------------------------------------------------------------------------------------------

/// Shortcuts for creating second-time based patterns.
impl BeatTimeBase {
    pub fn every_nth_seconds(&self, step: SecondTimeStep) -> SecondTimePattern {
        SecondTimePattern::new(*self, step)
    }
}
