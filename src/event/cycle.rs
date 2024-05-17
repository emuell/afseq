use std::borrow::Cow;

use fraction::Fraction;

use crate::{
    event::{new_note, Event, EventIter, EventIterItem, InstrumentId, NoteEvent},
    tidal::{Cycle, Target as CycleTarget, Value as CycleValue},
    BeatTimeBase, Note, PulseIterItem,
};

// -------------------------------------------------------------------------------------------------

/// Emits a vector of [`Event`]S from a Tidal [`Cycle`].
///
/// Channels from cycle are merged down into note events on different voices.
/// Float and String targets are currently unsupported and will result into None events.
#[derive(Clone, Debug)]
pub struct CycleEventIter {
    cycle: Cycle,
}

impl CycleEventIter {
    /// Create a new cycle event iter from the given precompiled cycle
    pub(crate) fn new(cycle: Cycle) -> Self {
        Self { cycle }
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

    /// Generate next batch of events from the next cycle run.
    /// Converts cycle events to note events and flattens channels into note columns.
    fn generate_events(&mut self) -> Vec<EventIterItem> {
        // convert cycle channel items to a list of note events, sorted by time
        let mut timed_note_events: Vec<(Fraction, Fraction, Vec<Option<NoteEvent>>)> = Vec::new();
        for (channel_index, channel) in self.cycle.generate().into_iter().enumerate() {
            for event in channel.into_iter() {
                let start = event.span().start();
                let length = event.span().length();
                let instrument = match event.target() {
                    CycleTarget::None => None,
                    CycleTarget::Index(i) => Some(InstrumentId::from(i as usize)),
                    CycleTarget::Name(_) => None, // unsupported
                };
                let note_event = match event.value() {
                    CycleValue::Hold => None,
                    CycleValue::Rest => new_note((Note::OFF, instrument)),
                    CycleValue::Float(_) => None, // unsupported
                    CycleValue::Integer(i) => new_note((Note::from(i as u8), instrument)),
                    CycleValue::Pitch(p) => new_note((Note::from(p.midi_note()), instrument)),
                    CycleValue::Name(_) => None, // TODO
                };
                if let Some(note_event) = note_event {
                    match timed_note_events.binary_search_by(|(time, _, _)| time.cmp(&start)) {
                        Ok(pos) => {
                            // use max length of all notes in stack
                            let note_length = &mut timed_note_events[pos].1;
                            *note_length = (*note_length).max(length);
                            // add note to existing time stack
                            let note_events = &mut timed_note_events[pos].2;
                            note_events.resize(channel_index + 1, None);
                            note_events[channel_index] = Some(note_event);
                        }
                        Err(pos) => {
                            timed_note_events.insert(pos, (start, length, vec![Some(note_event)]))
                        }
                    }
                }
            }
        }
        // convert to a list of NoteEvents, applying start time as note delay
        let mut events: Vec<EventIterItem> = Vec::with_capacity(timed_note_events.len());
        for (start_time, length, note_events) in timed_note_events.into_iter() {
            events.push(EventIterItem::new_with_fraction(
                Event::NoteEvents(note_events),
                start_time,
                length,
            ));
        }
        events
    }
}

impl EventIter for CycleEventIter {
    fn set_time_base(&mut self, _time_base: &BeatTimeBase) {
        // nothing to do
    }

    fn set_external_context(&mut self, _data: &[(Cow<str>, f64)]) {
        // nothing to do
    }

    fn run(
        &mut self,
        _pulse: PulseIterItem,
        _pulse_pattern_length: usize,
        emit_event: bool,
    ) -> Option<Vec<EventIterItem>> {
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