//! An experimental functional musical sequence generator.
//! Part of the [afplay](https://github.com/emuell/afplay) crates.

#![warn(clippy::clone_on_ref_ptr)]

pub mod time;
pub use time::{BeatTimeBase, SampleTime, SecondTimeBase};

pub mod note;
pub use note::Note;

pub mod chord;
pub use chord::Chord;

pub mod scale;
pub use scale::Scale;

pub mod event;
pub use event::{Event, EventIter};

pub mod pulse;
pub use pulse::{Pulse, PulseIter, PulseIterItem};

pub mod pattern;
pub use pattern::Pattern;

pub mod gate;
pub use gate::Gate;

pub mod rhythm;
pub use rhythm::{Rhythm, RhythmSampleIter};

pub mod phrase;
pub use phrase::Phrase;

pub mod sequence;
pub use sequence::Sequence;

#[cfg(feature = "scripting")]
pub mod bindings;

#[cfg(feature = "player")]
pub mod player;

pub mod prelude;
