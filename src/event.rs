//! Value and value iters which get emitted by `Rhythms`.

use self::{
    fixed::{FixedEventIter, ToFixedEventValue},
    sequence::{EventIterSequence, ToEventIterSequence},
};

use core::sync::atomic::{AtomicUsize, Ordering};
use std::{
    fmt::{Debug, Display},
    mem,
};

pub mod empty;
pub mod fixed;
pub mod mapped;
pub mod mapped_note;
pub mod sequence;

// -------------------------------------------------------------------------------------------------

/// Id to refer to a specific instrument in a NoteEvent.
pub type InstrumentId = usize;
/// Id to refer to a specific parameter in a ParameterChangeEvent.
pub type ParameterId = usize;

// -------------------------------------------------------------------------------------------------

/// Generate a new unique instrument id.
pub fn unique_instrument_id() -> InstrumentId {
    static ID: AtomicUsize = AtomicUsize::new(0);
    ID.fetch_add(1, Ordering::Relaxed)
}

// -------------------------------------------------------------------------------------------------

/// A note representable in a 7 bit unsigned int. The subscript 'S' to a note means sharp. The
/// subscript 'm' to an octave means negate, so `CSm2` = C# in octave -2.
/// Because it only uses the least significant 7 bits, any value can be interpreted as either an i8
/// or a u8 for free (as the representation is the same in both)
///
/// Note implements From<u8>, Into<u8> as well as From<i8>, Into<i8> and From<&str> so the enum names
/// usually should be completely ignored.
///
/// For From<&str> conversions, the following notation is supported:
/// `C4` (plain), `C-1` (minus 1 octave), `C#1` (sharps), `Db1` (flats),
/// `D_2` (using _ as optional separator), `G 5` (using space as optional separator)
///
/// Beware: From<&str> wil panic when string to note parsing failed! Use Note::try_from then instead.
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash)]
#[allow(non_camel_case_types)]
pub enum Note {
    C_m1 = 0x00,
    CSm1 = 0x01,
    D_m1 = 0x02,
    DSm1 = 0x03,
    E_m1 = 0x04,
    F_m1 = 0x05,
    FSm1 = 0x06,
    G_m1 = 0x07,
    GSm1 = 0x08,
    A_m1 = 0x09,
    ASm1 = 0x0A,
    B_m1 = 0x0B,
    C_0 = 0x0C,
    CS0 = 0x0D,
    D_0 = 0x0E,
    DS0 = 0x0F,
    E_0 = 0x10,
    F_0 = 0x11,
    FS0 = 0x12,
    G_0 = 0x13,
    GS0 = 0x14,
    A_0 = 0x15,
    AS0 = 0x16,
    B_0 = 0x17,
    C_1 = 0x18,
    CS1 = 0x19,
    D_1 = 0x1A,
    DS1 = 0x1B,
    E_1 = 0x1C,
    F_1 = 0x1D,
    FS1 = 0x1E,
    G_1 = 0x1F,
    GS1 = 0x20,
    A_1 = 0x21,
    AS1 = 0x22,
    B_1 = 0x23,
    C_2 = 0x24,
    CS2 = 0x25,
    D_2 = 0x26,
    DS2 = 0x27,
    E_2 = 0x28,
    F_2 = 0x29,
    FS2 = 0x2A,
    G_2 = 0x2B,
    GS2 = 0x2C,
    A_2 = 0x2D,
    AS2 = 0x2E,
    B_2 = 0x2F,
    C_3 = 0x30,
    CS3 = 0x31,
    D_3 = 0x32,
    DS3 = 0x33,
    E_3 = 0x34,
    F_3 = 0x35,
    FS3 = 0x36,
    G_3 = 0x37,
    GS3 = 0x38,
    A_3 = 0x39,
    AS3 = 0x3A,
    B_3 = 0x3B,
    C_4 = 0x3C,
    CS4 = 0x3D,
    D_4 = 0x3E,
    DS4 = 0x3F,
    E_4 = 0x40,
    F_4 = 0x41,
    FS4 = 0x42,
    G_4 = 0x43,
    GS4 = 0x44,
    A_4 = 0x45,
    AS4 = 0x46,
    B_4 = 0x47,
    C_5 = 0x48,
    CS5 = 0x49,
    D_5 = 0x4A,
    DS5 = 0x4B,
    E_5 = 0x4C,
    F_5 = 0x4D,
    FS5 = 0x4E,
    G_5 = 0x4F,
    GS5 = 0x50,
    A_5 = 0x51,
    AS5 = 0x52,
    B_5 = 0x53,
    C_6 = 0x54,
    CS6 = 0x55,
    D_6 = 0x56,
    DS6 = 0x57,
    E_6 = 0x58,
    F_6 = 0x59,
    FS6 = 0x5A,
    G_6 = 0x5B,
    GS6 = 0x5C,
    A_6 = 0x5D,
    AS6 = 0x5E,
    B_6 = 0x5F,
    C_7 = 0x60,
    CS7 = 0x61,
    D_7 = 0x62,
    DS7 = 0x63,
    E_7 = 0x64,
    F_7 = 0x65,
    FS7 = 0x66,
    G_7 = 0x67,
    GS7 = 0x68,
    A_7 = 0x69,
    AS7 = 0x6A,
    B_7 = 0x6B,
    C_8 = 0x6C,
    CS8 = 0x6D,
    D_8 = 0x6E,
    DS8 = 0x6F,
    E_8 = 0x70,
    F_8 = 0x71,
    FS8 = 0x72,
    G_8 = 0x73,
    GS8 = 0x74,
    A_8 = 0x75,
    AS8 = 0x76,
    B_8 = 0x77,
    C_9 = 0x78,
    CS9 = 0x79,
    D_9 = 0x7A,
    DS9 = 0x7B,
    E_9 = 0x7C,
    F_9 = 0x7D,
    FS9 = 0x7E,
    G_9 = 0x7F,
}

