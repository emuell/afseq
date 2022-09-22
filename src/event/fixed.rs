use crate::event::{
    Event, EventIter, NoteEvent,
    {mapped::MappedEventIter, mapped_note::MappedNoteEventIter},
};

// -------------------------------------------------------------------------------------------------

/// Endlessly emits a single, fixed [`Event`].
#[derive(Clone)]
pub struct FixedEventIter {
    event: Event,
}

impl FixedEventIter {
    pub fn new(event: Event) -> Self {
        Self { event }
    }
}

impl Iterator for FixedEventIter {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.event.clone())
    }
}

impl EventIter for FixedEventIter {
    fn reset(&mut self) {
        // fixed values don't change, so there's nothing to reset
    }
}

// -------------------------------------------------------------------------------------------------

pub trait ToMappedEventIter<EventMap>
where
    EventMap: FnMut(Event) -> Event,
{
    fn map_event(self, map: EventMap) -> MappedEventIter<EventMap>;
}

impl<EventMap> ToMappedEventIter<EventMap> for FixedEventIter
where
    EventMap: FnMut(Event) -> Event,
{
    /// Upgrade a [`FixedEventIter`] to a [`MappedEventIter`].
    fn map_event(self, map: EventMap) -> MappedEventIter<EventMap> {
        MappedEventIter::new(self.event, map)
    }
}

// -------------------------------------------------------------------------------------------------

pub trait ToMappedNotesEventIter<NoteMap>
where
    NoteMap: FnMut(NoteEvent) -> NoteEvent,
{
    fn map_notes(self, map: NoteMap) -> MappedNoteEventIter<NoteMap>;
}

impl<NoteMap> ToMappedNotesEventIter<NoteMap> for FixedEventIter
where
    NoteMap: FnMut(NoteEvent) -> NoteEvent,
{
    /// Upgrade a [`FixedEventIter`] to a [`MappedNoteEventIter`].
    fn map_notes(self, map: NoteMap) -> MappedNoteEventIter<NoteMap> {
        MappedNoteEventIter::new(self.event, map)
    }
}
