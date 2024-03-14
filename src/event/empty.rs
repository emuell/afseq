use std::{cell::RefCell, rc::Rc};

use crate::{
    event::{Event, EventIter},
    BeatTimeBase,
};

// -------------------------------------------------------------------------------------------------

/// Emits an empty, None [`Event`].
#[derive(Clone, Debug)]
pub struct EmptyEventIter {}

impl EventIter for EmptyEventIter {
    fn set_time_base(&mut self, _time_base: &BeatTimeBase) {
        // nothing to do
    }

    fn run(&mut self, _pulse: crate::PulseIterItem, _emit_event: bool) -> Option<Event> {
        None
    }

    fn duplicate(&self) -> Rc<RefCell<dyn EventIter>> {
        Rc::new(RefCell::new(self.clone()))
    }

    fn reset(&mut self) {
        // nothing to do
    }
}
