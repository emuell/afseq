use std::{cell::RefCell, rc::Rc};

use crate::{
    event::{Event, EventIter, NoteEvent, ParameterChangeEvent},
    BeatTimeBase, PulseIterItem,
};

// -------------------------------------------------------------------------------------------------

/// Endlessly emits a single, fixed [`Event`].
#[derive(Clone, Debug)]
pub struct FixedEventIter {
    events: Vec<Event>,
    current: usize,
}

impl FixedEventIter {
    pub fn new(events: Vec<Event>) -> Self {
        let current = 0;
        Self { events, current }
    }

    // Get a copy of the event that we're triggering
    pub fn events(&self) -> Vec<Event> {
        self.events.clone()
    }
}

impl Iterator for FixedEventIter {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        if self.events.is_empty() {
            return None;
        }
        let event = self.events[self.current].clone();
        self.current += 1;
        if self.current >= self.events.len() {
            self.current = 0;
        }
        Some(event)
    }
}

impl EventIter for FixedEventIter {
    fn set_time_base(&mut self, _time_base: &BeatTimeBase) {
        // nothing to do
    }

    fn set_context(&mut self, _context: PulseIterItem, _pulse_count: usize) {
        // nothing to do
    }

    fn duplicate(&self) -> Rc<RefCell<dyn EventIter>> {
        Rc::new(RefCell::new(self.clone()))
    }

    fn reset(&mut self) {
        // reset step counter
        self.current = 0;
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
