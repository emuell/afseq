use crate::{EmitterEvent, EmitterValue};
use std::fmt::Display;

// -------------------------------------------------------------------------------------------------

/// Emits a vector of const note values
#[derive(Clone)]
pub struct FixedEmitterValue {
    events: Vec<EmitterEvent>,
}

impl FixedEmitterValue {
    pub fn new(events: Vec<EmitterEvent>) -> Self {
        Self { events }
    }
}

impl Iterator for FixedEmitterValue {
    type Item = Vec<EmitterEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.events.clone())
    }
}

impl EmitterValue for FixedEmitterValue {}

impl Display for FixedEmitterValue {
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
