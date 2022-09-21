use crate::{EmitterEvent, EmitterValue};

// -------------------------------------------------------------------------------------------------

/// Emits events which can be mutated/mapped in each iter step with a custom map function.
#[derive(Clone)]
pub struct MappedEmitterValue<F>
where
    F: FnMut(&EmitterEvent) -> EmitterEvent,
{
    event: EmitterEvent,
    initial_event: EmitterEvent,
    map: F,
}

impl<F> MappedEmitterValue<F>
where
    F: FnMut(&EmitterEvent) -> EmitterEvent,
{
    pub fn new(event: EmitterEvent, map: F) -> Self {
        let initial_event = event.clone();
        Self {
            event,
            initial_event,
            map,
        }
    }
}

impl<F> Iterator for MappedEmitterValue<F>
where
    F: FnMut(&EmitterEvent) -> EmitterEvent,
{
    type Item = EmitterEvent;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.event.clone();
        self.event = (self.map)(&self.event);
        Some(current)
    }
}

impl<F> EmitterValue for MappedEmitterValue<F>
where
    F: FnMut(&EmitterEvent) -> EmitterEvent,
{
    fn reset(&mut self) {
        self.event = self.initial_event.clone();
    }
}
