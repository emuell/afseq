use crate::events::{PatternEvent, PatternEventIter};

// -------------------------------------------------------------------------------------------------

/// Endlessly emits a sequence of [`PatternEvent`].
#[derive(Clone)]
pub struct PatternEventIterSequence {
    events: Vec<PatternEvent>,
    current: usize,
}

impl PatternEventIterSequence {
    pub fn new(events: Vec<PatternEvent>) -> Self {
        let current = 0;
        Self { events, current }
    }
}

impl Iterator for PatternEventIterSequence {
    type Item = PatternEvent;

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

impl PatternEventIter for PatternEventIterSequence {
    fn reset(&mut self) {
        self.current = 0;
    }
}
