use crate::{BeatTimeBase, Event, InputParameterSet, Pattern, PulseIterItem};

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

    fn set_time_base(&mut self, _time_base: &BeatTimeBase) {
        // nothing to do
    }

    fn set_trigger_event(&mut self, _event: &Event) {
        // nothing to do
    }

    fn set_input_parameters(&mut self, _parameters: InputParameterSet) {
        // nothing to do
    }

    fn set_repeat_count(&mut self, _count: Option<usize>) {
        // nothing to do
    }

    fn run(&mut self) -> Option<PulseIterItem> {
        None
    }

    fn duplicate(&self) -> Box<dyn Pattern> {
        Box::new(self.clone())
    }

    fn reset(&mut self) {
        // nothing to do
    }
}
