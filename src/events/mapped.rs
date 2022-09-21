use crate::events::{PatternEvent, PatternEventIter};

// -------------------------------------------------------------------------------------------------

/// Endlessly emits [`PatternEvent`] which's value can be mutated/mapped in each iter step
/// with a custom map function.
#[derive(Clone)]
pub struct MappedPatternEventIter<F>
where
    F: FnMut(PatternEvent) -> PatternEvent,
{
    event: PatternEvent,
    initial_event: PatternEvent,
    map: F,
}

impl<F> MappedPatternEventIter<F>
where
    F: FnMut(PatternEvent) -> PatternEvent,
{
    pub fn new(event: PatternEvent, map: F) -> Self {
        let initial_event = event.clone();
        Self {
            event,
            initial_event,
            map,
        }
    }
}

impl<F> Iterator for MappedPatternEventIter<F>
where
    F: FnMut(PatternEvent) -> PatternEvent,
{
    type Item = PatternEvent;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.event.clone();
        self.event = (self.map)(self.event.clone());
        Some(current)
    }
}

impl<F> PatternEventIter for MappedPatternEventIter<F>
where
    F: FnMut(PatternEvent) -> PatternEvent,
{
    fn reset(&mut self) {
        self.event = self.initial_event.clone();
    }
}
