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
    events: Vec<(Fraction, Fraction, Vec<Option<Event>>)>,
}

impl CycleNoteEvents {
    /// Create a new, empty list of events.
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Add note events from a cycle channel event.
    pub fn add(
        &mut self,
        channel: usize,
        start: Fraction,
        length: Fraction,
        note_events: Vec<Option<NoteEvent>>,
    ) {
        match self
            .events
            .binary_search_by(|(time, _, _)| time.cmp(&start))
        {
            Ok(pos) => {
                // use min length of all notes in stack
                let event_length = &mut self.events[pos].1;
                *event_length = (*event_length).min(length);
                // add new notes to existing event stack
                let timed_event = &mut self.events[pos].2;
                timed_event.resize(channel + 1, None);
                timed_event[channel] = Some(Event::NoteEvents(note_events));
            }
            Err(pos) => self.events.insert(
                pos,
                (start, length, vec![Some(Event::NoteEvents(note_events))]),
            ),
        }
    }

    /// Convert to a list of EventIterItems.
    pub fn into_event_iter_items(self) -> Vec<EventIterItem> {
        let mut event_iter_items: Vec<EventIterItem> = Vec::with_capacity(self.events.len());
        for (start_time, length, events) in self.events.into_iter() {
            for event in events.into_iter().flatten() {
                event_iter_items.push(EventIterItem::new_with_fraction(event, start_time, length));
            }
        }
        event_iter_items
    }
}

// -------------------------------------------------------------------------------------------------

/// Emits a vector of [`EventIterItem`] from a Tidal [`Cycle`].
///
/// Channels from cycle are merged down into note events on different voices.
/// Values in cycles can be mapped to notes with an optional mapping table.
///
/// See also [`ScriptedCycleEventIter`](`super::scripted_cycle::ScriptedCycleEventIter`)
#[derive(Clone, Debug)]
pub struct CycleEventIter {
    cycle: Cycle,
    mappings: HashMap<String, Vec<Option<NoteEvent>>>,
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
        Ok(Self::new(Cycle::from(input)?))
    }

    /// Try creating a new cycle event iter from the given mini notation string
    /// and the given seed for the cycle's random number generator.
    ///
    /// Returns error when the cycle string failed to parse.
    pub fn from_mini_with_seed(input: &str, seed: [u8; 32]) -> Result<Self, String> {
        Ok(Self::new(Cycle::from(input)?.with_seed(seed)))
    }

    /// Return a new cycle with the given value mappings applied.
    pub fn with_mappings<S: Into<String> + Clone>(
        self,
        map: &[(S, Vec<Option<NoteEvent>>)],
    ) -> Self {
        let mut mappings = HashMap::new();
        for (k, v) in map.iter().cloned() {
            mappings.insert(k.into(), v);
        }
        Self { mappings, ..self }
    }

    /// Generate a note event from a single cycle event, applying mappings if necessary
    fn note_events(&mut self, event: CycleEvent) -> Result<Vec<Option<NoteEvent>>, String> {
        let mut note_events = {
            if let Some(note_events) = self.mappings.get(event.string()) {
                // apply custom note mappings
                note_events.clone()
            } else {
                // convert the cycle value to a single note
                vec![event.value().into()]
            }
        };
        // inject target instrument, if present
        if let Some(instrument) = event.target().into() {
            for mut note_event in &mut note_events {
                if let Some(note_event) = &mut note_event {
                    note_event.instrument = Some(instrument);
                }
            }
        }
        Ok(note_events)
    }

    /// Generate next batch of events from the next cycle run.
    /// Converts cycle events to note events and flattens channels into note columns.
    fn generate_events(&mut self) -> Vec<EventIterItem> {
        // run the cycle event generator
        let events = {
            match self.cycle.generate() {
                Ok(events) => events,
                Err(err) => {
                    // NB: only expected error here is exceeding the event limit
                    panic!("Cycle runtime error: {err}");
                }
            }
        };
        let mut timed_note_events = CycleNoteEvents::new();
        // convert possibly mapped cycle channel items to a list of note events
        for (channel_index, channel_events) in events.into_iter().enumerate() {
            for event in channel_events.into_iter() {
                let start = event.span().start();
                let length = event.span().length();
                match self.note_events(event) {
                    Ok(note_events) => {
                        if !note_events.is_empty() {
                            timed_note_events.add(channel_index, start, length, note_events);
                        }
                    }
                    Err(err) => {
                        //  NB: only expected error here is a chord parser error
                        panic!("Cycle runtime error: {err}");
                    }
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

pub fn new_cycle_event_with_seed(input: &str, seed: [u8; 32]) -> Result<CycleEventIter, String> {
    CycleEventIter::from_mini_with_seed(input, seed)
}
