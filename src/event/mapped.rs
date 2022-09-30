use crate::event::{fixed::FixedEventIter, Event, EventIter};

// -------------------------------------------------------------------------------------------------

/// Pointer to a function which mutates an Event.
type EventMapFn = dyn FnMut(Event) -> Event + 'static;

// -------------------------------------------------------------------------------------------------

/// Endlessly emits [`Event`] which's value can be mutated/mapped in each iter step
/// with a custom map function.
///
/// NB: This event iter is can not be cloned.
pub struct MappedEventIter {
    events: Vec<Event>,
    initial_events: Vec<Event>,
    map: Box<EventMapFn>,
    reset_map: Box<dyn Fn() -> Box<EventMapFn>>,
    current: usize,
}

impl MappedEventIter {
    pub fn new<F>(events: Vec<Event>, map: F) -> Self
    where
        F: FnMut(Event) -> Event + Copy + 'static,
    {
        // capture initial map state
        let initial_map = map;
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
            reset_map: Box::new(move || Box::new(initial_map)),
            map,
            current,
        }
    }

    fn mutate(event: Event, map: &mut dyn FnMut(Event) -> Event) -> Event {
        (*map)(event)
    }
}

impl Iterator for MappedEventIter {
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

impl EventIter for MappedEventIter {
    fn reset(&mut self) {
        self.events = self.initial_events.clone();
        self.map = (self.reset_map)();
        self.current = 0;
    }
}

// -------------------------------------------------------------------------------------------------

pub trait ToMappedEventIter<F>
where
    F: FnMut(Event) -> Event + Copy + 'static,
{
    fn map_events(self, map: F) -> MappedEventIter;
}

impl<F> ToMappedEventIter<F> for FixedEventIter
where
    F: FnMut(Event) -> Event + Copy + 'static,
{
    /// Upgrade a [`FixedEventIter`] to a [`MappedEventIter`].
    fn map_events(self, map: F) -> MappedEventIter {
        MappedEventIter::new(self.events(), map)
    }
}