impl Note {
    /// Try converting the iven string to a Note
    pub fn try_from(s: &str) -> Result<Self, String> {
        fn is_sharp_symbol(s: &str, index: usize) -> bool {
            if let Some(c) = s.chars().nth(index) {
                if c == 'S' || c == 's' || c == '#' || c == '♮' {
                    return true;
                }
            }
            false
        }
        fn is_flat_symbol(s: &str, index: usize) -> bool {
            if let Some(c) = s.chars().nth(index) {
                if c == 'B' || c == 'b' || c == '♭' {
                    return true;
                }
            }
            false
        }
        fn is_empty_symbol(s: &str, index: usize) -> bool {
            if let Some(c) = s.chars().nth(index) {
                // NB: don't allow '-': it's used for negative octaves
                if c == ' ' || c == '_' {
                    return true;
                }
            }
            false
        }
        fn octave_value_at(s: &str, index: usize) -> Result<i32, String> {
            if let Some(c) = s.chars().nth(index) {
                let octave = match c {
                    '-' => match octave_value_at(s, index + 1) {
                        Ok(octave) => -octave,
                        Err(err) => return Err(err),
                    },
                    '0' => 0,
                    '1' => 1,
                    '2' => 2,
                    '3' => 3,
                    '4' => 4,
                    '5' => 5,
                    '6' => 6,
                    '7' => 7,
                    '8' => 8,
                    '9' => 9,
                    _ => {
                        return Err(format!(
                            "Unexpected note str {} (invalid octave character '{}')",
                            s, c
                        ))
                    }
                };
                Ok(octave)
            } else {
                Err(format!("Invalid note str: {} (too short)", s))
            }
        }
        fn note_value_at(s: &str, index: usize) -> Result<i8, String> {
            if let Some(c) = s.chars().nth(index) {
                let note = match c {
                    'c' | 'C' => 0,
                    'd' | 'D' => 2,
                    'e' | 'E' => 4,
                    'f' | 'F' => 5,
                    'g' | 'G' => 7,
                    'a' | 'A' => 9,
                    'b' | 'B' => 11,
                    _ => {
                        return Err(format!(
                            "Invalid note str '{}' (unexpected note character '{}')",
                            s, c
                        ))
                    }
                };
                if is_sharp_symbol(s, 1) {
                    Ok(note + 1)
                } else if is_flat_symbol(s, 1) {
                    Ok(note - 1)
                } else {
                    Ok(note)
                }
            } else {
                return Err(format!("Invalid note str '{}' (too short)", s));
            }
        }

        let note = note_value_at(s, 0)? as i32;
        let octave = if is_sharp_symbol(s, 1) || is_flat_symbol(s, 1) || is_empty_symbol(s, 1) {
            octave_value_at(s, 2)?
        } else {
            octave_value_at(s, 1)?
        };
        if octave < -1 || octave > 9 {
            return Err(format!(
                "Invalid note str '{}' (octave out of range '{}')",
                s, octave
            ));
        }
        Ok(((octave * 12 + 12 + note) as u8).into())
    }
}

