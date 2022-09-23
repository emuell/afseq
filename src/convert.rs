//! Conversion traits to help create and map `Events`.
//!
//! To enable all conversion helpers:
//! ```
//! # #![allow(unused_imports)]
//! use afseq::convert::*;
//! ```

use crate::event::{
    fixed::FixedEventIter,
    sequence::EventIterSequence,
    Event, NoteEvent, ParameterChangeEvent,
    {mapped::MappedEventIter, mapped_note::MappedNoteEventIter},
};

// -------------------------------------------------------------------------------------------------

pub trait ToFixedEventValue {
    fn to_event(self) -> FixedEventIter;
}

impl ToFixedEventValue for NoteEvent {
    /// Wrap a [`NoteEvent`] to a new [`FixedEventIter`].
    fn to_event(self) -> FixedEventIter {
        FixedEventIter::new(Event::NoteEvents(vec![self]))
    }
}

impl ToFixedEventValue for Vec<NoteEvent> {
    /// Wrap a vector of  [`NoteEvent`] to a new [`FixedEventIter`].
    fn to_event(self) -> FixedEventIter {
        FixedEventIter::new(Event::NoteEvents(self))
    }
}

impl ToFixedEventValue for ParameterChangeEvent {
    /// Wrap a [`ParameterChangeEvent`] into a new [`FixedEventIter`].
    fn to_event(self) -> FixedEventIter {
        FixedEventIter::new(Event::ParameterChangeEvent(self))
    }
}

// -------------------------------------------------------------------------------------------------

pub trait ToEventIterSequence {
    fn to_event_sequence(self) -> EventIterSequence;
}

impl ToEventIterSequence for Vec<NoteEvent> {
    /// Wrap a vector of  [`NoteEvent`] to a new [`EventIterSequence`].
    fn to_event_sequence(self) -> EventIterSequence {
        let mut sequence = Vec::new();
        for note in self {
            sequence.push(Event::NoteEvents(vec![note]));
        }
        EventIterSequence::new(sequence)
    }
}

impl ToEventIterSequence for Vec<Vec<NoteEvent>> {
    /// Wrap a vector of vectors of [`NoteEvent`] to a new [`EventIterSequence`].
    fn to_event_sequence(self) -> EventIterSequence {
        let mut sequence = Vec::new();
        for notes in self {
            sequence.push(Event::NoteEvents(notes));
        }
        EventIterSequence::new(sequence)
    }
}

// -------------------------------------------------------------------------------------------------

pub trait ToMappedEventIter<EventMap>
where
    EventMap: FnMut(Event) -> Event + Copy,
{
    fn map_events(self, map: EventMap) -> MappedEventIter<EventMap>;
}

impl<EventMap> ToMappedEventIter<EventMap> for FixedEventIter
where
    EventMap: FnMut(Event) -> Event + Copy,
{
    /// Upgrade a [`FixedEventIter`] to a [`MappedEventIter`].
    fn map_events(self, map: EventMap) -> MappedEventIter<EventMap> {
        MappedEventIter::new(self.event, map)
    }
}

// -------------------------------------------------------------------------------------------------

pub trait ToMappedNotesEventIter<NoteMap>
where
    NoteMap: FnMut(NoteEvent) -> NoteEvent,
{
    fn map_notes(self, map: NoteMap) -> MappedNoteEventIter<NoteMap>;
}

impl<NoteMap> ToMappedNotesEventIter<NoteMap> for FixedEventIter
where
    NoteMap: FnMut(NoteEvent) -> NoteEvent + Copy,
{
    /// Upgrade a [`FixedEventIter`] to a [`MappedNoteEventIter`].
    fn map_notes(self, map: NoteMap) -> MappedNoteEventIter<NoteMap> {
        MappedNoteEventIter::new(self.event, map)
    }
}
