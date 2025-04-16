use crate::{BeatTimeBase, Event, EventIter, EventIterItem, InputParameterSet, PulseIterItem};

// -------------------------------------------------------------------------------------------------

/// Continuously emits empty [`EventIterItem`]S.
#[derive(Clone, Debug)]
pub struct EmptyEventIter {}

impl EventIter for EmptyEventIter {
    fn set_time_base(&mut self, _time_base: &BeatTimeBase) {
        // nothing to do
    }

    fn set_trigger_event(&mut self, _event: &Event) {
        // nothing to do
    }

    fn set_input_parameters(&mut self, _parameters: InputParameterSet) {
        // nothing to do
    }

    fn run(&mut self, _pulse: PulseIterItem, _emit_event: bool) -> Option<Vec<EventIterItem>> {
        None
    }

    fn duplicate(&self) -> Box<dyn EventIter> {
        Box::new(self.clone())
    }

    fn reset(&mut self) {
        // nothing to do
    }
}
