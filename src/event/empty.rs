use crate::{
    event::{Event, EventIter},
    BeatTimeBase,
};

// -------------------------------------------------------------------------------------------------

/// Emits an empty, None [`Event`].
#[derive(Clone, Debug)]
pub struct EmptyEventIter {}

impl Iterator for EmptyEventIter {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

impl EventIter for EmptyEventIter {
    fn update_time_base(&mut self, _time_base: &BeatTimeBase) {
        // nothing to do
    }
    fn reset(&mut self) {
        // nothing to do
    }
}
