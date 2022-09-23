use crate::{
    event::EventIter, rhythm::beat_time::BeatTimeRhythm,
    rhythm::beat_time_sequence::BeatTimeSequenceRhythm, SampleTime,
};

// -------------------------------------------------------------------------------------------------

/// Beat & bar timing base for beat based [Rhythm](`crate::Rhythm`) impls.
#[derive(Clone)]
pub struct BeatTimeBase {
    pub beats_per_min: f32,
    pub beats_per_bar: u32,
    pub samples_per_sec: u32,
}

impl BeatTimeBase {
    /// Time base's samples per beat, in order to convert beat to sample time and vice versa.
    pub fn samples_per_beat(&self) -> f64 {
        self.samples_per_sec as f64 * 60.0 / self.beats_per_min as f64
    }
    /// Time base's samples per bar, in order to convert bar to sample time and vice versa.
    pub fn samples_per_bar(&self) -> f64 {
        self.samples_per_sec as f64 * 60.0 / self.beats_per_min as f64 * self.beats_per_bar as f64
    }

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

/// Defines a number of steps in either beat or bar timing values.
pub enum BeatTimeStep {
    Sixteenth(u32),
    Beats(u32),
    Bar(u32),
}

impl BeatTimeStep {
    /// Get number of steps in the current time range.
    pub fn steps(&self) -> u32 {
        match *self {
            BeatTimeStep::Sixteenth(amount) => amount,
            BeatTimeStep::Beats(amount) => amount,
            BeatTimeStep::Bar(amount) => amount,
        }
    }
    /// Get number of samples for a single step.
    pub fn samples_per_step(&self, time_base: &BeatTimeBase) -> f64 {
        match *self {
            BeatTimeStep::Sixteenth(_) => time_base.samples_per_beat() / 4.0,
            BeatTimeStep::Beats(_) => time_base.samples_per_beat() as f64,
            BeatTimeStep::Bar(_) => time_base.samples_per_bar() as f64,
        }
    }
    /// Convert a beat or bar step to samples for the given beat time base.
    pub fn to_samples(&self, time_base: &BeatTimeBase) -> f64 {
        self.steps() as f64 * self.samples_per_step(time_base)
    }
}

// -------------------------------------------------------------------------------------------------

/// Shortcuts for creating beat-time based patterns.
impl BeatTimeBase {
    // Emit every Nth step
    pub fn every_nth_step<Iter: EventIter + 'static>(
        &self,
        step: BeatTimeStep,
        event_iter: Iter,
    ) -> BeatTimeRhythm {
        BeatTimeRhythm::new(self.clone(), step, event_iter)
    }
    pub fn every_nth_step_with_offset<Iter: EventIter + 'static>(
        &self,
        step: BeatTimeStep,
        offset: BeatTimeStep,
        event_iter: Iter,
    ) -> BeatTimeRhythm {
        BeatTimeRhythm::new_with_offset(self.clone(), step, offset, event_iter)
    }
    pub fn every_nth_step_with_pattern<
        Iter: EventIter + 'static,
        const N: usize,
        T: Ord + Default,
    >(
        &self,
        step: BeatTimeStep,
        pattern: [T; N],
        event_iter: Iter,
    ) -> BeatTimeSequenceRhythm {
        BeatTimeSequenceRhythm::new(self.clone(), step, pattern, event_iter)
    }

    // Emit every sixteenth
    pub fn every_nth_sixteenth<Iter: EventIter + 'static>(
        &self,
        sixteenth: u32,
        event_iter: Iter,
    ) -> BeatTimeRhythm {
        self.every_nth_step(BeatTimeStep::Sixteenth(sixteenth), event_iter)
    }
    pub fn every_nth_sixteenth_with_offset<Iter: EventIter + 'static>(
        &self,
        sixteenth: u32,
        offset: u32,
        event_iter: Iter,
    ) -> BeatTimeRhythm {
        self.every_nth_step_with_offset(
            BeatTimeStep::Sixteenth(sixteenth),
            BeatTimeStep::Sixteenth(offset),
            event_iter,
        )
    }
    pub fn every_nth_sixteenth_with_pattern<
        Iter: EventIter + 'static,
        const N: usize,
        T: Ord + Default,
    >(
        &self,
        sixteenth: u32,
        pattern: [T; N],
        event_iter: Iter,
    ) -> BeatTimeSequenceRhythm {
        self.every_nth_step_with_pattern(BeatTimeStep::Sixteenth(sixteenth), pattern, event_iter)
    }

    // Emit every beat
    pub fn every_nth_beat<Iter: EventIter + 'static>(
        &self,
        beats: u32,
        event_iter: Iter,
    ) -> BeatTimeRhythm {
        self.every_nth_step(BeatTimeStep::Beats(beats), event_iter)
    }
    pub fn every_nth_beat_with_offset<Iter: EventIter + 'static>(
        &self,
        beats: u32,
        offset: u32,
        event_iter: Iter,
    ) -> BeatTimeRhythm {
        self.every_nth_step_with_offset(
            BeatTimeStep::Beats(beats),
            BeatTimeStep::Beats(offset),
            event_iter,
        )
    }
    pub fn every_nth_beat_with_pattern<
        Iter: EventIter + 'static,
        const N: usize,
        T: Ord + Default,
    >(
        &self,
        beats: u32,
        pattern: [T; N],
        event_iter: Iter,
    ) -> BeatTimeSequenceRhythm {
        self.every_nth_step_with_pattern(BeatTimeStep::Beats(beats), pattern, event_iter)
    }

    // Emit every bar
    pub fn every_nth_bar<Iter: EventIter + 'static>(
        &self,
        bars: u32,
        event_iter: Iter,
    ) -> BeatTimeRhythm {
        self.every_nth_step(BeatTimeStep::Bar(bars), event_iter)
    }
    pub fn every_nth_bar_with_offset<Iter: EventIter + 'static>(
        &self,
        bars: u32,
        offset_in_beats: u32,
        event_iter: Iter,
    ) -> BeatTimeRhythm {
        self.every_nth_step_with_offset(
            BeatTimeStep::Bar(bars),
            BeatTimeStep::Beats(offset_in_beats),
            event_iter,
        )
    }
    pub fn every_nth_bar_with_pattern<
        Iter: EventIter + 'static,
        const N: usize,
        T: Ord + Default,
    >(
        &self,
        bars: u32,
        pattern: [T; N],
        event_iter: Iter,
    ) -> BeatTimeSequenceRhythm {
        self.every_nth_step_with_pattern(BeatTimeStep::Bar(bars), pattern, event_iter)
    }
}
