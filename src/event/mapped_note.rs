use crate::event::{fixed::FixedEventIter, Event, EventIter, NoteEvent};

// -------------------------------------------------------------------------------------------------

/// Pointer to a function which mutates a NoteEvent.
type NoteEventMapFn = dyn FnMut(NoteEvent, usize) -> NoteEvent + 'static;

// -------------------------------------------------------------------------------------------------

/// Endlessly emits [`Event`] which's notes can be mutated/mapped in each iter step
/// with a custom map function.
///
/// NB: This event iter is can not be cloned.
pub struct MappedNoteEventIter {
    events: Vec<Event>,
    initial_events: Vec<Event>,
    map: Box<NoteEventMapFn>,
    reset_map: Box<dyn Fn() -> Box<NoteEventMapFn>>,
    current: usize,
}

impl MappedNoteEventIter {
    pub fn new<F>(events: Vec<Event>, map: F) -> Self
    where
        F: FnMut(NoteEvent, usize) -> NoteEvent + 'static + Copy,
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
            map,
            reset_map: Box::new(move || Box::new(initial_map)),
            current,
        }
    }

    fn mutate(mut event: Event, map: &mut dyn FnMut(NoteEvent, usize) -> NoteEvent) -> Event {
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

impl Iterator for MappedNoteEventIter {
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

impl EventIter for MappedNoteEventIter {
    fn reset(&mut self) {
        self.events = self.initial_events.clone();
        self.map = (self.reset_map)();
        self.current = 0;
    }
}

// -------------------------------------------------------------------------------------------------

pub trait ToMappedNotesEventIter<F>
where
    F: FnMut(NoteEvent, usize) -> NoteEvent + Copy + 'static,
{
    fn map_notes(self, map: F) -> MappedNoteEventIter;
}

impl<F> ToMappedNotesEventIter<F> for FixedEventIter
where
    F: FnMut(NoteEvent, usize) -> NoteEvent + Copy + 'static,
{
    /// Upgrade a [`FixedEventIter`] to a [`MappedNoteEventIter`].
    fn map_notes(self, map: F) -> MappedNoteEventIter {
        MappedNoteEventIter::new(self.events(), map)
    }
}
