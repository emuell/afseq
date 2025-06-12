//! Beat time based `Pattern` implementation.

use crate::{
    pattern::generic::{GenericPattern, GenericPatternTimeStep},
    time::BeatTimeStep,
    BeatTimeBase,
};

// -------------------------------------------------------------------------------------------------

impl GenericPatternTimeStep for BeatTimeStep {
    #[inline]
    fn default_offset() -> Self {
        Self::Beats(0.0)
    }

    #[inline]
    fn default_step() -> Self {
        Self::Beats(1.0)
    }

    #[inline]
    fn to_samples(&self, time_base: &BeatTimeBase) -> f64 {
        BeatTimeStep::to_samples(self, time_base)
    }
}

// -------------------------------------------------------------------------------------------------

/// A Pattern with a beat time offset and beat time step.
pub type BeatTimePattern = GenericPattern<BeatTimeStep, BeatTimeStep>;

// -------------------------------------------------------------------------------------------------

macro_rules! generate_step_funcs {
    ($name:ident, $type:expr) => {
        paste::paste! {
            pub fn [<every_nth_ $name>](
                &self,
                step: f32,
            ) -> BeatTimePattern {
                self.every_nth_step($type(step))
            }
        }
    };
}

/// Shortcuts for creating beat-time based patterns.
impl BeatTimeBase {
    pub fn every_nth_step(&self, step: BeatTimeStep) -> BeatTimePattern {
        BeatTimePattern::new(*self, step)
    }
    generate_step_funcs!(sixteenth, BeatTimeStep::Sixteenth);
    generate_step_funcs!(eighth, BeatTimeStep::Eighth);
    generate_step_funcs!(beat, BeatTimeStep::Beats);
    generate_step_funcs!(half, BeatTimeStep::Half);
    generate_step_funcs!(bar, BeatTimeStep::Bar);
}
