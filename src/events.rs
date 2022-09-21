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

/// Single note event in a [`PatternEvent`].
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

pub trait ToFixedPatternEventValue {
    fn to_event(self) -> self::fixed::FixedPatternEventIter;
}

impl ToFixedPatternEventValue for NoteEvent {
    fn to_event(self) -> self::fixed::FixedPatternEventIter {
        self::fixed::FixedPatternEventIter::new(PatternEvent::NoteEvents(vec![self]))
    }
}

impl ToFixedPatternEventValue for Vec<NoteEvent> {
    fn to_event(self) -> self::fixed::FixedPatternEventIter {
        self::fixed::FixedPatternEventIter::new(PatternEvent::NoteEvents(self))
    }
}

pub trait ToPatternEventValueSequence {
    fn to_event_sequence(self) -> self::sequence::PatternEventIterSequence;
}

impl ToPatternEventValueSequence for Vec<NoteEvent> {
    fn to_event_sequence(self) -> self::sequence::PatternEventIterSequence {
        let mut sequence = Vec::new();
        for note in self {
            sequence.push(PatternEvent::NoteEvents(vec![note]));
        }
        self::sequence::PatternEventIterSequence::new(sequence)
    }
}

impl ToPatternEventValueSequence for Vec<Vec<NoteEvent>> {
    fn to_event_sequence(self) -> self::sequence::PatternEventIterSequence {
        let mut sequence = Vec::new();
        for notes in self {
            sequence.push(PatternEvent::NoteEvents(notes));
        }
        self::sequence::PatternEventIterSequence::new(sequence)
    }
}

// -------------------------------------------------------------------------------------------------

/// Single parameter change event in a [`PatternEvent`].
#[derive(Clone)]
pub struct ParameterChangeEvent {
    pub parameter: Option<ParameterId>,
    pub value: f32,
}

pub fn new_parameter_change(parameter: Option<ParameterId>, value: f32) -> ParameterChangeEvent {
    ParameterChangeEvent { parameter, value }
}

impl ToFixedPatternEventValue for ParameterChangeEvent {
    fn to_event(self) -> self::fixed::FixedPatternEventIter {
        self::fixed::FixedPatternEventIter::new(PatternEvent::ParameterChangeEvent(self))
    }
}

// -------------------------------------------------------------------------------------------------

/// Event which gets triggered by [`PatternEventIter`].
#[derive(Clone)]
pub enum PatternEvent {
    NoteEvents(Vec<NoteEvent>),
    ParameterChangeEvent(ParameterChangeEvent),
}

impl PatternEvent {
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

/// A resettable [`PatternEvent`] iterator, which typically will be used in
/// [Pattern](`super::Pattern`) trait impls to sequencially emit new events.
pub trait PatternEventIter: Iterator<Item = PatternEvent> {
    /// Reset/rewind the iterator to its initial state.
    fn reset(&mut self);
}

/// Converts  arbitary [`PatternEvent`] iterator into a [`PatternEventIter`] implementation in
/// order to fulfy the `PatternEventIter` trait.
pub fn from_iter<Iter>(iter: Iter) -> from_iter::PatternEventIterFromIter<Iter>
where
    Iter: Iterator<Item = PatternEvent> + Clone,
{
    from_iter::PatternEventIterFromIter::new(iter)
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

impl Debug for PatternEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PatternEvent::NoteEvents(note_vector) => f.write_fmt(format_args!(
                "{:?}",
                note_vector
                    .iter()
                    .map(|f| format!("{:?}", f))
                    .collect::<Vec<String>>()
                    .join(" | ")
            )),
            PatternEvent::ParameterChangeEvent(change) => f.write_fmt(format_args!("{:?}", change)),
        }
    }
}
