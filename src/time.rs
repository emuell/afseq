//! The time base that `Rhythms` can run on.

mod beats;
pub use beats::{BeatTimeBase, BeatTimeStep};
mod seconds;
pub use seconds::{SecondTimeBase, SecondTimeStep};

// -------------------------------------------------------------------------------------------------

/// Sample time value type emitted by the [Rhythm](`crate::Rhythm`) trait.
pub type SampleTime = u64;
