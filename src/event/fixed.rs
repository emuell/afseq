use crate::{
    event::new_note, BeatTimeBase, Event, EventIter, EventIterItem, InputParameterSet, Note,
    NoteEvent, ParameterChangeEvent, PulseIterItem,
};

// -------------------------------------------------------------------------------------------------

/// Continuously emits a single, fixed [`EventIterItem`].
#[derive(Clone, Debug)]
pub struct FixedEventIter {
    events: Vec<Event>,
    event_index: usize,
}

impl FixedEventIter {
    pub fn new(events: Vec<Event>) -> Self {
        let mut events = events;
        Self::normalize_events(&mut events);
        let event_index = 0;
        Self {
            events,
            event_index,
        }
    }

    /// Access to the event that we're triggering
    pub fn events(&self) -> &Vec<Event> {
        &self.events
    }

    /// Add note-offs for all notes in the given event list
    pub(crate) fn normalize_events(events: &mut Vec<Event>) {
        let mut note_event_state = Vec::<Option<NoteEvent>>::new();
        for event in &mut *events {
            Self::normalize_event(event, &mut note_event_state);
        }
        if events.len() > 1 {
            Self::normalize_event(events.first_mut().unwrap(), &mut note_event_state)
        }
    }

    /// Add note-offs for all notes in the given event to stop pending notes from last runs.
    /// This will only pad, resize the given event's note list with note-offs where necessary.
    pub(crate) fn normalize_event(
        event: &mut Event,
        note_event_state: &mut Vec<Option<NoteEvent>>,
    ) {
        if let Event::NoteEvents(note_events) = event {
            if !note_events.iter().all(|n| n.is_none()) {
                // auto-close previous note's note-ons, unless there is some explicit content
                while note_events.len() < note_event_state.len() {
                    if note_event_state[note_events.len()]
                        .as_ref()
                        .is_some_and(|n| n.note.is_note_on())
                    {
                        note_events.push(new_note(Note::OFF));
                    } else {
                        note_events.push(None);
                    }
                }
                // update note state
                if note_event_state.len() < note_events.len() {
                    note_event_state.resize(note_events.len(), None);
                }
                for (note_event_state, note_event) in
                    note_event_state.iter_mut().zip(note_events.iter())
                {
                    if let Some(note_event) = note_event {
                        if note_event.note.is_note_on() {
                            *note_event_state = Some(note_event.clone());
                        } else {
                            *note_event_state = None;
                        }
                    }
                }
            }
            // remove trailing none's from note state
            while note_event_state.last().is_some_and(|n| n.is_none()) {
                note_event_state.pop();
            }
        }
    }
}

impl Default for FixedEventIter {
    fn default() -> Self {
        Self::new(vec![Event::NoteEvents(vec![Some((Note::C4).into())])])
    }
}
impl EventIter for FixedEventIter {
    fn set_time_base(&mut self, _time_base: &BeatTimeBase) {
        // nothing to do
    }

    fn set_trigger_event(&mut self, _event: &Event) {
        // nothing to do
    }

    fn set_input_parameters(&mut self, _parameters: InputParameterSet) {
        // nothing to do
    }

    fn run(&mut self, _pulse: PulseIterItem, emit_event: bool) -> Option<Vec<EventIterItem>> {
        if !emit_event || self.events.is_empty() {
            return None;
        }
        let event = self.events[self.event_index].clone();
        self.event_index = (self.event_index + 1) % self.events.len();
        Some(vec![EventIterItem::new(event)])
    }

    fn advance(&mut self, _pulse: PulseIterItem, emit_event: bool) {
        if !emit_event || self.events.is_empty() {
            return;
        }
        self.event_index = (self.event_index + 1) % self.events.len();
    }

    fn duplicate(&self) -> Box<dyn EventIter> {
        Box::new(self.clone())
    }

    fn reset(&mut self) {
        // reset step counter
        self.event_index = 0;
    }
}

// -------------------------------------------------------------------------------------------------

pub trait ToFixedEventIter {
    fn to_event(self) -> FixedEventIter;
}

impl ToFixedEventIter for NoteEvent {
    /// Wrap a [`NoteEvent`] to a new [`FixedEventIter`]
    /// resulting into a single monophonic event.
    fn to_event(self) -> FixedEventIter {
        FixedEventIter::new(vec![Event::NoteEvents(vec![Some(self)])])
    }
}
impl ToFixedEventIter for Option<NoteEvent> {
    /// Wrap a [`NoteEvent`] to a new [`FixedEventIter`]
    /// resulting into a single monophonic event.
    fn to_event(self) -> FixedEventIter {
        FixedEventIter::new(vec![Event::NoteEvents(vec![self])])
    }
}

impl ToFixedEventIter for Vec<NoteEvent> {
    /// Wrap a vector of [`NoteEvent`] to a new [`FixedEventIter`].
    /// resulting into a single polyphonic event.
    fn to_event(self) -> FixedEventIter {
        FixedEventIter::new(vec![Event::NoteEvents(
            self.iter().map(|v| Some(v.clone())).collect::<Vec<_>>(),
        )])
    }
}
impl ToFixedEventIter for Vec<Option<NoteEvent>> {
    /// Wrap a vector of [`NoteEvent`] to a new [`FixedEventIter`].
    /// resulting into a single polyphonic event.
    fn to_event(self) -> FixedEventIter {
        FixedEventIter::new(vec![Event::NoteEvents(self)])
    }
}

impl ToFixedEventIter for ParameterChangeEvent {
    /// Wrap a [`ParameterChangeEvent`] into a new [`FixedEventIter`].
    fn to_event(self) -> FixedEventIter {
        FixedEventIter::new(vec![Event::ParameterChangeEvent(self)])
    }
}

// -------------------------------------------------------------------------------------------------

pub trait ToFixedEventIterSequence {
    fn to_event_sequence(self) -> FixedEventIter;
}

impl ToFixedEventIterSequence for Vec<Option<NoteEvent>> {
    /// Wrap a vector of [`NoteEvent`] to a new [`FixedEventIter`]
    /// resulting into a sequence of single note events.
    fn to_event_sequence(self) -> FixedEventIter {
        let mut sequence = Vec::with_capacity(self.len());
        for note in self {
            sequence.push(Event::NoteEvents(vec![note]));
        }
        FixedEventIter::new(sequence)
    }
}

impl ToFixedEventIterSequence for Vec<Vec<Option<NoteEvent>>> {
    /// Wrap a vector of vectors of [`NoteEvent`] to a new [`FixedEventIter`]
    /// resulting into a sequence of polyphonic note events.
    fn to_event_sequence(self) -> FixedEventIter {
        let mut sequence = Vec::with_capacity(self.len());
        for notes in self {
            sequence.push(Event::NoteEvents(notes));
        }
        FixedEventIter::new(sequence)
    }
}

impl ToFixedEventIterSequence for Vec<ParameterChangeEvent> {
    /// Wrap a [`ParameterChangeEvent`] into a new [`FixedEventIter`]
    fn to_event_sequence(self) -> FixedEventIter {
        let mut sequence = Vec::with_capacity(self.len());
        for p in self {
            sequence.push(Event::ParameterChangeEvent(p));
        }
        FixedEventIter::new(sequence)
    }
}
