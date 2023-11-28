//! Musical chords as list of `Note` with intervals.

use lazy_static::lazy_static;
use std::collections::HashMap;

use crate::note::Note;

// --------------------------------------------------------------------------------------------------

// major chords
const MAJOR: [u8; 3] = [0, 4, 7];
const AUG: [u8; 3] = [0, 4, 8];
const SIX: [u8; 4] = [0, 4, 7, 9];
const SIX_NINE: [u8; 5] = [0, 4, 7, 9, 14];
const MAJOR7: [u8; 4] = [0, 4, 7, 11];
const MAJOR9: [u8; 5] = [0, 4, 7, 11, 14];
const ADD9: [u8; 4] = [0, 4, 7, 14];
const MAJOR11: [u8; 6] = [0, 4, 7, 11, 14, 17];
const ADD11: [u8; 4] = [0, 4, 7, 17];
const MAJOR13: [u8; 6] = [0, 4, 7, 11, 14, 21];
const ADD13: [u8; 4] = [0, 4, 7, 21];
// dominant chords
const DOM7: [u8; 4] = [0, 4, 7, 10];
const DOM9: [u8; 4] = [0, 4, 7, 14];
const DOM11: [u8; 4] = [0, 4, 7, 17];
const DOM13: [u8; 4] = [0, 4, 7, 21];
const SEVEN_FLAT5: [u8; 4] = [0, 4, 6, 10];
const SEVEN_SHARP5: [u8; 4] = [0, 4, 8, 10];
const SEVEN_FLAT9: [u8; 5] = [0, 4, 7, 10, 13];
const NINE: [u8; 5] = [0, 4, 7, 10, 14];
const ELEVEN: [u8; 6] = [0, 4, 7, 10, 14, 17];
const THIRTEEN: [u8; 7] = [0, 4, 7, 10, 14, 17, 21];
// minor chords
const MINOR: [u8; 3] = [0, 3, 7];
const DIMINISHED: [u8; 3] = [0, 3, 6];
const MINOR_SHARP5: [u8; 3] = [0, 3, 8];
const MINOR6: [u8; 4] = [0, 3, 7, 9];
const MINOR_SIX_NINE: [u8; 5] = [0, 3, 9, 7, 14];
const MINOR7FLAT5: [u8; 4] = [0, 3, 6, 10];
const MINOR7: [u8; 4] = [0, 3, 7, 10];
const MINOR7SHARP5: [u8; 4] = [0, 3, 8, 10];
const MINOR7FLAT9: [u8; 5] = [0, 3, 7, 10, 13];
const MINOR7SHARP9: [u8; 5] = [0, 3, 7, 10, 14];
const DIMINISHED7: [u8; 4] = [0, 3, 6, 9];
const MINOR9: [u8; 5] = [0, 3, 7, 10, 14];
const MINOR11: [u8; 6] = [0, 3, 7, 10, 14, 17];
const MINOR13: [u8; 7] = [0, 3, 7, 10, 14, 17, 21];
const MINOR_MAJOR7: [u8; 4] = [0, 3, 7, 11];
// other chords
const FIVE: [u8; 2] = [0, 7];
const SUS2: [u8; 3] = [0, 2, 7];
const SUS4: [u8; 3] = [0, 5, 7];
const SEVEN_SUS2: [u8; 4] = [0, 2, 7, 10];
const SEVEN_SUS4: [u8; 4] = [0, 5, 7, 10];
const NINE_SUS2: [u8; 5] = [0, 2, 7, 10, 14];
const NINE_SUS4: [u8; 5] = [0, 5, 7, 10, 14];

