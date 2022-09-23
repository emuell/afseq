use crate::{
    event::EventIter, rhythm::second_time::SecondTimeRhythm,
    rhythm::second_time_sequence::SecondTimeSequenceRhythm, SampleTime,
};

// -------------------------------------------------------------------------------------------------

/// Second time timing base for beat based [Rhythm](`crate::Rhythm`) impls.
#[derive(Clone)]
pub struct SecondTimeBase {
    pub samples_per_sec: u32,
}

impl SecondTimeBase {
    /// Convert given sample amount in seconds, using this time bases' samples per second rate.
    pub fn samples_to_seconds(&self, samples: SampleTime) -> f64 {
        samples as f64 / self.samples_per_sec as f64
    }
    /// Convert given second duration in samples, using this time bases' samples per second rate.
    pub fn seconds_to_samples(&self, seconds: f64) -> SampleTime {
        (seconds as f64 * self.samples_per_sec as f64) as SampleTime
    }
}

// -------------------------------------------------------------------------------------------------

/// Defines a number of seconds in SecondTimeBase.
pub type SecondTimeStep = f64;

/// Shortcuts for creating beat-time based patterns.
impl SecondTimeBase {
    pub fn every_nth_seconds<Iter: EventIter + 'static>(
        &self,
        step: SecondTimeStep,
        event_iter: Iter,
    ) -> SecondTimeRhythm {
        SecondTimeRhythm::new(self.clone(), step, event_iter)
    }
    pub fn every_nth_seconds_with_offset<Iter: EventIter + 'static>(
        &self,
        step: SecondTimeStep,
        offset: SecondTimeStep,
        event_iter: Iter,
    ) -> SecondTimeRhythm {
        SecondTimeRhythm::new_with_offset(self.clone(), step, offset, event_iter)
    }
    pub fn every_nth_seconds_with_pattern<
        Iter: EventIter + 'static,
        const N: usize,
        T: Ord + Default,
    >(
        &self,
        step: SecondTimeStep,
        pattern: [T; N],
        event_iter: Iter,
    ) -> SecondTimeSequenceRhythm {
        SecondTimeSequenceRhythm::new(self.clone(), step, pattern, event_iter)
    }
}
