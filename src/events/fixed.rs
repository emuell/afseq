use crate::events::{
    NoteEvent, PatternEvent, PatternEventIter,
    {mapped::MappedPatternEventIter, mapped_note::MappedNotePatternEventIter},
};

// -------------------------------------------------------------------------------------------------

/// Endlessly emits a single, fixed [`PatternEvent`].
#[derive(Clone)]
pub struct FixedPatternEventIter {
    event: PatternEvent,
}

impl FixedPatternEventIter {
    pub fn new(event: PatternEvent) -> Self {
        Self { event }
    }
}

impl Iterator for FixedPatternEventIter {
    type Item = PatternEvent;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.event.clone())
    }
}

impl PatternEventIter for FixedPatternEventIter {
    fn reset(&mut self) {
        // fixed values don't change, so there's nothing to reset
    }
}

// -------------------------------------------------------------------------------------------------

pub trait ToMappedEmitterValue<EventMap>
where
    EventMap: FnMut(PatternEvent) -> PatternEvent,
{
    fn map_event(self, map: EventMap) -> MappedPatternEventIter<EventMap>;
}

impl<EventMap> ToMappedEmitterValue<EventMap> for FixedPatternEventIter
where
    EventMap: FnMut(PatternEvent) -> PatternEvent,
{
    /// Upgrade a [`FixedPatternEventIter`] to a [`MappedPatternEventIter`].
    fn map_event(self, map: EventMap) -> MappedPatternEventIter<EventMap> {
        MappedPatternEventIter::new(self.event, map)
    }
}

// -------------------------------------------------------------------------------------------------

pub trait ToMappedNotesEmitterValue<NoteMap>
where
    NoteMap: FnMut(NoteEvent) -> NoteEvent,
{
    fn map_notes(self, map: NoteMap) -> MappedNotePatternEventIter<NoteMap>;
}

impl<NoteMap> ToMappedNotesEmitterValue<NoteMap> for FixedPatternEventIter
where
    NoteMap: FnMut(NoteEvent) -> NoteEvent,
{
    /// Upgrade a [`FixedPatternEventIter`] to a [`MappedNotePatternEventIter`].
    fn map_notes(self, map: NoteMap) -> MappedNotePatternEventIter<NoteMap> {
        MappedNotePatternEventIter::new(self.event, map)
    }
}
