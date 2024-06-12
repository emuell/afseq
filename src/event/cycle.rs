use std::{borrow::Cow, collections::HashMap};

use fraction::Fraction;

use crate::{
    event::{new_note, Event, EventIter, EventIterItem, InstrumentId, NoteEvent},
    tidal::{Cycle, Event as CycleEvent, Target as CycleTarget, Value as CycleValue},
    BeatTimeBase, Note, PulseIterItem,
};

// -------------------------------------------------------------------------------------------------

/// Default conversion of a cycle event value to an optional NoteEvent, as used by [`EventIter`].
impl From<&CycleValue> for Option<NoteEvent> {
    fn from(value: &CycleValue) -> Self {
        match value {
            CycleValue::Hold => None,
            CycleValue::Rest => new_note(Note::OFF),
            CycleValue::Float(_f) => None,
            CycleValue::Integer(i) => new_note(Note::from((*i).clamp(0, 0x7f) as u8)),
            CycleValue::Pitch(p) => new_note(Note::from(p.midi_note())),
            CycleValue::Name(s) => {
                if s.eq_ignore_ascii_case("off") {
                    new_note(Note::OFF)
                } else {
                    None
                }
            }
        }
    }
}

/// Default conversion of a cycle target to an optional instrument id, as used by [`EventIter`].
impl From<&CycleTarget> for Option<InstrumentId> {
    fn from(value: &CycleTarget) -> Self {
        match value {
            CycleTarget::None => None,
            CycleTarget::Index(i) => Some(InstrumentId::from(*i as usize)),
            CycleTarget::Name(_) => None, // unsupported
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Helper struct to convert time tagged events from Cycle into a `Vec<EventIterItem>`
pub(crate) struct CycleNoteEvents {
    events: Vec<(Fraction, Fraction, Vec<Option<NoteEvent>>)>,
}

impl CycleNoteEvents {
    /// Create a new, empty list of events.
    pub fn new() -> Self {
        Self { events: vec![] }
    }

    /// Add a new cycle channel item.
    pub fn add(
        &mut self,
        channel: usize,
        start: Fraction,
        length: Fraction,
        note_event: NoteEvent,
    ) {
        match self
            .events
            .binary_search_by(|(time, _, _)| time.cmp(&start))
        {
            Ok(pos) => {
                // use max length of all notes in stack
                let note_length = &mut self.events[pos].1;
                *note_length = (*note_length).max(length);
                // add note to existing time stack
                let note_events = &mut self.events[pos].2;
                note_events.resize(channel + 1, None);
                note_events[channel] = Some(note_event);
            }
            Err(pos) => self
                .events
                .insert(pos, (start, length, vec![Some(note_event)])),
        }
    }

    /// Convert to a list of NoteEvents.
    pub fn into_event_iter_items(self) -> Vec<EventIterItem> {
        let mut events: Vec<EventIterItem> = Vec::with_capacity(self.events.len());
        for (start_time, length, note_events) in self.events.into_iter() {
            events.push(EventIterItem::new_with_fraction(
                Event::NoteEvents(note_events),
                start_time,
                length,
            ));
        }
        events
    }
}

// -------------------------------------------------------------------------------------------------

/// Emits a vector of [`Event`]S from a Tidal [`Cycle`].
///
/// Channels from cycle are merged down into note events on different voices.
/// Float and String targets are currently unsupported and will result into None events.
#[derive(Clone, Debug)]
pub struct CycleEventIter {
    cycle: Cycle,
    mappings: HashMap<String, Option<NoteEvent>>,
}

impl CycleEventIter {
    /// Create a new cycle event iter from the given precompiled cycle.
    pub(crate) fn new(cycle: Cycle) -> Self {
        let mappings = HashMap::new();
        Self { cycle, mappings }
    }

    /// Try creating a new cycle event iter from the given mini notation string.
    ///
    /// Returns error when the cycle string failed to parse.
    pub fn from_mini(input: &str) -> Result<Self, String> {
        Ok(Self::new(Cycle::from(input, None)?))
    }

    /// Try creating a new cycle event iter from the given mini notation string
    /// and the given seed for the cycle's random number generator.
    ///
    /// Returns error when the cycle string failed to parse.
    pub fn from_mini_with_seed(input: &str, seed: Option<[u8; 32]>) -> Result<Self, String> {
        Ok(Self::new(Cycle::from(input, seed)?))
    }

    /// Return a new cycle with the given value mappings applied.
    pub fn with_mappings<S: Into<String> + Clone>(self, map: &[(S, Option<NoteEvent>)]) -> Self {
        let mut mappings = HashMap::new();
        for (k, v) in map.iter().cloned() {
            mappings.insert(k.into(), v);
        }
        Self { mappings, ..self }
    }

    /// Generate a note event from a single cycle event, applying mappings if necessary
    fn note_event(&mut self, event: CycleEvent) -> Option<NoteEvent> {
        let mut note_event = {
            if let Some(mapped_note_event) = self.mappings.get(event.string()) {
                // apply custom note mapping
                mapped_note_event.clone()
            } else {
                // else try to convert value to a note
                event.value().into()
            }
        };
        // inject target instrument, if present
        if let Some(instrument) = event.target().into() {
            if let Some(note_event) = &mut note_event {
                note_event.instrument = Some(instrument);
            }
        }
        note_event
    }

    /// Generate next batch of events from the next cycle run.
    /// Converts cycle events to note events and flattens channels into note columns.
    fn generate_events(&mut self) -> Vec<EventIterItem> {
        let mut timed_note_events = CycleNoteEvents::new();
        // convert possibly mapped cycle channel items to a list of note events
        for (channel_index, channel_events) in self.cycle.generate().into_iter().enumerate() {
            for event in channel_events.into_iter() {
                let start = event.span().start();
                let length = event.span().length();
                if let Some(note_event) = self.note_event(event) {
                    timed_note_events.add(channel_index, start, length, note_event);
                }
            }
        }
        // convert timed note events into EventIterItems
        timed_note_events.into_event_iter_items()
    }
}

impl EventIter for CycleEventIter {
    fn set_time_base(&mut self, _time_base: &BeatTimeBase) {
        // nothing to do
    }

    fn set_external_context(&mut self, _data: &[(Cow<str>, f64)]) {
        // nothing to do
    }

    fn run(&mut self, _pulse: PulseIterItem, emit_event: bool) -> Option<Vec<EventIterItem>> {
        if emit_event {
            Some(self.generate_events())
        } else {
            None
        }
    }

    fn duplicate(&self) -> Box<dyn EventIter> {
        Box::new(self.clone())
    }

    fn reset(&mut self) {
        self.cycle.reset();
    }
}

// -------------------------------------------------------------------------------------------------

pub fn new_cycle_event(input: &str) -> Result<CycleEventIter, String> {
    CycleEventIter::from_mini(input)
}

pub fn new_cycle_event_with_seed(
    input: &str,
    seed: Option<[u8; 32]>,
) -> Result<CycleEventIter, String> {
    CycleEventIter::from_mini_with_seed(input, seed)
}
