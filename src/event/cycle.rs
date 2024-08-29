use std::{borrow::Cow, collections::HashMap};

use fraction::Fraction;

use crate::{
    event::new_note, BeatTimeBase, Chord, Cycle, CycleEvent, CycleTarget, CycleValue, Event,
    EventIter, EventIterItem, InputParameterMap, InstrumentId, Note, NoteEvent, PulseIterItem,
};

// -------------------------------------------------------------------------------------------------

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

/// Default conversion of a CycleValue into a note stack.
///
/// Returns an error when resolving chord modes failed.
impl TryFrom<&CycleValue> for Vec<Option<NoteEvent>> {
    type Error = String;

    fn try_from(value: &CycleValue) -> Result<Self, String> {
        match value {
            CycleValue::Hold => Ok(vec![None]),
            CycleValue::Rest => Ok(vec![new_note(Note::OFF)]),
            CycleValue::Float(_f) => Ok(vec![None]),
            CycleValue::Integer(i) => Ok(vec![new_note(Note::from((*i).clamp(0, 0x7f) as u8))]),
            CycleValue::Pitch(p) => Ok(vec![new_note(Note::from(p.midi_note()))]),
            CycleValue::Chord(p, m) => {
                let chord = Chord::try_from((p.midi_note(), m.as_ref()))?;
                Ok(chord
                    .intervals()
                    .iter()
                    .map(|i| new_note(chord.note().transposed(*i as i32)))
                    .collect())
            }
            CycleValue::Name(s) => {
                if s.eq_ignore_ascii_case("off") {
                    Ok(vec![new_note(Note::OFF)])
                } else {
                    Ok(vec![None])
                }
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Helper struct to convert time tagged events from Cycle into a `Vec<EventIterItem>`
pub(crate) struct CycleNoteEvents {
    // collected events for a given time span per channels
    events: Vec<(Fraction, Fraction, Vec<Option<Event>>)>,
    // max note event count per channel
    event_counts: Vec<usize>,
}

impl CycleNoteEvents {
    /// Create a new, empty list of events.
    pub fn new() -> Self {
        let events = Vec::with_capacity(16);
        let event_counts = Vec::with_capacity(3);
        Self {
            events,
            event_counts,
        }
    }

    /// Add a single note event stack from a cycle channel event.
    pub fn add(
        &mut self,
        channel: usize,
        start: Fraction,
        length: Fraction,
        note_events: Vec<Option<NoteEvent>>,
    ) {
        // memorize max event count per channel
        if self.event_counts.len() <= channel {
            self.event_counts.resize(channel + 1, 0);
        }
        self.event_counts[channel] = self.event_counts[channel].max(note_events.len());
        // insert events into existing time slot or a new one
        match self
            .events
            .binary_search_by(|(time, _, _)| time.cmp(&start))
        {
            Ok(pos) => {
                // use min length of all notes in stack
                let event_length = &mut self.events[pos].1;
                *event_length = (*event_length).min(length);
                // add new notes to existing events
                let timed_event = &mut self.events[pos].2;
                timed_event.resize(channel + 1, None);
                timed_event[channel] = Some(Event::NoteEvents(note_events));
            }
            Err(pos) => {
                // insert a new time event
                let mut timed_event = Vec::with_capacity(channel + 1);
                timed_event.resize(channel + 1, None);
                timed_event[channel] = Some(Event::NoteEvents(note_events));
                self.events.insert(pos, (start, length, timed_event))
            }
        }
    }

    /// Convert to a list of EventIterItems.
    pub fn into_event_iter_items(self) -> Vec<EventIterItem> {
        // max number of note events in a single merged down Event
        let max_event_count = self.event_counts.iter().sum::<usize>();
        // apply padding per channel, merge down and convert to EventIterItem
        let mut event_iter_items: Vec<EventIterItem> = Vec::with_capacity(self.events.len());
        for (start_time, length, mut events) in self.events.into_iter() {
            // ensure that each event in the channel, contains the same number of note events
            for (channel, mut event) in events.iter_mut().enumerate() {
                if let Some(Event::NoteEvents(note_events)) = &mut event {
                    // pad existing note events with OFFs
                    note_events.resize_with(self.event_counts[channel], || new_note(Note::OFF));
                } else if self.event_counts[channel] > 0 {
                    // pad missing note events with 'None'
                    *event = Some(Event::NoteEvents(vec![None; self.event_counts[channel]]))
                }
            }
            // merge all events that happen at the same time together
            let mut merged_note_events = Vec::with_capacity(max_event_count);
            for mut event in events.into_iter().flatten() {
                if let Event::NoteEvents(note_events) = &mut event {
                    merged_note_events.append(note_events);
                }
            }
            // convert padded, merged note events to a timed 'Event'
            let event = Event::NoteEvents(merged_note_events);
            event_iter_items.push(EventIterItem::new_with_fraction(event, start_time, length));
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
    pub fn from_mini_with_seed(input: &str, seed: u64) -> Result<Self, String> {
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
    fn map_note_event(&mut self, event: CycleEvent) -> Result<Vec<Option<NoteEvent>>, String> {
        let mut note_events = {
            if let Some(note_events) = self.mappings.get(event.string()) {
                // apply custom note mappings
                note_events.clone()
            } else {
                // try converting the cycle value to a single note
                event.value().try_into()?
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
    fn generate(&mut self) -> Vec<EventIterItem> {
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
                match self.map_note_event(event) {
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

    fn set_input_parameters(&mut self, _parameters: InputParameterMap) {
        // nothing to do
    }

    fn run(&mut self, _pulse: PulseIterItem, emit_event: bool) -> Option<Vec<EventIterItem>> {
        if emit_event {
            Some(self.generate())
        } else {
            None
        }
    }

    fn advance(&mut self, _pulse: PulseIterItem, emit_event: bool) {
        if emit_event {
            self.cycle.advance();
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

pub fn new_cycle_event_with_seed(input: &str, seed: u64) -> Result<CycleEventIter, String> {
    CycleEventIter::from_mini_with_seed(input, seed)
}
