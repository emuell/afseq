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
    Beats(u32),
    Bar(u32),
}

impl BeatTimeStep {
    /// Convert a beat or bar step to samples for the given beat time base.
    pub fn to_samples(&self, time_base: &BeatTimeBase) -> f64 {
        match *self {
            BeatTimeStep::Beats(amount) => time_base.samples_per_beat() * amount as f64,
            BeatTimeStep::Bar(amount) => time_base.samples_per_bar() * amount as f64,
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Shortcuts for creating beat-time based patterns.
impl BeatTimeBase {
    pub fn every_nth_beat<Value: PatternEventIter + 'static>(
        &self,
        beats: u32,
        value: Value,
    ) -> BeatTimePattern {
        BeatTimePattern::new(self.clone(), BeatTimeStep::Beats(beats), value)
    }
    pub fn every_nth_beat_with_offset<Value: PatternEventIter + 'static>(
        &self,
        beats: u32,
        offset: u32,
        value: Value,
    ) -> BeatTimePattern {
        BeatTimePattern::new_with_offset(
            self.clone(),
            BeatTimeStep::Beats(beats),
            BeatTimeStep::Beats(offset),
            value,
        )
    }

    pub fn every_nth_beat_with_pattern<Value: PatternEventIter + 'static>(
        &self,
        beats: u32,
        pattern: Vec<bool>,
        value: Value,
    ) -> BeatTimeSequencePattern {
        BeatTimeSequencePattern::new(self.clone(), BeatTimeStep::Beats(beats), pattern, value)
    }

    pub fn every_nth_bar<Value: PatternEventIter + 'static>(
        &self,
        bars: u32,
        value: Value,
    ) -> BeatTimePattern {
        BeatTimePattern::new(self.clone(), BeatTimeStep::Bar(bars), value)
    }
    pub fn every_nth_bar_with_offset<Value: PatternEventIter + 'static>(
        &self,
        bars: u32,
        offset_in_beats: u32,
        value: Value,
    ) -> BeatTimePattern {
        BeatTimePattern::new_with_offset(
            self.clone(),
            BeatTimeStep::Bar(bars),
            BeatTimeStep::Beats(offset_in_beats),
            value,
        )
    }
}
