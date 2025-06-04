use std::fmt::Debug;

use crate::{
    event::fixed::FixedEventIter, BeatTimeBase, Event, EventIter, EventIterItem, InputParameterSet,
    PulseIterItem,
};

// -------------------------------------------------------------------------------------------------

/// Pointer to a function which mutates an Event.
type EventMapFn = dyn FnMut(&mut Event) + 'static;

// -------------------------------------------------------------------------------------------------

/// Continuously emits [`EventIterItem`] which's value can be mutated in each iter step
/// with a custom closure.
///
/// NB: This event iter can not be cloned. `clone_dyn` thus will cause a panic!
pub struct MutatedEventIter {
    events: Vec<Event>,
    event_index: usize,
    initial_events: Vec<Event>,
    map: Box<EventMapFn>,
    reset_map: Box<dyn Fn() -> Box<EventMapFn>>,
}

impl MutatedEventIter {
    pub fn new<F>(events: Vec<Event>, map: F) -> Self
    where
        F: FnMut(&mut Event) + Clone + 'static,
    {
        // capture initial map state
        let initial_map = map.clone();
        // apply first mutation and memorize initial set of events
        let mut map = Box::new(map);
        let mut initial_events = events;
        if !initial_events.is_empty() {
            map(&mut initial_events[0]);
        }
        let reset_map: Box<dyn Fn() -> Box<EventMapFn>> =
            Box::new(move || Box::new(initial_map.clone()));
        let events = initial_events.clone();
        let event_index = 0;
        Self {
            events,
            event_index,
            initial_events,
            reset_map,
            map,
        }
    }
}

impl Debug for MutatedEventIter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("MutatedEventIter")
            .field("events", &self.events)
            .field("event_index", &self.event_index)
            .field("initial_events", &self.initial_events)
            .finish_non_exhaustive()
    }
}

impl EventIter for MutatedEventIter {
    fn set_time_base(&mut self, _time_base: &BeatTimeBase) {
        // nothing to do
    }

    fn set_trigger_event(&mut self, _event: &Event) {
        // nothing to do
    }

    fn set_input_parameters(&mut self, _parameters: InputParameterSet) {
        // nothing to do
    }

    fn run(&mut self, _pulse: PulseIterItem, emit_event: bool) -> Option<Vec<EventIterItem>> {
        if !emit_event || self.events.is_empty() {
            return None;
        }
        let mut event = self.events[self.event_index].clone();
        (*self.map)(&mut event);
        self.event_index = (self.event_index + 1) % self.events.len();
        Some(vec![EventIterItem::new(event)])
    }

    fn duplicate(&self) -> Box<dyn EventIter> {
        panic!("Mutated event iters can't be cloned")
    }

    fn reset(&mut self) {
        self.events.clone_from(&self.initial_events);
        self.event_index = 0;
        self.map = (self.reset_map)();
    }
}

// -------------------------------------------------------------------------------------------------

pub trait ToMutatedEventIter<F>
where
    F: FnMut(&mut Event) + Clone + 'static,
{
    fn mutate(self, map: F) -> MutatedEventIter;
}

impl<F> ToMutatedEventIter<F> for FixedEventIter
where
    F: FnMut(&mut Event) + Clone + 'static,
{
    /// Upgrade a [`FixedEventIter`] to a [`MutatedEventIter`].
    fn mutate(self, map: F) -> MutatedEventIter {
        MutatedEventIter::new(self.events().clone(), map)
    }
}
