use std::fmt::Display;

use crate::{EmitterEvent, EmitterValue};

// -------------------------------------------------------------------------------------------------

/// Emits a emitter value which can be mutated with a custom function
#[derive(Clone)]
pub struct MappedEmitterValue {
    events: Vec<EmitterEvent>,
    map_fn: fn(events: &Vec<EmitterEvent>) -> Vec<EmitterEvent>,
}

impl MappedEmitterValue {
    pub fn new(
        events: Vec<EmitterEvent>,
        map_fn: fn(event: &Vec<EmitterEvent>) -> Vec<EmitterEvent>,
    ) -> Self {
        Self { events, map_fn }
    }
}

impl Iterator for MappedEmitterValue {
    type Item = Vec<EmitterEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        let copy = self.events.clone();
        self.events = (self.map_fn)(&self.events);
        Some(copy)
    }
}

impl EmitterValue for MappedEmitterValue {}

impl Display for MappedEmitterValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}",
            self.events
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join(","),
        ))
    }
}
