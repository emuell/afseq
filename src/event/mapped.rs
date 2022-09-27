use crate::event::{fixed::FixedEventIter, Event, EventIter};

// -------------------------------------------------------------------------------------------------

/// Endlessly emits [`Event`] which's value can be mutated/mapped in each iter step
/// with a custom map function.
#[derive(Clone)]
pub struct MappedEventIter<F>
where
    F: FnMut(Event) -> Event + Copy,
{
    events: Vec<Event>,
    initial_events: Vec<Event>,
    map: F,
    initial_map: F,
    current: usize,
}

impl<F> MappedEventIter<F>
where
    F: FnMut(Event) -> Event + Copy,
{
    pub fn new(events: Vec<Event>, map: F) -> Self {
        let mut initial_map = map;
        let mut initial_events = events;
        if !initial_events.is_empty() {
            initial_events[0] = Self::mutate(initial_events[0].clone(), &mut initial_map);
        }
        let current = 0;
        Self {
            events: initial_events.clone(),
            initial_events,
            map: initial_map,
            initial_map,
            current,
        }
    }

    fn mutate(event: Event, map: &mut F) -> Event {
        (*map)(event)
    }
}

impl<F> Iterator for MappedEventIter<F>
where
    F: FnMut(Event) -> Event + Copy,
{
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

impl<F> EventIter for MappedEventIter<F>
where
    F: FnMut(Event) -> Event + Copy,
{
    fn reset(&mut self) {
        self.events = self.initial_events.clone();
        self.map = self.initial_map;
        self.current = 0;
    }
}

// -------------------------------------------------------------------------------------------------

pub trait ToMappedEventIter<EventMap>
where
    EventMap: FnMut(Event) -> Event + Copy,
{
    fn map_events(self, map: EventMap) -> MappedEventIter<EventMap>;
}

impl<EventMap> ToMappedEventIter<EventMap> for FixedEventIter
where
    EventMap: FnMut(Event) -> Event + Copy,
{
    /// Upgrade a [`FixedEventIter`] to a [`MappedEventIter`].
    fn map_events(self, map: EventMap) -> MappedEventIter<EventMap> {
        MappedEventIter::new(self.events(), map)
    }
}
