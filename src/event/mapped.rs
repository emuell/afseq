use crate::event::{Event, EventIter};

// -------------------------------------------------------------------------------------------------

/// Endlessly emits [`Event`] which's value can be mutated/mapped in each iter step
/// with a custom map function.
#[derive(Clone)]
pub struct MappedEventIter<F>
where
    F: FnMut(Event) -> Event,
{
    event: Event,
    initial_event: Event,
    map: F,
}

impl<F> MappedEventIter<F>
where
    F: FnMut(Event) -> Event,
{
    pub fn new(event: Event, map: F) -> Self {
        let initial_event = event.clone();
        Self {
            event,
            initial_event,
            map,
        }
    }
}

impl<F> Iterator for MappedEventIter<F>
where
    F: FnMut(Event) -> Event,
{
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.event.clone();
        self.event = (self.map)(self.event.clone());
        Some(current)
    }
}

impl<F> EventIter for MappedEventIter<F>
where
    F: FnMut(Event) -> Event,
{
    fn reset(&mut self) {
        self.event = self.initial_event.clone();
    }
}
