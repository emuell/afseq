use crate::event::{fixed::FixedEventIter, Event, EventIter};

// -------------------------------------------------------------------------------------------------

/// Endlessly emits [`Event`] which's value can be mutated/mapped in each iter step
/// with a custom map function.
#[derive(Clone)]
pub struct MappedEventIter<F>
where
    F: FnMut(Event) -> Event + Copy,
{
    event: Event,
    initial_event: Event,
    map: F,
    initial_map: F,
}

impl<F> MappedEventIter<F>
where
    F: FnMut(Event) -> Event + Copy,
{
    pub fn new(event: Event, map: F) -> Self {
        let mut initial_map = map;
        let initial_event = Self::mutate(event, &mut initial_map);
        Self {
            event: initial_event.clone(),
            initial_event,
            map: initial_map,
            initial_map,
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
        let current = self.event.clone();
        self.event = Self::mutate(self.event.clone(), &mut self.map);
        Some(current)
    }
}

impl<F> EventIter for MappedEventIter<F>
where
    F: FnMut(Event) -> Event + Copy,
{
    fn reset(&mut self) {
        self.event = self.initial_event.clone();
        self.map = self.initial_map;
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
        MappedEventIter::new(self.event, map)
    }
}
