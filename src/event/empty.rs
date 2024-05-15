use std::borrow::Cow;

use crate::{
    event::{Event, EventIter},
    BeatTimeBase, PulseIterItem,
};

// -------------------------------------------------------------------------------------------------

/// Emits an empty, None [`Event`].
#[derive(Clone, Debug)]
pub struct EmptyEventIter {}

impl EventIter for EmptyEventIter {
    fn set_time_base(&mut self, _time_base: &BeatTimeBase) {
        // nothing to do
    }

    fn set_external_context(&mut self, _data: &[(Cow<str>, f64)]) {
        // nothing to do
    }

    fn run(
        &mut self,
        _pulse: PulseIterItem,
        _pulse_pattern_length: usize,
        _emit_event: bool,
    ) -> Option<Vec<Event>> {
        None
    }

    fn duplicate(&self) -> Box<dyn EventIter> {
        Box::new(self.clone())
    }

    fn reset(&mut self) {
        // nothing to do
    }
}
