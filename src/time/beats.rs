use crate::{rhythm::beat_time::BeatTimeRhythm, time::TimeBase};

// -------------------------------------------------------------------------------------------------

/// Beat & bar timing base for beat based [Rhythm](`crate::Rhythm`) impls.
#[derive(Copy, Clone)]
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

// -------------------------------------------------------------------------------------------------

/// Defines a number of steps in sixteenth, beat or bar amounts.
#[derive(Copy, Clone)]
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
    /// Get number of samples for a single step.
    pub fn samples_per_step(&self, time_base: &BeatTimeBase) -> f64 {
        match *self {
            BeatTimeStep::Sixteenth(_) => time_base.samples_per_beat() / 4.0,
            BeatTimeStep::Eighth(_) => time_base.samples_per_beat() / 2.0,
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
