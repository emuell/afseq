//! Defines musical or wall clock time bases that `Patterns` can run on.

use std::fmt::Debug;

mod beats;
pub use beats::{BeatTimeBase, BeatTimeStep};

mod seconds;
pub use seconds::{SecondTimeBase, SecondTimeStep};

// -------------------------------------------------------------------------------------------------

/// Sample time value type as emitted by
/// [`Pattern`](crate::Pattern) and [`Rhythm`](crate::Rhythm).
pub type SampleTime = u64;

/// Sample time as real number value, used to keep track of other units as sample time.
pub type ExactSampleTime = f64;

// -------------------------------------------------------------------------------------------------

/// Convert sample times to strings in [`SampleTimeBase`] impls.
pub trait SampleTimeDisplay: Debug {
    /// generate a string representation of the the given sample time
    fn display(&self, sample_time: SampleTime) -> String;
}

// -------------------------------------------------------------------------------------------------

/// Root time trait, providing sample <-> second rate conversion only.
pub trait SampleTimeBase: Debug {
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
