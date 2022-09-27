use crate::event::{fixed::FixedEventIter, Event, EventIter, NoteEvent};

// -------------------------------------------------------------------------------------------------

/// Endlessly emits [`Event`] which's notes can be mutated/mapped in each iter step
/// with a custom map function.
#[derive(Clone)]
pub struct MappedNoteEventIter<F>
where
    F: FnMut(NoteEvent, usize) -> NoteEvent,
{
    events: Vec<Event>,
    initial_events: Vec<Event>,
    map: F,
    initial_map: F,
    current: usize,
}

impl<F> MappedNoteEventIter<F>
where
    F: FnMut(NoteEvent, usize) -> NoteEvent + Copy,
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

    fn mutate(mut event: Event, map: &mut F) -> Event {
        match &event {
            Event::NoteEvents(notes) => {
                let mut mut_notes = notes.clone();
                for (index, note) in &mut mut_notes.iter_mut().enumerate() {
                    *note = (*map)(note.clone(), index);
                }
                event = Event::NoteEvents(mut_notes);
            }
            Event::ParameterChangeEvent(_) => {}
        }
        event
    }
}

impl<F> Iterator for MappedNoteEventIter<F>
where
    F: FnMut(NoteEvent, usize) -> NoteEvent + Copy,
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

impl<F> EventIter for MappedNoteEventIter<F>
where
    F: FnMut(NoteEvent, usize) -> NoteEvent + Copy,
{
    fn reset(&mut self) {
        self.events = self.initial_events.clone();
        self.map = self.initial_map;
        self.current = 0;
    }
}

// -------------------------------------------------------------------------------------------------

pub trait ToMappedNotesEventIter<NoteMap>
where
    NoteMap: FnMut(NoteEvent, usize) -> NoteEvent,
{
    fn map_notes(self, map: NoteMap) -> MappedNoteEventIter<NoteMap>;
}

impl<NoteMap> ToMappedNotesEventIter<NoteMap> for FixedEventIter
where
    NoteMap: FnMut(NoteEvent, usize) -> NoteEvent + Copy,
{
    /// Upgrade a [`FixedEventIter`] to a [`MappedNoteEventIter`].
    fn map_notes(self, map: NoteMap) -> MappedNoteEventIter<NoteMap> {
        MappedNoteEventIter::new(self.events(), map)
    }
}