impl From<u8> for Note {
    #[inline(always)]
    fn from(n: u8) -> Note {
        unsafe { mem::transmute(n & 0x7f) }
    }
}

impl From<Note> for u8 {
    #[inline(always)]
    fn from(note: Note) -> u8 {
        note as u8
    }
}

impl From<i8> for Note {
    #[inline(always)]
    fn from(n: i8) -> Note {
        unsafe { mem::transmute(n & 0x7f) }
    }
}

impl From<Note> for i8 {
    #[inline(always)]
    fn from(note: Note) -> Self {
        note as i8
    }
}

impl From<&str> for Note {
    fn from(s: &str) -> Self {
        match Self::try_from(s) {
            Ok(n) => n,
            Err(err) => panic!("{}", err),
        }
    }
}

impl Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        static NOTE_NAMES: [&str; 12] = [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ];

        let octave = (*self as u8 / 12) as i32 - 1;
        let note = (*self as u8 % 12) as usize;
        write!(f, "{}{}", NOTE_NAMES[note], octave)
    }
}

// -------------------------------------------------------------------------------------------------

/// Single note event in a [`Event`].
#[derive(Clone)]
pub struct NoteEvent {
    pub instrument: Option<InstrumentId>,
    pub note: Note,
    pub velocity: f32,
}

/// Shortcut for creating a new [`NoteEvent`].
pub fn new_note<I: Into<Option<InstrumentId>>, N: Into<Note>>(
    instrument: I,
    note: N,
    velocity: f32,
) -> NoteEvent {
    let instrument: Option<InstrumentId> = instrument.into();
    let note: Note = note.into();
    NoteEvent {
        instrument,
        note,
        velocity,
    }
}

/// Shortcut for creating a vector of [`NoteEvent`]:
/// e.g. a sequence of single notes
pub fn new_note_vector<I: Into<Option<InstrumentId>>, N: Into<Note>>(
    sequence: Vec<(I, N, f32)>,
) -> Vec<NoteEvent> {
    let mut event_sequence = Vec::with_capacity(sequence.len());
    for (instrument, note, velocity) in sequence {
        let instrument = instrument.into();
        let note = note.into();
        event_sequence.push(NoteEvent {
            instrument,
            note,
            velocity,
        });
    }
    event_sequence
}

/// Shortcut for creating a new sequence of polyphonic [`NoteEvent`]:
/// e.g. a sequence of chords
pub fn new_polyphonic_note_sequence<I: Into<Option<InstrumentId>>, N: Into<Note>>(
    polyphonic_sequence: Vec<Vec<(I, N, f32)>>,
) -> Vec<Vec<NoteEvent>> {
    let mut polyphonic_event_sequence = Vec::with_capacity(polyphonic_sequence.len());
    for sequence in polyphonic_sequence {
        let mut event_sequence = Vec::with_capacity(sequence.len());
        for (instrument, note, velocity) in sequence {
            let instrument = instrument.into();
            let note = note.into();
            event_sequence.push(NoteEvent {
                instrument,
                note,
                velocity,
            });
        }
        polyphonic_event_sequence.push(event_sequence);
    }
    polyphonic_event_sequence
}

/// Shortcut for creating a new [`NoteEvent`] [`EventIter`].
pub fn new_note_event<I: Into<Option<InstrumentId>>, N: Into<Note>>(
    instrument: I,
    note: N,
    velocity: f32,
) -> FixedEventIter {
    new_note(instrument, note, velocity).to_event()
}

/// Shortcut for creating a new sequence of [`NoteEvent`] [`EventIter`].
pub fn new_note_event_sequence<I: Into<Option<InstrumentId>>, N: Into<Note>>(
    sequence: Vec<(I, N, f32)>,
) -> EventIterSequence {
    new_note_vector(sequence).to_event_sequence()
}

