use std::fmt::Debug;

pub mod fixed;
pub mod from_iter;
pub mod mapped;
pub mod mapped_note;
pub mod sequence;

// -------------------------------------------------------------------------------------------------

/// Id to refer to a specific instrument in a NoteEvent.
pub type InstrumentId = usize;
/// Id to refer to a specific parameter in a ParameterChangeEvent.
pub type ParameterId = usize;

// -------------------------------------------------------------------------------------------------

/// Single note event in a [`Event`].
#[derive(Clone)]
pub struct NoteEvent {
    pub instrument: Option<InstrumentId>,
    pub note: u32,
    pub velocity: f32,
}

pub fn new_note(instrument: Option<InstrumentId>, note: u32, velocity: f32) -> NoteEvent {
    NoteEvent {
        instrument,
        note,
        velocity,
    }
}

pub fn new_note_event(
    instrument: Option<InstrumentId>,
    note: u32,
    velocity: f32,
) -> fixed::FixedEventIter {
    new_note(instrument, note, velocity).to_event()
}

pub trait ToFixedEventValue {
    fn to_event(self) -> self::fixed::FixedEventIter;
}

impl ToFixedEventValue for NoteEvent {
    fn to_event(self) -> self::fixed::FixedEventIter {
        self::fixed::FixedEventIter::new(Event::NoteEvents(vec![self]))
    }
}

impl ToFixedEventValue for Vec<NoteEvent> {
    fn to_event(self) -> self::fixed::FixedEventIter {
        self::fixed::FixedEventIter::new(Event::NoteEvents(self))
    }
}

pub trait ToEventValueSequence {
    fn to_event_sequence(self) -> self::sequence::EventIterSequence;
}

impl ToEventValueSequence for Vec<NoteEvent> {
    fn to_event_sequence(self) -> self::sequence::EventIterSequence {
        let mut sequence = Vec::new();
        for note in self {
            sequence.push(Event::NoteEvents(vec![note]));
        }
        self::sequence::EventIterSequence::new(sequence)
    }
}

impl ToEventValueSequence for Vec<Vec<NoteEvent>> {
    fn to_event_sequence(self) -> self::sequence::EventIterSequence {
        let mut sequence = Vec::new();
        for notes in self {
            sequence.push(Event::NoteEvents(notes));
        }
        self::sequence::EventIterSequence::new(sequence)
    }
}

// -------------------------------------------------------------------------------------------------

/// Single parameter change event in a [`Event`].
#[derive(Clone)]
pub struct ParameterChangeEvent {
    pub parameter: Option<ParameterId>,
    pub value: f32,
}

pub fn new_parameter_change(parameter: Option<ParameterId>, value: f32) -> ParameterChangeEvent {
    ParameterChangeEvent { parameter, value }
}

impl ToFixedEventValue for ParameterChangeEvent {
    fn to_event(self) -> self::fixed::FixedEventIter {
        self::fixed::FixedEventIter::new(Event::ParameterChangeEvent(self))
    }
}

// -------------------------------------------------------------------------------------------------

/// Event which gets triggered by [`EventIter`].
#[derive(Clone)]
pub enum Event {
    NoteEvents(Vec<NoteEvent>),
    ParameterChangeEvent(ParameterChangeEvent),
}

impl Event {
    pub fn new_note(note: NoteEvent) -> Self {
        Self::NoteEvents(vec![note])
    }
    pub fn new_note_vector(notes: Vec<NoteEvent>) -> Self {
        Self::NoteEvents(notes)
    }
    pub fn new_parameter_change(parameter: Option<ParameterId>, value: f32) -> Self {
        Self::ParameterChangeEvent(ParameterChangeEvent { parameter, value })
    }
}

// -------------------------------------------------------------------------------------------------

/// A resettable [`Event`] iterator, which typically will be used in
/// [Rhythm](`super::Rhythm`) trait impls to sequencially emit new events.
pub trait EventIter: Iterator<Item = Event> {
    /// Reset/rewind the iterator to its initial state.
    fn reset(&mut self);
}

/// Converts a std::collection::Iter of [`Event`] into a [`EventIter`] implementation.
pub fn from_iter<Iter>(iter: Iter) -> from_iter::FromIter<Iter>
where
    Iter: Iterator<Item = Event> + Clone,
{
    from_iter::FromIter::new(iter)
}

// -------------------------------------------------------------------------------------------------

impl Debug for NoteEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{} {} {}",
            if self.instrument.is_some() {
                self.instrument.unwrap().to_string()
            } else {
                "NA".to_string()
            },
            self.note,
            self.velocity
        ))
    }
}

impl Debug for ParameterChangeEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{} {}",
            if self.parameter.is_some() {
                self.parameter.unwrap().to_string()
            } else {
                "NA".to_string()
            },
            self.value,
        ))
    }
}

impl Debug for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::NoteEvents(note_vector) => f.write_fmt(format_args!(
                "{:?}",
                note_vector
                    .iter()
                    .map(|f| format!("{:?}", f))
                    .collect::<Vec<String>>()
                    .join(" | ")
            )),
            Event::ParameterChangeEvent(change) => f.write_fmt(format_args!("{:?}", change)),
        }
    }
}
