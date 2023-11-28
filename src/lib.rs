//! An experimental functional musical sequence generator.
//! Part of the [afplay](https://github.com/emuell/afplay) crates.

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

pub mod rhythm;
pub use rhythm::Rhythm;

pub mod phrase;
pub use phrase::Phrase;

pub mod sequence;
pub use sequence::Sequence;

pub mod prelude;

#[cfg(feature = "scripting")]
pub mod bindings;

#[cfg(feature = "player")]
pub mod player;
