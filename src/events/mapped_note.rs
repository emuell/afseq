use crate::events::{NoteEvent, PatternEvent, PatternEventIter};

// -------------------------------------------------------------------------------------------------

/// Endlessly emits [`PatternEvent`] which's notes can be mutated/mapped in each iter step
/// with a custom map function.
#[derive(Clone)]
pub struct MappedNotePatternEventIter<F>
where
    F: FnMut(NoteEvent) -> NoteEvent,
{
    event: PatternEvent,
    initial_event: PatternEvent,
    map: F,
}

impl<F> MappedNotePatternEventIter<F>
where
    F: FnMut(NoteEvent) -> NoteEvent,
{
    pub fn new(event: PatternEvent, map: F) -> Self {
        let initial_event = event.clone();
        Self {
            event,
            initial_event,
            map,
        }
    }
}

impl<F> Iterator for MappedNotePatternEventIter<F>
where
    F: FnMut(NoteEvent) -> NoteEvent,
{
    type Item = PatternEvent;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.event.clone();
        match &self.event {
            PatternEvent::NoteEvents(notes) => {
                let mut mut_notes = notes.clone();
                for note in &mut mut_notes {
                    *note = (self.map)(note.clone());
                }
                self.event = PatternEvent::NoteEvents(mut_notes);
            }
            PatternEvent::ParameterChangeEvent(_) => {}
        }
        Some(current)
    }
}

impl<F> PatternEventIter for MappedNotePatternEventIter<F>
where
    F: FnMut(NoteEvent) -> NoteEvent,
{
    fn reset(&mut self) {
        self.event = self.initial_event.clone();
    }
}
