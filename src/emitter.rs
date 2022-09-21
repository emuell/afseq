use self::beat_time::BeatTimeEmitter;
use crate::EmitterValue;

pub mod beat_time;
pub mod beat_time_pattern;

// -------------------------------------------------------------------------------------------------

/// Beat & Bar time base
#[derive(Clone)]
pub struct BeatTimeBase {
    pub beats_per_min: f32,
    pub beats_per_bar: u32,
    pub samples_per_sec: u32,
}

impl BeatTimeBase {
    /// Convert beat to sample time and vice versa
    pub fn samples_per_beat(&self) -> f64 {
        self.samples_per_sec as f64 * 60.0 / self.beats_per_min as f64
    }
    /// Convert beat to sample time and vice versa
    pub fn samples_per_bar(&self) -> f64 {
        self.samples_per_sec as f64 * 60.0 / self.beats_per_min as f64 * self.beats_per_bar as f64
    }
}

// -------------------------------------------------------------------------------------------------

/// Beat & Bar Time Step: defines number of steps in either beats or bars.
pub enum BeatTimeStep {
    Beats(u32),
    Bar(u32),
}

impl BeatTimeStep {
    /// Convert beat time step to samples for the given beat time base.
    pub fn to_samples(&self, time_base: &BeatTimeBase) -> f64 {
        match *self {
            BeatTimeStep::Beats(amount) => time_base.samples_per_beat() * amount as f64,
            BeatTimeStep::Bar(amount) => time_base.samples_per_bar() * amount as f64,
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Shortcuts for creating beat time based emittors
impl BeatTimeBase {
    pub fn every_nth_beat<Value: EmitterValue + 'static>(
        &self,
        beats: u32,
        value: Value,
    ) -> BeatTimeEmitter {
        BeatTimeEmitter::new(self.clone(), BeatTimeStep::Beats(beats), value)
    }

    pub fn every_nth_beat_with_offset<Value: EmitterValue + 'static>(
        &self,
        beats: u32,
        offset: u32,
        value: Value,
    ) -> BeatTimeEmitter {
        BeatTimeEmitter::new_with_offset(
            self.clone(),
            BeatTimeStep::Beats(beats),
            BeatTimeStep::Beats(offset),
            value,
        )
    }

    pub fn every_nth_bar<Value: EmitterValue + 'static>(
        &self,
        bars: u32,
        value: Value,
    ) -> BeatTimeEmitter {
        BeatTimeEmitter::new(self.clone(), BeatTimeStep::Bar(bars), value)
    }
    pub fn every_nth_bar_with_offset<Value: EmitterValue + 'static>(
        &self,
        bars: u32,
        offset_in_beats: u32,
        value: Value,
    ) -> BeatTimeEmitter {
        BeatTimeEmitter::new_with_offset(
            self.clone(),
            BeatTimeStep::Bar(bars),
            BeatTimeStep::Beats(offset_in_beats),
            value,
        )
    }
}