/// Shortcut for creating a single [`EventIter`] from multiple [`NoteEvent`]:
/// e.g. a chord.
pub fn new_polyphonic_note_event<I: Into<Option<InstrumentId>>, N: Into<Note>>(
    polyphonic_events: Vec<(I, N, f32)>,
) -> FixedEventIter {
    new_note_vector(polyphonic_events).to_event()
}

/// Shortcut for creating a single [`EventIter`] from multiple [`NoteEvent`]:
/// e.g. a sequence of chords.
pub fn new_polyphonic_note_sequence_event<I: Into<Option<InstrumentId>>, N: Into<Note>>(
    polyphonic_sequence: Vec<Vec<(I, N, f32)>>,
) -> EventIterSequence {
    new_polyphonic_note_sequence(polyphonic_sequence).to_event_sequence()
}

// -------------------------------------------------------------------------------------------------

/// Single parameter change event in a [`Event`].
#[derive(Clone)]
pub struct ParameterChangeEvent {
    pub parameter: Option<ParameterId>,
    pub value: f32,
}

/// Shortcut for creating a new [`ParameterChangeEvent`].
pub fn new_parameter_change<Parameter: Into<Option<ParameterId>>>(
    parameter: Parameter,
    value: f32,
) -> ParameterChangeEvent {
    let parameter: Option<ParameterId> = parameter.into();
    ParameterChangeEvent { parameter, value }
}

/// Shortcut for creating a new [`ParameterChangeEvent`] [`EventIter`].
pub fn new_parameter_change_event<Parameter: Into<Option<ParameterId>>>(
    parameter: Parameter,
    value: f32,
) -> FixedEventIter {
    new_parameter_change(parameter, value).to_event()
}

// -------------------------------------------------------------------------------------------------

/// Event which gets triggered by [`EventIter`].
#[derive(Clone)]
pub enum Event {
    NoteEvents(Vec<NoteEvent>),
    ParameterChangeEvent(ParameterChangeEvent),
}

// -------------------------------------------------------------------------------------------------

/// A resettable [`Event`] iterator, which typically will be used in
/// [Rhythm](`super::Rhythm`) trait impls to sequencially emit new events.
pub trait EventIter: Iterator<Item = Event> {
    /// Reset/rewind the iterator to its initial state.
    fn reset(&mut self);
}

// -------------------------------------------------------------------------------------------------

impl Debug for NoteEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{} {} {:.2}",
            if let Some(instrument) = self.instrument {
                format!("{:02}", instrument)
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
            if let Some(parameter) = self.parameter {
                format!("{:02}", parameter)
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

// --------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::Note;

    #[test]
    fn note_number_conversion() {
        assert_eq!(Note::from(0x0_u8), Note::C_m1);
        assert_eq!(Note::from(0x30_u8), Note::C_3);
        assert_eq!(Note::from(0x80_u8), Note::C_m1); // wraps, probably should not be allowed
        assert_eq!(i8::from(Note::C_4), 60);
        assert_eq!(u8::from(Note::C_3), 0x30_u8);
    }

    #[test]
    fn note_serialization() {
        assert_eq!(Note::C_4.to_string(), "C4");
        assert_eq!(Note::CSm1.to_string(), "C#-1");
        assert_eq!(Note::G_9.to_string(), "G9");
    }

    #[test]
    fn note_deserialization() {
        assert!(Note::try_from("").is_err());
        assert!(Note::try_from("x4").is_err());
        assert!(Note::try_from("c-2").is_err());
        assert!(Note::try_from("cc2").is_err());
        assert!(Note::try_from("cbb2").is_err());
        assert!(Note::try_from("c##2").is_err());
        assert_eq!(Note::from("C4"), Note::C_4);
        assert_eq!(Note::from("Cb4"), Note::B_3);
        assert_eq!(Note::from("C#3"), Note::CS3);
        assert_eq!(Note::from("D#-1"), Note::DSm1);
        assert_eq!(Note::from("E_7"), Note::E_7);
        assert_eq!(Note::from("f5"), Note::F_5);
        assert_eq!(Note::from("g 9"), Note::G_9);
        assert_eq!(Note::from("A 8"), Note::A_8);
        assert_eq!(Note::from("bb2"), Note::AS2);
    }
}
