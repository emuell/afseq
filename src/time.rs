mod beats;
pub use beats::{BeatTimeBase, BeatTimeStep};

// -------------------------------------------------------------------------------------------------

/// Sample time value type emitted by the [Pattern](`crate::Pattern`) trait.
pub type SampleTime = u64;
