use crate::event::{Event, EventIter, NoteEvent};

// -------------------------------------------------------------------------------------------------

/// Endlessly emits a sequence of [`Event`].
#[derive(Clone)]
pub struct EventIterSequence {
    events: Vec<Event>,
    current: usize,
}

impl EventIterSequence {
    pub fn new(events: Vec<Event>) -> Self {
        let current = 0;
        Self { events, current }
    }
}

impl Iterator for EventIterSequence {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        if self.events.is_empty() {
            return None;
        }
        let current = self.events[self.current].clone();
        self.current += 1;
        if self.current >= self.events.len() {
            self.current = 0;
        }
        Some(current)
    }
}

impl EventIter for EventIterSequence {
    fn reset(&mut self) {
        self.current = 0;
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
