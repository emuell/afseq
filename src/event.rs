use std::fmt::Debug;

// -------------------------------------------------------------------------------------------------

/// Id to refer to a specific instrument in a NoteEvent.
type InstrumentId = usize;
/// Id to refer to a specific parameter in a ParameterChangeEvent.
type ParameterId = usize;

// -------------------------------------------------------------------------------------------------

/// Single note event in an [`EmitterEvent`].
#[derive(Clone)]
pub struct NoteEvent {
    pub instrument: Option<InstrumentId>,
    pub note: u32,
    pub velocity: f32,
}

/// Single parameter change event in an [`EmitterEvent`].
#[derive(Clone)]
pub struct ParameterChangeEvent {
    pub parameter: Option<ParameterId>,
    pub value: f32,
}

// -------------------------------------------------------------------------------------------------

/// Events which may get triggered by [Emitter](`super::EmitterValue`) iterators.
#[derive(Clone)]
pub enum EmitterEvent {
    NoteEvents(Vec<NoteEvent>),
    ParameterChangeEvent(ParameterChangeEvent),
}

impl EmitterEvent {
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

impl Debug for EmitterEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmitterEvent::NoteEvents(note_vector) => f.write_fmt(format_args!(
                "{:?}",
                note_vector
                    .iter()
                    .map(|f| format!("{:?}", f))
                    .collect::<Vec<String>>()
                    .join(" | ")
            )),
            EmitterEvent::ParameterChangeEvent(change) => f.write_fmt(format_args!("{:?}", change)),
        }
    }
}