// map of all known chords with various aliases
lazy_static! {
    static ref CHORD_TABLE: HashMap<&'static str, Vec<u8>> = {
        HashMap::from([
            ("major", Vec::from(MAJOR)),
            ("maj", Vec::from(MAJOR)),
            ("M", Vec::from(MAJOR)),
            ("Δ", Vec::from(MAJOR)),
            ("aug", Vec::from(AUG)),
            ("+", Vec::from(AUG)),
            ("six", Vec::from(SIX)),
            ("6", Vec::from(SIX)),
            ("sixNine", Vec::from(SIX_NINE)),
            ("69", Vec::from(SIX_NINE)),
            ("major7", Vec::from(MAJOR7)),
            ("maj7", Vec::from(MAJOR7)),
            ("M7", Vec::from(MAJOR7)),
            ("Δ7", Vec::from(MAJOR7)),
            ("major9", Vec::from(MAJOR9)),
            ("maj9", Vec::from(MAJOR9)),
            ("M9", Vec::from(MAJOR9)),
            ("Δ9", Vec::from(MAJOR9)),
            ("add9", Vec::from(ADD9)),
            ("+9", Vec::from(ADD9)),
            ("major11", Vec::from(MAJOR11)),
            ("maj11", Vec::from(MAJOR11)),
            ("M11", Vec::from(MAJOR11)),
            ("Δ11", Vec::from(MAJOR11)),
            ("add11", Vec::from(ADD11)),
            ("+11", Vec::from(ADD11)),
            ("major13", Vec::from(MAJOR13)),
            ("maj13", Vec::from(MAJOR13)),
            ("M13", Vec::from(MAJOR13)),
            ("Δ13", Vec::from(MAJOR13)),
            ("add13", Vec::from(ADD13)),
            ("+13", Vec::from(ADD13)),
            ("dom7", Vec::from(DOM7)),
            ("dom9", Vec::from(DOM9)),
            ("dom11", Vec::from(DOM11)),
            ("dom13", Vec::from(DOM13)),
            ("7b5", Vec::from(SEVEN_FLAT5)),
            ("7#5", Vec::from(SEVEN_SHARP5)),
            ("7b9", Vec::from(SEVEN_FLAT9)),
            ("9", Vec::from(NINE)),
            ("nine", Vec::from(NINE)),
            ("eleven", Vec::from(ELEVEN)),
            ("11", Vec::from(ELEVEN)),
            ("thirteen", Vec::from(THIRTEEN)),
            ("13", Vec::from(THIRTEEN)),
            ("minor", Vec::from(MINOR)),
            ("min", Vec::from(MINOR)),
            ("m", Vec::from(MINOR)),
            ("-", Vec::from(MINOR)),
            ("diminished", Vec::from(DIMINISHED)),
            ("dim", Vec::from(DIMINISHED)),
            ("o", Vec::from(DIMINISHED)),
            ("minor#5", Vec::from(MINOR_SHARP5)),
            ("min#5", Vec::from(MINOR_SHARP5)),
            ("m#5", Vec::from(MINOR_SHARP5)),
            ("-#5", Vec::from(MINOR_SHARP5)),
            ("minor6", Vec::from(MINOR6)),
            ("min6", Vec::from(MINOR6)),
            ("m6", Vec::from(MINOR6)),
            ("-6", Vec::from(MINOR6)),
            ("minor69", Vec::from(MINOR_SIX_NINE)),
            ("min69", Vec::from(MINOR_SIX_NINE)),
            ("m69", Vec::from(MINOR_SIX_NINE)),
            ("-69", Vec::from(MINOR_SIX_NINE)),
            ("minor7b5", Vec::from(MINOR7FLAT5)),
            ("min7b5", Vec::from(MINOR7FLAT5)),
            ("m7b5", Vec::from(MINOR7FLAT5)),
            ("-7b5", Vec::from(MINOR7FLAT5)),
            ("minor7", Vec::from(MINOR7)),
            ("min7", Vec::from(MINOR7)),
            ("m7", Vec::from(MINOR7)),
            ("-7", Vec::from(MINOR7)),
            ("minor7#5", Vec::from(MINOR7SHARP5)),
            ("min7#5", Vec::from(MINOR7SHARP5)),
            ("m7#5", Vec::from(MINOR7SHARP5)),
            ("-7#5", Vec::from(MINOR7SHARP5)),
            ("minor7b9", Vec::from(MINOR7FLAT9)),
            ("min7b9", Vec::from(MINOR7FLAT9)),
            ("m7b9", Vec::from(MINOR7FLAT9)),
            ("-7b9", Vec::from(MINOR7FLAT9)),
            ("minor7#9", Vec::from(MINOR7SHARP9)),
            ("min7#9", Vec::from(MINOR7SHARP9)),
            ("m7#9", Vec::from(MINOR7SHARP9)),
            ("-7#9", Vec::from(MINOR7SHARP9)),
            ("diminished7", Vec::from(DIMINISHED7)),
            ("dim7", Vec::from(DIMINISHED7)),
            ("o7", Vec::from(DIMINISHED7)),
            ("minor9", Vec::from(MINOR9)),
            ("min9", Vec::from(MINOR9)),
            ("m9", Vec::from(MINOR9)),
            ("-9", Vec::from(MINOR9)),
            ("minor11", Vec::from(MINOR11)),
            ("min11", Vec::from(MINOR11)),
            ("m11", Vec::from(MINOR11)),
            ("-11", Vec::from(MINOR11)),
            ("minor13", Vec::from(MINOR13)),
            ("min13", Vec::from(MINOR13)),
            ("m13", Vec::from(MINOR13)),
            ("-13", Vec::from(MINOR13)),
            ("minorMajor7", Vec::from(MINOR_MAJOR7)),
            ("minMaj7", Vec::from(MINOR_MAJOR7)),
            ("mM7", Vec::from(MINOR_MAJOR7)),
            ("−Δ7", Vec::from(MINOR_MAJOR7)),
            ("five", Vec::from(FIVE)),
            ("5", Vec::from(FIVE)),
            ("sus2", Vec::from(SUS2)),
            ("sus4", Vec::from(SUS4)),
            ("7sus2", Vec::from(SEVEN_SUS2)),
            ("7sus4", Vec::from(SEVEN_SUS4)),
            ("9sus2", Vec::from(NINE_SUS2)),
            ("9sus4", Vec::from(NINE_SUS4)),
        ])
    };
}

