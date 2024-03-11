use std::{cell::RefCell, rc::Rc};

use crate::{BeatTimeBase, Pattern, PulseIterItem};

// -------------------------------------------------------------------------------------------------

/// Defines an empty pattern which never triggers a pulse.
#[derive(Clone, Debug, Default)]
pub struct EmptyPattern {}

impl EmptyPattern {
    pub fn new() -> Self {
        Self {}
    }
}

impl Pattern for EmptyPattern {
    fn is_empty(&self) -> bool {
        true
    }

    fn len(&self) -> usize {
        0
    }

    fn peek(&self) -> PulseIterItem {
        PulseIterItem {
            value: 0.0,
            step_time: 1.0,
        }
    }

    fn run(&mut self) -> PulseIterItem {
        panic!("Empty patterns should not be run");
    }

    fn set_time_base(&mut self, _time_base: &BeatTimeBase) {
        // nothing to do
    }

    fn duplicate(&self) -> Rc<RefCell<dyn Pattern>> {
        Rc::new(RefCell::new(self.clone()))
    }

    fn reset(&mut self) {
        // nothing to do
    }
}
