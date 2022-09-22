use crate::event::{Event, EventIter};

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