// --------------------------------------------------------------------------------------------------

/// return list of all known chords with aliases.
pub fn chords() -> HashMap<&'static str, Vec<u8>> {
    CHORD_TABLE.clone()
}

/// return list of all known chord names.
pub fn chord_names() -> String {
    CHORD_TABLE
        .keys()
        .map(|name| String::from(*name))
        .collect::<Vec<_>>()
        .join(", ")
}

/// return chord intervals for the given chord string or []
pub fn chord_intervals(p: &str) -> Vec<u8> {
    CHORD_TABLE.get(p).cloned().unwrap_or(vec![])
}

// --------------------------------------------------------------------------------------------------

/// Note vector, created from a root note and intervals.
#[derive(Debug, Clone, PartialEq)]
pub struct Chord {
    note: Note,
    intervals: Vec<u8>,
}

impl Chord {
    /// Create a new chord from the given base note and interval
    pub fn new<N: Into<Note>>(note: N, intervals: Vec<u8>) -> Self {
        Self {
            note: note.into(),
            intervals,
        }
    }

    /// Root note.
    pub fn note(&self) -> Note {
        self.note
    }

    /// Note intervals / steps.
    pub fn intervals(&self) -> &Vec<u8> {
        &self.intervals
    }
}

impl TryFrom<&str> for Chord {
    type Error = String;

    /// Try converting the given string to a chord string in the form:
    /// $note`$chord where $note is a root key or note string and $chord is a key of CHORD_TABLE
    fn try_from(s: &str) -> Result<Self, String> {
        let mut splits = s.split('\'');
        if let Some(note_part) = splits.next() {
            if let Some(chord_part) = splits.next() {
                if splits.next().is_some() {
                    return Err(
                        "Invalid chord string (found more than one ' separator)".to_string()
                    );
                }
                let note = Note::try_from(note_part)?;
                let intervals = CHORD_TABLE.get(chord_part).ok_or(format!(
                    "Invalid chord identifier. Valid chords are: {}",
                    chord_names()
                ))?;
                return Ok(Self::new(note, intervals.to_vec()));
            }
        }
        Err("Invalid chord string. ".to_string()
            + "Expecting a note and chord identifier separated by ' e.g. \"c4'maj\"")
    }
}

// --------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use crate::{Chord, Note};

    #[test]
    fn chord_string_conversion() -> Result<(), String> {
        assert!(Chord::try_from("c").is_err());
        assert!(Chord::try_from("c4").is_err());
        assert!(Chord::try_from("x4'maj").is_err());
        assert!(Chord::try_from("c4'qwe").is_err());
        assert_eq!(
            Chord::try_from("c'maj")?,
            Chord::new(Note::C4, vec![0, 4, 7])
        );
        assert_eq!(
            Chord::try_from("c4'maj")?,
            Chord::new(Note::C4, vec![0, 4, 7])
        );
        Ok(())
    }
}
