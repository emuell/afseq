//! An experimental functional musical sequence generator.
//! Part of the [afplay](https://github.com/emuell/afplay) crates.

pub mod time;
pub use time::{BeatTimeBase, SampleTime, SecondTimeBase};

pub mod midi;
pub use midi::Note;

pub mod event;
pub use event::{Event, EventIter};

pub mod rhythm;
pub use rhythm::Rhythm;

pub mod phrase;
pub use phrase::Phrase;
pub mod prelude;

#[cfg(feature = "scripting")]
pub mod bindings;
