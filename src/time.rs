mod beats;
pub use beats::{BeatTimeBase, BeatTimeStep};

// -------------------------------------------------------------------------------------------------

/// Sample time value type emitted by the [Rhythm](`crate::Rhythm`) trait.
pub type SampleTime = u64;
