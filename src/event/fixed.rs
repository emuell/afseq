use crate::event::{Event, EventIter};

// -------------------------------------------------------------------------------------------------

/// Endlessly emits a single, fixed [`Event`].
#[derive(Clone)]
pub struct FixedEventIter {
    pub(crate) event: Event,
}

impl FixedEventIter {
    pub fn new(event: Event) -> Self {
        Self { event }
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
