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
