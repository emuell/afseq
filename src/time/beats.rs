use crate::{
    rhythm::beat_time::BeatTimeRhythm,
    time::{SampleTimeDisplay, TimeBase},
    SampleTime,
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
    pub fn samples_per_beat(&self) -> f64 {
        self.samples_per_sec as f64 * 60.0 / self.beats_per_min as f64
    }
    /// Time base's samples per bar, in order to convert bar to sample time and vice versa.
    pub fn samples_per_bar(&self) -> f64 {
        self.samples_per_sec as f64 * 60.0 / self.beats_per_min as f64 * self.beats_per_bar as f64
    }
}

impl TimeBase for BeatTimeBase {
    fn samples_per_second(&self) -> u32 {
        self.samples_per_sec
    }
}

impl SampleTimeDisplay for BeatTimeBase {
    /// generate a bar.beat.ppq string representation of the the given sample time
    fn display(&self, sample_time: SampleTime) -> String {
        let total_beats = sample_time / self.samples_per_beat() as u64;
        let total_beats_f = sample_time as f64 / self.samples_per_beat();
        let beat_frations = total_beats_f - total_beats as f64;
        let bars = total_beats / self.beats_per_bar as u64;
        let beats = total_beats - self.beats_per_bar as u64 * bars;
        let ppq = (beat_frations * 960.0 + 0.5) as u64;
        format!("{}.{}.{:03}", bars + 1, beats + 1, ppq)
    }
}

// -------------------------------------------------------------------------------------------------

/// Defines a number of steps in sixteenth, beat or bar amounts.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum BeatTimeStep {
    Sixteenth(f32),
    Eighth(f32),
    Beats(f32),
    Bar(f32),
}

impl BeatTimeStep {
    /// Get number of steps in the current time range.
    pub fn steps(&self) -> f32 {
        match *self {
            BeatTimeStep::Sixteenth(amount) => amount,
            BeatTimeStep::Eighth(amount) => amount,
            BeatTimeStep::Beats(amount) => amount,
            BeatTimeStep::Bar(amount) => amount,
        }
    }
    /// Set number of steps in the current time range.
    pub fn set_steps(&mut self, step: f32) {
        match *self {
            BeatTimeStep::Sixteenth(_) => *self = BeatTimeStep::Sixteenth(step),
            BeatTimeStep::Eighth(_) => *self = BeatTimeStep::Eighth(step),
            BeatTimeStep::Beats(_) => *self = BeatTimeStep::Beats(step),
            BeatTimeStep::Bar(_) => *self = BeatTimeStep::Bar(step),
        };
    }

    /// Get number of samples for a single step.
    pub fn samples_per_step(&self, time_base: &BeatTimeBase) -> f64 {
        match *self {
            BeatTimeStep::Sixteenth(_) => time_base.samples_per_beat() / 4.0,
            BeatTimeStep::Eighth(_) => time_base.samples_per_beat() / 2.0,
            BeatTimeStep::Beats(_) => time_base.samples_per_beat(),
            BeatTimeStep::Bar(_) => time_base.samples_per_bar(),
        }
    }
    /// Convert a beat or bar step to samples for the given beat time base.
    pub fn to_samples(&self, time_base: &BeatTimeBase) -> f64 {
        self.steps() as f64 * self.samples_per_step(time_base)
    }
}

// -------------------------------------------------------------------------------------------------

macro_rules! generate_step_funcs {
    ($name:ident, $type:expr) => {
        paste::paste! {
            pub fn [<every_nth_ $name>](
                &self,
                step: f32,
            ) -> BeatTimeRhythm {
                self.every_nth_step($type(step))
            }
        }
    };
}

/// Shortcuts for creating beat-time based patterns.
impl BeatTimeBase {
    pub fn every_nth_step(&self, step: BeatTimeStep) -> BeatTimeRhythm {
        BeatTimeRhythm::new(*self, step)
    }
    generate_step_funcs!(sixteenth, BeatTimeStep::Sixteenth);
    generate_step_funcs!(eighth, BeatTimeStep::Eighth);
    generate_step_funcs!(beat, BeatTimeStep::Beats);
    generate_step_funcs!(bar, BeatTimeStep::Bar);
}
