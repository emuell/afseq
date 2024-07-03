use crate::{
    time::{SampleTimeDisplay, TimeBase},
    SampleTime, SecondTimeBase,
};

// -------------------------------------------------------------------------------------------------

/// Beat & bar timing base for beat based [Rhythm](`crate::Rhythm`) impls.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct BeatTimeBase {
    pub beats_per_min: f32,
    pub beats_per_bar: u32,
    pub samples_per_sec: u32,
}

impl BeatTimeBase {
    /// Time base's samples per beat, in order to convert beat to sample time and vice versa.
    #[inline]
    pub fn samples_per_beat(&self) -> f64 {
        self.samples_per_sec as f64 * 60.0 / self.beats_per_min as f64
    }
    /// Time base's samples per bar, in order to convert bar to sample time and vice versa.
    #[inline]
    pub fn samples_per_bar(&self) -> f64 {
        self.samples_per_sec as f64 * 60.0 / self.beats_per_min as f64 * self.beats_per_bar as f64
    }
}

impl From<BeatTimeBase> for SecondTimeBase {
    fn from(val: BeatTimeBase) -> Self {
        SecondTimeBase {
            samples_per_sec: val.samples_per_sec,
        }
    }
}

impl TimeBase for BeatTimeBase {
    #[inline]
    fn samples_per_second(&self) -> u32 {
        self.samples_per_sec
    }
}

impl SampleTimeDisplay for BeatTimeBase {
    /// generate a bar.beat.ppq string representation of the the given sample time
    fn display(&self, sample_time: SampleTime) -> String {
        let total_beats = sample_time / self.samples_per_beat() as u64;
        let total_beats_f = sample_time as f64 / self.samples_per_beat();
        let beat_fractions = total_beats_f - total_beats as f64;
        let bars = total_beats / self.beats_per_bar as u64;
        let beats = total_beats - self.beats_per_bar as u64 * bars;
        let ppq = (beat_fractions * 960.0 + 0.5) as u64;
        format!("{}.{}.{:03}", bars + 1, beats + 1, ppq)
    }
}

// -------------------------------------------------------------------------------------------------

/// Defines a number of steps in sixteenth, beat or bar amounts.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum BeatTimeStep {
    SixtyFourth(f32),
    ThirtySecond(f32),
    Sixteenth(f32),
    Eighth(f32),
    Beats(f32),
    Half(f32),
    Whole(f32),
    Bar(f32),
}

impl BeatTimeStep {
    /// Get number of steps in the current time resolution.
    pub fn steps(&self) -> f32 {
        match *self {
            BeatTimeStep::SixtyFourth(amount) => amount,
            BeatTimeStep::ThirtySecond(amount) => amount,
            BeatTimeStep::Sixteenth(amount) => amount,
            BeatTimeStep::Eighth(amount) => amount,
            BeatTimeStep::Beats(amount) => amount,
            BeatTimeStep::Half(amount) => amount,
            BeatTimeStep::Whole(amount) => amount,
            BeatTimeStep::Bar(amount) => amount,
        }
    }
    /// Set number of steps in the current time resolution.
    pub fn set_steps(&mut self, step: f32) {
        match *self {
            BeatTimeStep::SixtyFourth(_) => *self = BeatTimeStep::SixtyFourth(step),
            BeatTimeStep::ThirtySecond(_) => *self = BeatTimeStep::ThirtySecond(step),
            BeatTimeStep::Sixteenth(_) => *self = BeatTimeStep::Sixteenth(step),
            BeatTimeStep::Eighth(_) => *self = BeatTimeStep::Eighth(step),
            BeatTimeStep::Beats(_) => *self = BeatTimeStep::Beats(step),
            BeatTimeStep::Half(_) => *self = BeatTimeStep::Half(step),
            BeatTimeStep::Whole(_) => *self = BeatTimeStep::Whole(step),
            BeatTimeStep::Bar(_) => *self = BeatTimeStep::Bar(step),
        };
    }

    /// Get number of samples for a single step.
    pub fn samples_per_step(&self, time_base: &BeatTimeBase) -> f64 {
        match *self {
            BeatTimeStep::SixtyFourth(_) => time_base.samples_per_beat() / 16.0,
            BeatTimeStep::ThirtySecond(_) => time_base.samples_per_beat() / 8.0,
            BeatTimeStep::Sixteenth(_) => time_base.samples_per_beat() / 4.0,
            BeatTimeStep::Eighth(_) => time_base.samples_per_beat() / 2.0,
            BeatTimeStep::Beats(_) => time_base.samples_per_beat(),
            BeatTimeStep::Half(_) => time_base.samples_per_beat() * 2.0,
            BeatTimeStep::Whole(_) => time_base.samples_per_beat() * 4.0,
            BeatTimeStep::Bar(_) => time_base.samples_per_bar(),
        }
    }
    /// Convert a beat or bar step to samples for the given beat time base.
    #[inline]
    pub fn to_samples(&self, time_base: &BeatTimeBase) -> f64 {
        self.steps() as f64 * self.samples_per_step(time_base)
    }
}

impl Default for BeatTimeStep {
    fn default() -> Self {
        Self::Beats(0.0)
    }
}
