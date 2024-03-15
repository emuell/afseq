//! The afseq prelude.
//!
//! The purpose of this module is to alleviate imports of common afseq traits:
//!
//! ```
//! # #![allow(unused_imports)]
//! use afseq::prelude::*;
//! ```

pub use super::{
    // all types to create event iters, gates and patterns
    event::{
        fixed::ToFixedEventIter, fixed::ToFixedEventIterSequence, mutated::ToMutatedEventIter,
        new_empty_note, new_empty_note_event, new_note, new_note_event, new_note_event_sequence,
        new_parameter_change_event, new_polyphonic_note_event, new_polyphonic_note_sequence_event,
        unique_instrument_id, InstrumentId, NoteEvent, ParameterChangeEvent, ParameterId,
    },
    gate::ProbabilityGate,
    pattern::{euclidean, fixed::ToFixedPattern},
    phrase::RhythmSlot,
    rhythm::{beat_time::BeatTimeRhythm, second_time::SecondTimeRhythm},
    time::{BeatTimeStep, SecondTimeStep},
    // all basic types
    BeatTimeBase,
    Chord,
    Event,
    EventIter,
    Gate,
    Note,
    Pattern,
    Phrase,
    Pulse,
    PulseIter,
    PulseIterItem,
    Rhythm,
    RhythmIter,
    RhythmIterItem,
    SampleTime,
    Scale,
    SecondTimeBase,
    Sequence,
    TimeBase,
};

#[cfg(feature = "scripting")]
// all scripting types
pub use super::{
    bindings::{self},
    event::scripted::ScriptedEventIter,
    pattern::scripted::ScriptedPattern,
};

#[cfg(feature = "player")]
// all player types
pub use super::player::{NewNoteAction, SamplePlaybackContext, SamplePlayer, SamplePool};
