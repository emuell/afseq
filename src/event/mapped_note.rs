use crate::event::{Event, EventIter, NoteEvent};

// -------------------------------------------------------------------------------------------------

/// Endlessly emits [`Event`] which's notes can be mutated/mapped in each iter step
/// with a custom map function.
#[derive(Clone)]
pub struct MappedNoteEventIter<F>
where
    F: FnMut(NoteEvent) -> NoteEvent,
{
    event: Event,
    initial_event: Event,
    map: F,
    initial_map: F,
}

impl<F> MappedNoteEventIter<F>
where
    F: FnMut(NoteEvent) -> NoteEvent + Copy,
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

    fn mutate(mut event: Event, map: &mut F) -> Event {
        match &event {
            Event::NoteEvents(notes) => {
                let mut mut_notes = notes.clone();
                for note in &mut mut_notes {
                    *note = (*map)(note.clone());
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
    F: FnMut(NoteEvent) -> NoteEvent + Copy,
{
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.event.clone();
        self.event = Self::mutate(self.event.clone(), &mut self.map);
        Some(current)
    }
}

impl<F> EventIter for MappedNoteEventIter<F>
where
    F: FnMut(NoteEvent) -> NoteEvent + Copy,
{
    fn reset(&mut self) {
        self.event = self.initial_event.clone();
        self.map = self.initial_map;
    }
}
