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
}

impl<F> MappedNoteEventIter<F>
where
    F: FnMut(NoteEvent) -> NoteEvent,
{
    pub fn new(event: Event, map: F) -> Self {
        let initial_event = event.clone();
        Self {
            event,
            initial_event,
            map,
        }
    }
}

impl<F> Iterator for MappedNoteEventIter<F>
where
    F: FnMut(NoteEvent) -> NoteEvent,
{
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.event.clone();
        match &self.event {
            Event::NoteEvents(notes) => {
                let mut mut_notes = notes.clone();
                for note in &mut mut_notes {
                    *note = (self.map)(note.clone());
                }
                self.event = Event::NoteEvents(mut_notes);
            }
            Event::ParameterChangeEvent(_) => {}
        }
        Some(current)
    }
}

impl<F> EventIter for MappedNoteEventIter<F>
where
    F: FnMut(NoteEvent) -> NoteEvent,
{
    fn reset(&mut self) {
        self.event = self.initial_event.clone();
    }
}
