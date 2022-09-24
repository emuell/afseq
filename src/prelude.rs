//! The afseq prelude.
//!
//! The purpose of this module is to alleviate imports of common afseq traits:
//!
//! ```
//! # #![allow(unused_imports)]
//! use afseq::prelude::*;
//! ```

pub use super::convert::*;

pub use super::{
    event::{new_note_event, new_parameter_change_event, InstrumentId, ParameterId},
    time::{BeatTimeBase, BeatTimeStep, SecondTimeBase, SecondTimeStep, TimeBase},
    Event, EventIter, Phrase, Rhythm, SampleTime,
};
