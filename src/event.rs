//! Value and value iters which get emitted by `Rhythms`.

use crate::midi::Note;
use fixed::{FixedEventIter, ToFixedEventIter, ToFixedEventIterSequence};

use derive_more::{Deref, Display, From, Into};

use core::sync::atomic::{AtomicUsize, Ordering};
use std::fmt;

pub mod empty;
pub mod fixed;
pub mod mutated;
#[cfg(feature = "scripting")]
pub mod scripted;

// -------------------------------------------------------------------------------------------------

/// Id to refer to a specific instrument in a NoteEvent.
#[derive(Copy, Clone, Debug, Display, Deref, From, Into, PartialEq, Eq, Hash)]
pub struct InstrumentId(usize);

/// Id to refer to a specific parameter in a ParameterChangeEvent.
#[derive(Copy, Clone, Debug, Display, Deref, From, Into, PartialEq, Eq, Hash)]
pub struct ParameterId(usize);

// -------------------------------------------------------------------------------------------------

/// Generate a new unique instrument id.
pub fn unique_instrument_id() -> InstrumentId {
    static ID: AtomicUsize = AtomicUsize::new(0);
    InstrumentId(ID.fetch_add(1, Ordering::Relaxed))
}

// -------------------------------------------------------------------------------------------------

/// Single note event in a [`Event`].
#[derive(Clone, PartialEq, Debug)]
pub struct NoteEvent {
    pub instrument: Option<InstrumentId>,
    pub note: Note,
    pub velocity: f32,
}

impl NoteEvent {
    pub fn to_string(&self, show_instruments: bool) -> String {
        if show_instruments {
            format!(
                "{} {} {:.2}",
                if let Some(instrument) = self.instrument {
                    format!("{:02}", instrument)
                } else {
                    "NA".to_string()
                },
                self.note,
                self.velocity
            )
        } else {
            format!("{} {:.3}", self.note, self.velocity)
        }
    }
}

/// Shortcut for creating an empty [`NoteEvent`] [`EventIter`].
pub fn new_empty_note() -> Option<NoteEvent> {
    None
}

/// Shortcut for creating a new [`NoteEvent`].
pub fn new_note<I: Into<Option<InstrumentId>>, N: Into<Note>>(
    instrument: I,
    note: N,
    velocity: f32,
) -> NoteEvent {
    let instrument: Option<InstrumentId> = instrument.into();
    let note: Note = note.into();
    NoteEvent {
        instrument,
        note,
        velocity,
    }
}

/// Shortcut for creating a vector of [`NoteEvent`]:
/// e.g. a sequence of single notes
pub fn new_note_vector<
    I: Into<Option<InstrumentId>>,
    N: Into<Note>,
    T: Into<Option<(I, N, f32)>>,
>(
    sequence: Vec<T>,
) -> Vec<Option<NoteEvent>> {
    let mut event_sequence = Vec::with_capacity(sequence.len());
    for event in sequence {
        if let Some((instrument, note, velocity)) = event.into() {
            let instrument = instrument.into();
            let note = note.into();
            event_sequence.push(Some(NoteEvent {
                instrument,
                note,
                velocity,
            }));
        } else {
            event_sequence.push(None);
        }
    }
    event_sequence
}

/// Shortcut for creating a new sequence of polyphonic [`NoteEvent`]:
/// e.g. a sequence of chords
pub fn new_polyphonic_note_sequence<
    I: Into<Option<InstrumentId>>,
    N: Into<Note>,
    T: Into<Option<(I, N, f32)>>,
>(
    polyphonic_sequence: Vec<Vec<T>>,
) -> Vec<Vec<Option<NoteEvent>>> {
    let mut polyphonic_event_sequence = Vec::with_capacity(polyphonic_sequence.len());
    for sequence in polyphonic_sequence {
        let mut event_sequence = Vec::with_capacity(sequence.len());
        for event in sequence {
            if let Some((instrument, note, velocity)) = event.into() {
                let instrument = instrument.into();
                let note = note.into();
                event_sequence.push(Some(NoteEvent {
                    instrument,
                    note,
                    velocity,
                }));
            } else {
                event_sequence.push(None)
            }
        }
        polyphonic_event_sequence.push(event_sequence);
    }
    polyphonic_event_sequence
}

/// Shortcut for creating a new empty [`NoteEvent`] [`EventIter`].
pub fn new_empty_note_event() -> FixedEventIter {
    new_empty_note().to_event()
}

