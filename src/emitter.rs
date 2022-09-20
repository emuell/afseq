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
}

// -------------------------------------------------------------------------------------------------

/// Beat & Bar Time Step: defines number of steps in either beats or bars.
pub enum BeatTimeStep {
    Beats(u32),
    Bar(u32),
}
