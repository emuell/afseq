use crate::event::{Event, EventIter};

// -------------------------------------------------------------------------------------------------

/// Emits an empty, None [`Event`].
#[derive(Clone)]
pub struct EmptyEventIter {}

impl Iterator for EmptyEventIter {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

impl EventIter for EmptyEventIter {
    fn reset(&mut self) {}
}
