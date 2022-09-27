use crate::event::{Event, EventIter, NoteEvent, ParameterChangeEvent};

// -------------------------------------------------------------------------------------------------

/// Endlessly emits a single, fixed [`Event`].
#[derive(Clone)]
pub struct FixedEventIter {
    event: Event,
}

impl FixedEventIter {
    pub fn new(event: Event) -> Self {
        Self { event }
    }

    // Get a copy of the event that we're triggering
    pub fn event(&self) -> Event {
        self.event.clone()
    }
}

impl Iterator for FixedEventIter {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.event.clone())
    }
}

impl EventIter for FixedEventIter {
    fn reset(&mut self) {
        // fixed values don't change, so there's nothing to reset
    }
}

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
    /// Wrap a vector of [`NoteEvent`] to a new [`FixedEventIter`].
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
