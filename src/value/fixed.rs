use crate::{EmitterEvent, EmitterValue};

// -------------------------------------------------------------------------------------------------

/// Emits a fixed set of immutable events.
#[derive(Clone)]
pub struct FixedEmitterValue {
    event: EmitterEvent,
}

impl FixedEmitterValue {
    pub fn new(events: EmitterEvent) -> Self {
        Self { event: events }
    }
}

impl Iterator for FixedEmitterValue {
    type Item = EmitterEvent;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.event.clone())
    }
}

impl EmitterValue for FixedEmitterValue {
    fn reset(&mut self) {
        // fixed values fon't change: nothing to do
    }
}
