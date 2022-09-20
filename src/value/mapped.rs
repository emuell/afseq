use crate::{EmitterEvent, EmitterValue};

// -------------------------------------------------------------------------------------------------

/// Emits events which can be mutated/mapped in each iter step with a custom map function.
#[derive(Clone)]
pub struct MappedEmitterValue {
    event: EmitterEvent,
    initial_event: EmitterEvent,
    map: fn(event: &EmitterEvent) -> EmitterEvent,
}

impl MappedEmitterValue {
    pub fn new(event: EmitterEvent, map: fn(event: &EmitterEvent) -> EmitterEvent) -> Self {
        let initial_event = event.clone();
        Self {
            event,
            initial_event,
            map,
        }
    }
}

impl Iterator for MappedEmitterValue {
    type Item = EmitterEvent;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.event.clone();
        self.event = (self.map)(&self.event);
        Some(current)
    }
}

impl EmitterValue for MappedEmitterValue {
    fn reset(&mut self) {
        self.event = self.initial_event.clone();
    }
}
