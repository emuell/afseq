use std::fmt::Debug;

use crate::{
    emitter::fixed::FixedEmitter, BeatTimeBase, Emitter, EmitterEvent, Event, ParameterSet,
    RhythmEvent,
};

// -------------------------------------------------------------------------------------------------

/// Pointer to a function which mutates an Event.
type EventMapFn = dyn FnMut(&mut Event) + 'static;

// -------------------------------------------------------------------------------------------------

/// Continuously emits a single, static emitter event value, whose value can be mutated in each
/// iter step with a custom closure.
///
/// NB: This emitter can not be cloned. `duplicate` will panic!
pub struct MutatedEmitter {
    events: Vec<Event>,
    event_index: usize,
    initial_events: Vec<Event>,
    map: Box<EventMapFn>,
    reset_map: Box<dyn Fn() -> Box<EventMapFn>>,
}

impl MutatedEmitter {
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

impl Debug for MutatedEmitter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("MutatedEmitter")
            .field("events", &self.events)
            .field("event_index", &self.event_index)
            .field("initial_events", &self.initial_events)
            .finish_non_exhaustive()
    }
}

impl Emitter for MutatedEmitter {
    fn set_time_base(&mut self, _time_base: &BeatTimeBase) {
        // nothing to do
    }

    fn set_trigger_event(&mut self, _event: &Event) {
        // nothing to do
    }

    fn set_parameters(&mut self, _parameters: ParameterSet) {
        // nothing to do
    }

    fn run(&mut self, _pulse: RhythmEvent, emit_event: bool) -> Option<Vec<EmitterEvent>> {
        if !emit_event || self.events.is_empty() {
            return None;
        }
        let mut event = self.events[self.event_index].clone();
        (*self.map)(&mut event);
        self.event_index = (self.event_index + 1) % self.events.len();
        Some(vec![EmitterEvent::new(event)])
    }

    fn duplicate(&self) -> Box<dyn Emitter> {
        panic!("Mutated event emitters can't be cloned")
    }

    fn reset(&mut self) {
        self.events.clone_from(&self.initial_events);
        self.event_index = 0;
        self.map = (self.reset_map)();
    }
}

// -------------------------------------------------------------------------------------------------

pub trait ToMutatedEmitter<F>
where
    F: FnMut(&mut Event) + Clone + 'static,
{
    fn mutate(self, map: F) -> MutatedEmitter;
}

impl<F> ToMutatedEmitter<F> for FixedEmitter
where
    F: FnMut(&mut Event) + Clone + 'static,
{
    /// Upgrade a [`FixedEmitter`] to a [`MutatedEmitter`].
    fn mutate(self, map: F) -> MutatedEmitter {
        MutatedEmitter::new(self.events().to_vec(), map)
    }
}
