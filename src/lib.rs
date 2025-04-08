//! An experimental functional musical sequence generator.
//! Part of the [afplay](https://github.com/emuell/afplay) crates.

// -------------------------------------------------------------------------------------------------

// Clippy lints

#![warn(clippy::clone_on_ref_ptr)]
// Useful, but also annoying: enable and check every now and then
// #![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_lossless)]

// -------------------------------------------------------------------------------------------------

#[cfg(all(
    feature = "scripting",
    not(any(
        feature = "lua",
        feature = "lua-jit",
        feature = "luau",
        feature = "luau-jit"
    ))
))]
compile_error!(
    "When enabling the `scripting` feature, enable one of the lua interpreter features as well: `lua`, `lua-jit`, `luau` or `lua-jit`"
);

// -------------------------------------------------------------------------------------------------

// Exports

pub mod time;
pub use time::{
    BeatTimeBase, BeatTimeStep, SampleTime, SampleTimeDisplay, SecondTimeBase, TimeBase,
};

pub mod note;
pub use note::Note;

pub mod chord;
pub use chord::Chord;

pub mod scale;
pub use scale::Scale;

pub mod event;
pub use event::{
    Event, EventIter, EventIterItem, InstrumentId, NoteEvent, ParameterChangeEvent, ParameterId,
};

pub mod tidal;
pub use tidal::{
    Cycle, Event as CycleEvent, PropertyKey as CyclePropertyKey,
    PropertyValue as CyclePropertyValue, Span as CycleSpan, Target as CycleTarget,
    Value as CycleValue,
};

pub mod pulse;
pub use pulse::{Pulse, PulseIter, PulseIterItem};

pub mod pattern;
pub use pattern::Pattern;

pub mod gate;
pub use gate::Gate;

pub mod rhythm;
pub use rhythm::{Rhythm, RhythmIter, RhythmIterItem};

pub mod phrase;
pub use phrase::{Phrase, RhythmSlot};

pub mod sequence;
pub use sequence::Sequence;

pub mod input;
pub use input::{
    Parameter as InputParameter, ParameterSet as InputParameterSet,
    ParameterType as InputParameterType,
};

#[cfg(feature = "scripting")]
pub mod bindings;

#[cfg(feature = "player")]
pub mod player;

pub mod prelude;
