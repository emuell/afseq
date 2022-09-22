use crate::{
    events::PatternEventIter, pattern::beat_time::BeatTimePattern,
    pattern::beat_time_sequence::BeatTimeSequencePattern,
};

// -------------------------------------------------------------------------------------------------

/// Beat & bar timing base for beat based [Pattern](`crate::Pattern`) impls.
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
    pub fn every_nth_step<Value: PatternEventIter + 'static>(
        &self,
        step: BeatTimeStep,
        value: Value,
    ) -> BeatTimePattern {
        BeatTimePattern::new(self.clone(), step, value)
    }
    pub fn every_nth_step_with_offset<Value: PatternEventIter + 'static>(
        &self,
        step: BeatTimeStep,
        offset: BeatTimeStep,
        value: Value,
    ) -> BeatTimePattern {
        BeatTimePattern::new_with_offset(self.clone(), step, offset, value)
    }
    pub fn every_nth_step_with_pattern<Value: PatternEventIter + 'static, const N: usize>(
        &self,
        step: BeatTimeStep,
        pattern: [u8; N],
        value: Value,
    ) -> BeatTimeSequencePattern {
        BeatTimeSequencePattern::new(
            self.clone(),
            step,
            pattern.iter().map(|f| *f != 0).collect(),
            value,
        )
    }

    pub fn every_nth_sixteenth<Value: PatternEventIter + 'static>(
        &self,
        sixteenth: u32,
        value: Value,
    ) -> BeatTimePattern {
        self.every_nth_step(BeatTimeStep::Sixteenth(sixteenth), value)
    }
    pub fn every_nth_sixteenth_with_offset<Value: PatternEventIter + 'static>(
        &self,
        sixteenth: u32,
        offset: u32,
        value: Value,
    ) -> BeatTimePattern {
        self.every_nth_step_with_offset(
            BeatTimeStep::Sixteenth(sixteenth),
            BeatTimeStep::Sixteenth(offset),
            value,
        )
    }
    pub fn every_nth_sixteenth_with_pattern<Value: PatternEventIter + 'static, const N: usize>(
        &self,
        sixteenth: u32,
        pattern: [u8; N],
        value: Value,
    ) -> BeatTimeSequencePattern {
        self.every_nth_step_with_pattern(BeatTimeStep::Sixteenth(sixteenth), pattern, value)
    }

    pub fn every_nth_beat<Value: PatternEventIter + 'static>(
        &self,
        beats: u32,
        value: Value,
    ) -> BeatTimePattern {
        self.every_nth_step(BeatTimeStep::Beats(beats), value)
    }
    pub fn every_nth_beat_with_offset<Value: PatternEventIter + 'static>(
        &self,
        beats: u32,
        offset: u32,
        value: Value,
    ) -> BeatTimePattern {
        self.every_nth_step_with_offset(
            BeatTimeStep::Beats(beats),
            BeatTimeStep::Beats(offset),
            value,
        )
    }
    pub fn every_nth_beat_with_pattern<Value: PatternEventIter + 'static, const N: usize>(
        &self,
        beats: u32,
        pattern: [u8; N],
        value: Value,
    ) -> BeatTimeSequencePattern {
        self.every_nth_step_with_pattern(BeatTimeStep::Beats(beats), pattern, value)
    }

    pub fn every_nth_bar<Value: PatternEventIter + 'static>(
        &self,
        bars: u32,
        value: Value,
    ) -> BeatTimePattern {
        self.every_nth_step(BeatTimeStep::Bar(bars), value)
    }
    pub fn every_nth_bar_with_offset<Value: PatternEventIter + 'static>(
        &self,
        bars: u32,
        offset_in_beats: u32,
        value: Value,
    ) -> BeatTimePattern {
        self.every_nth_step_with_offset(
            BeatTimeStep::Bar(bars),
            BeatTimeStep::Beats(offset_in_beats),
            value,
        )
    }
    pub fn every_nth_bar_with_pattern<Value: PatternEventIter + 'static, const N: usize>(
        &self,
        bars: u32,
        pattern: [u8; N],
        value: Value,
    ) -> BeatTimeSequencePattern {
        self.every_nth_step_with_pattern(BeatTimeStep::Bar(bars), pattern, value)
    }
}