/// Shortcut for creating a new [`NoteEvent`] [`EventIter`].
pub fn new_note_event<I: Into<Option<InstrumentId>>, N: Into<Note>>(
    instrument: I,
    note: N,
    velocity: f32,
) -> FixedEventIter {
    new_note(instrument, note, velocity).to_event()
}

/// Shortcut for creating a new sequence of [`NoteEvent`] [`EventIter`].
pub fn new_note_event_sequence<
    I: Into<Option<InstrumentId>>,
    N: Into<Note>,
    T: Into<Option<(I, N, f32)>>,
>(
    sequence: Vec<T>,
) -> FixedEventIter {
    new_note_vector(sequence).to_event_sequence()
}

/// Shortcut for creating a single [`EventIter`] from multiple [`NoteEvent`]:
/// e.g. a chord.
pub fn new_polyphonic_note_event<I: Into<Option<InstrumentId>>, N: Into<Note>>(
    polyphonic_events: Vec<Option<(I, N, f32)>>,
) -> FixedEventIter {
    new_note_vector(polyphonic_events).to_event()
}

/// Shortcut for creating a single [`EventIter`] from multiple [`NoteEvent`]:
/// e.g. a sequence of chords.
pub fn new_polyphonic_note_sequence_event<
    I: Into<Option<InstrumentId>>,
    N: Into<Note>,
    T: Into<Option<(I, N, f32)>>,
>(
    polyphonic_sequence: Vec<Vec<T>>,
) -> FixedEventIter {
    new_polyphonic_note_sequence(polyphonic_sequence).to_event_sequence()
}

// -------------------------------------------------------------------------------------------------

/// Single parameter change event in a [`Event`].
#[derive(Clone, PartialEq, Debug)]
pub struct ParameterChangeEvent {
    pub parameter: Option<ParameterId>,
    pub value: f32,
}

impl ParameterChangeEvent {
    pub fn to_string(&self, show_parameter: bool) -> String {
        if show_parameter {
            format!(
                "{} {:.3}",
                if let Some(parameter) = self.parameter {
                    format!("{:02}", parameter)
                } else {
                    "NA".to_string()
                },
                self.value,
            )
        } else {
            format!("{:.3}", self.value)
        }
    }
}

/// Shortcut for creating a new [`ParameterChangeEvent`].
pub fn new_parameter_change<Parameter: Into<Option<ParameterId>>>(
    parameter: Parameter,
    value: f32,
) -> ParameterChangeEvent {
    let parameter: Option<ParameterId> = parameter.into();
    ParameterChangeEvent { parameter, value }
}

/// Shortcut for creating a new [`ParameterChangeEvent`] [`EventIter`].
pub fn new_parameter_change_event<Parameter: Into<Option<ParameterId>>>(
    parameter: Parameter,
    value: f32,
) -> FixedEventIter {
    new_parameter_change(parameter, value).to_event()
}

// -------------------------------------------------------------------------------------------------

/// Event which gets emitted by an [`EventIter`].
#[derive(Clone, PartialEq, Debug)]
pub enum Event {
    NoteEvents(Vec<Option<NoteEvent>>),
    ParameterChangeEvent(ParameterChangeEvent),
}

impl Event {
    pub fn to_string(&self, show_instruments_and_parameters: bool) -> String {
        match self {
            Event::NoteEvents(note_vector) => note_vector
                    .iter()
                    .map(|n| if let Some(v) = n {
                        v.to_string(show_instruments_and_parameters)
                    } else {
                        "---".to_string()
                    })
                    .collect::<Vec<_>>()
                    .join(" | "),
            Event::ParameterChangeEvent(change) => {
                change.to_string(show_instruments_and_parameters)
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// A resettable [`Event`] iterator, which typically will be used in
/// [Rhythm](`super::Rhythm`) trait impls to sequencially emit new events.
pub trait EventIter: Iterator<Item = Event> {
    /// Reset/rewind the iterator to its initial state.
    fn reset(&mut self);
}

// -------------------------------------------------------------------------------------------------

impl fmt::Display for NoteEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const SHOW_INSTRUMENTS: bool = true;
        f.write_fmt(format_args!("{}", self.to_string(SHOW_INSTRUMENTS)))
    }
}

impl fmt::Display for ParameterChangeEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const SHOW_PARAMETERS: bool = true;
        f.write_fmt(format_args!("{}", self.to_string(SHOW_PARAMETERS)))
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const SHOW_INSTRUMENTS_AND_PARAMETERS: bool = true;
        f.write_fmt(format_args!("{}", self.to_string(SHOW_INSTRUMENTS_AND_PARAMETERS)))
    }
}
