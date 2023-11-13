//! The afseq prelude.
//!
//! The purpose of this module is to alleviate imports of common afseq traits:
//!
//! ```
//! # #![allow(unused_imports)]
//! use afseq::prelude::*;
//! ```

pub use super::{
    event::{
        fixed::ToFixedEventIter, fixed::ToFixedEventIterSequence, mutated::ToMutatedEventIter,
        new_empty_note, new_empty_note_event, new_note, new_note_event, new_note_event_sequence,
        new_parameter_change_event, new_polyphonic_note_event, new_polyphonic_note_sequence_event,
        unique_instrument_id, InstrumentId, NoteEvent, ParameterChangeEvent, ParameterId,
    },
    note::Note,
    phrase::{Phrase, RhythmSlot},
    rhythm::{beat_time::BeatTimeRhythm, second_time::SecondTimeRhythm},
    time::{BeatTimeBase, BeatTimeStep, SecondTimeBase, SecondTimeStep, TimeBase},
    Chord, Event, EventIter, Rhythm, SampleTime, Sequence,
};

#[cfg(feature = "scripting")]
pub use super::bindings::{self};

#[cfg(feature = "player")]
pub use super::player::{NewNoteAction, SamplePlaybackContext, SamplePlayer, SamplePool};
