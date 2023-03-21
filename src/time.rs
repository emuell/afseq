//! The time base that `Rhythms` can run on.

mod beats;
pub use beats::{BeatTimeBase, BeatTimeStep};

mod seconds;
pub use seconds::{SecondTimeBase, SecondTimeStep};

// -------------------------------------------------------------------------------------------------

/// Sample time value type emitted by the [Rhythm](`crate::Rhythm`) trait.
pub type SampleTime = u64;

// -------------------------------------------------------------------------------------------------

/// Displays sample times as strings in various formats
pub trait SampleTimeDisplay {
    /// generate a string representation of the the given sample time
    fn display(&self, sample_time: SampleTime) -> String;
}

// -------------------------------------------------------------------------------------------------

/// Basic time trait, providing samples <-> second rate conversion only.
pub trait TimeBase {
    /// Sample rate for the time base.  
    fn samples_per_second(&self) -> u32;

    /// Convert given sample amount in seconds, using this time bases' samples per second rate.
    fn samples_to_seconds(&self, samples: SampleTime) -> f64 {
        samples as f64 / self.samples_per_second() as f64
    }
    /// Convert given second duration in samples, using this time bases' samples per second rate.
    fn seconds_to_samples(&self, seconds: f64) -> SampleTime {
        (seconds * self.samples_per_second() as f64).trunc() as SampleTime
    }
}
