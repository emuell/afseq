//! The afseq prelude.
//!
//! The purpose of this module is to alleviate imports of common afseq traits:
//!
//! ```
//! # #![allow(unused_imports)]
//! use afseq::prelude::*;
//! ```

pub use super::{
    // all public types to create event iters, gates and patterns
    event::{
        cycle::{new_cycle_event, new_cycle_event_with_seed, CycleEventIter},
        fixed::ToFixedEventIter,
        fixed::ToFixedEventIterSequence,
        mutated::ToMutatedEventIter,
        new_empty_note, new_empty_note_event, new_note, new_note_event, new_note_event_sequence,
        new_parameter_change_event, new_polyphonic_note_event, new_polyphonic_note_sequence_event,
        unique_instrument_id, InstrumentId, NoteEvent, ParameterChangeEvent, ParameterId,
    },
    gate::probability::ProbabilityGate,
    pattern::{euclidean, fixed::ToFixedPattern},
    phrase::RhythmSlot,
    rhythm::{beat_time::BeatTimeRhythm, second_time::SecondTimeRhythm},
    time::{BeatTimeStep, SecondTimeStep},
    // all public basic types
    BeatTimeBase,
    Chord,
    Event,
    EventIter,
    EventIterItem,
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
// all public scripting types
pub use super::{
    bindings::{
        clear_lua_callback_errors, has_lua_callback_errors, lua_callback_errors,
        new_rhythm_from_file, new_rhythm_from_string,
    },
    event::{scripted::ScriptedEventIter, scripted_cycle::ScriptedCycleEventIter},
    gate::scripted::ScriptedGate,
    pattern::scripted::ScriptedPattern,
};

#[cfg(feature = "player")]
// all public player types
pub use super::player::{NewNoteAction, SamplePlaybackContext, SamplePlayer, SamplePool};
