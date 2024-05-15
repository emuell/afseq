use std::borrow::Cow;

use crate::{
    event::{Event, EventIter, NoteEvent, ParameterChangeEvent},
    BeatTimeBase, Note, PulseIterItem,
};

// -------------------------------------------------------------------------------------------------

/// Endlessly emits a single, fixed [`Event`].
#[derive(Clone, Debug)]
pub struct FixedEventIter {
    events: Vec<Event>,
    event_index: usize,
}

impl FixedEventIter {
    pub fn new(events: Vec<Event>) -> Self {
        let event_index = 0;
        Self {
            events,
            event_index,
        }
    }

    // Get a copy of the event that we're triggering
    pub fn events(&self) -> Vec<Event> {
        self.events.clone()
    }
}

impl Default for FixedEventIter {
    fn default() -> Self {
        Self::new(vec![Event::NoteEvents(vec![Some((Note::C4).into())])])
    }
}
impl EventIter for FixedEventIter {
    fn set_time_base(&mut self, _time_base: &BeatTimeBase) {
        // nothing to do
    }

    fn set_external_context(&mut self, _data: &[(Cow<str>, f64)]) {
        // nothing to do
    }

    fn run(
        &mut self,
        _pulse: PulseIterItem,
        _pulse_pattern_length: usize,
        emit_event: bool,
    ) -> Option<Vec<Event>> {
        if !emit_event || self.events.is_empty() {
            return None;
        }
        let event = self.events[self.event_index].clone();
        self.event_index += 1;
        if self.event_index >= self.events.len() {
            self.event_index = 0;
        }
        Some(vec![event])
    }

    fn duplicate(&self) -> Box<dyn EventIter> {
        Box::new(self.clone())
    }

    fn reset(&mut self) {
        // reset step counter
        self.event_index = 0;
    }
}

// -------------------------------------------------------------------------------------------------

pub trait ToFixedEventIter {
    fn to_event(self) -> FixedEventIter;
}

impl ToFixedEventIter for NoteEvent {
    /// Wrap a [`NoteEvent`] to a new [`FixedEventIter`]
    /// resulting into a single monophonic event.
    fn to_event(self) -> FixedEventIter {
        FixedEventIter::new(vec![Event::NoteEvents(vec![Some(self)])])
    }
}
impl ToFixedEventIter for Option<NoteEvent> {
    /// Wrap a [`NoteEvent`] to a new [`FixedEventIter`]
    /// resulting into a single monophonic event.
    fn to_event(self) -> FixedEventIter {
        FixedEventIter::new(vec![Event::NoteEvents(vec![self])])
    }
}

impl ToFixedEventIter for Vec<NoteEvent> {
    /// Wrap a vector of [`NoteEvent`] to a new [`FixedEventIter`].
    /// resulting into a single polyphonic event.
    fn to_event(self) -> FixedEventIter {
        FixedEventIter::new(vec![Event::NoteEvents(
            self.iter().map(|v| Some(v.clone())).collect::<Vec<_>>(),
        )])
    }
}
impl ToFixedEventIter for Vec<Option<NoteEvent>> {
    /// Wrap a vector of [`NoteEvent`] to a new [`FixedEventIter`].
    /// resulting into a single polyphonic event.
    fn to_event(self) -> FixedEventIter {
        FixedEventIter::new(vec![Event::NoteEvents(self)])
    }
}

impl ToFixedEventIter for ParameterChangeEvent {
    /// Wrap a [`ParameterChangeEvent`] into a new [`FixedEventIter`].
    fn to_event(self) -> FixedEventIter {
        FixedEventIter::new(vec![Event::ParameterChangeEvent(self)])
    }
}

// -------------------------------------------------------------------------------------------------

pub trait ToFixedEventIterSequence {
    fn to_event_sequence(self) -> FixedEventIter;
}

impl ToFixedEventIterSequence for Vec<Option<NoteEvent>> {
    /// Wrap a vector of [`NoteEvent`] to a new [`FixedEventIter`]
    /// resulting into a sequence of single note events.
    fn to_event_sequence(self) -> FixedEventIter {
        let mut sequence = Vec::with_capacity(self.len());
        for note in self {
            sequence.push(Event::NoteEvents(vec![note]));
        }
        FixedEventIter::new(sequence)
    }
}

impl ToFixedEventIterSequence for Vec<Vec<Option<NoteEvent>>> {
    /// Wrap a vector of vectors of [`NoteEvent`] to a new [`FixedEventIter`]
    /// resulting into a sequence of polyphonic note events.
    fn to_event_sequence(self) -> FixedEventIter {
        let mut sequence = Vec::with_capacity(self.len());
        for notes in self {
            sequence.push(Event::NoteEvents(notes));
        }
        FixedEventIter::new(sequence)
    }
}

impl ToFixedEventIterSequence for Vec<ParameterChangeEvent> {
    /// Wrap a [`ParameterChangeEvent`] into a new [`FixedEventIter`]
    fn to_event_sequence(self) -> FixedEventIter {
        let mut sequence = Vec::with_capacity(self.len());
        for p in self {
            sequence.push(Event::ParameterChangeEvent(p));
        }
        FixedEventIter::new(sequence)
    }
}
