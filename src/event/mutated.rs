use std::{cell::RefCell, fmt::Debug, rc::Rc};

use crate::{
    event::{fixed::FixedEventIter, Event, EventIter},
    BeatTimeBase,
};

// -------------------------------------------------------------------------------------------------

/// Pointer to a function which mutates an Event.
type EventMapFn = dyn FnMut(Event) -> Event + 'static;

// -------------------------------------------------------------------------------------------------

/// Endlessly emits [`Event`] which's value can be mutated in each iter step
/// with a custom closure.
///
/// NB: This event iter can not be cloned. `clone_dyn` thus will cause a panic!
pub struct MutatedEventIter {
    events: Vec<Event>,
    initial_events: Vec<Event>,
    map: Box<EventMapFn>,
    reset_map: Box<dyn Fn() -> Box<EventMapFn>>,
    current: usize,
}

impl MutatedEventIter {
    pub fn new<F>(events: Vec<Event>, map: F) -> Self
    where
        F: FnMut(Event) -> Event + Clone + 'static,
    {
        // capture initial map state
        let initial_map = map.clone();
        // apply first mutation and memorize initial set of events
        let mut map = Box::new(map);
        let mut initial_events = events;
        if !initial_events.is_empty() {
            initial_events[0] = Self::mutate(initial_events[0].clone(), &mut map);
        }
        let events = initial_events.clone();
        let current = 0;
        Self {
            events,
            initial_events,
            reset_map: Box::new(move || Box::new(initial_map.clone())),
            map,
            current,
        }
    }

    fn mutate(event: Event, map: &mut dyn FnMut(Event) -> Event) -> Event {
        (*map)(event)
    }
}

impl Debug for MutatedEventIter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("MutatedEventIter")
            .field("events", &self.events)
            .field("initial_events", &self.initial_events)
            .field("current", &self.current)
            .finish()
    }
}

impl Iterator for MutatedEventIter {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.events[self.current].clone();
        self.events[self.current] = Self::mutate(current.clone(), &mut self.map);
        self.current += 1;
        if self.current >= self.events.len() {
            self.current = 0;
        }
        Some(current)
    }
}

impl EventIter for MutatedEventIter {
    fn set_time_base(&mut self, _time_base: &BeatTimeBase) {
        // nothing to do
    }

    fn duplicate(&self) -> Rc<RefCell<dyn EventIter>> {
        panic!("Mutated event iters can't be cloned")
    }

    fn reset(&mut self) {
        self.events = self.initial_events.clone();
        self.map = (self.reset_map)();
        self.current = 0;
    }
}

// -------------------------------------------------------------------------------------------------

pub trait ToMutatedEventIter<F>
where
    F: FnMut(Event) -> Event + Clone + 'static,
{
    fn mutate(self, map: F) -> MutatedEventIter;
}

impl<F> ToMutatedEventIter<F> for FixedEventIter
where
    F: FnMut(Event) -> Event + Clone + 'static,
{
    /// Upgrade a [`FixedEventIter`] to a [`MutatedEventIter`].
    fn mutate(self, map: F) -> MutatedEventIter {
        MutatedEventIter::new(self.events(), map)
    }
}
