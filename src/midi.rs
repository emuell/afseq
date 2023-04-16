//! Raw MIDI events used in `Event`.

use std::{
    fmt::Display,
    mem,
    ops::{Add, Sub},
};

use rust_music_theory::note as rmt_note;

// -------------------------------------------------------------------------------------------------

/// A note representable in a 7 bit unsigned int. The subscript 'S' to a note means sharp. The
/// subscript 'm' to an octave means negate, so `CSm2` = C# in octave -2.
/// Because it only uses the least significant 7 bits, any value can be interpreted as either an i8
/// or a u8 for free (as the representation is the same in both)
///
/// Note implements From\<u8\>, Into\<u8\> as well as From\<i8\>, Into\<i8\> and From\<&str\> so the 
/// enum names usually should be completely ignored.
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
    Cm1 = 0x00,
    Csm1 = 0x01,
    Dm1 = 0x02,
    Dsm1 = 0x03,
    Em1 = 0x04,
    Fm1 = 0x05,
    Fsm1 = 0x06,
    Gm1 = 0x07,
    Gsm1 = 0x08,
    Am1 = 0x09,
    Asm1 = 0x0A,
    Bm1 = 0x0B,
    C0 = 0x0C,
    Cs0 = 0x0D,
    D0 = 0x0E,
    Ds0 = 0x0F,
    E0 = 0x10,
    F0 = 0x11,
    Fs0 = 0x12,
    G0 = 0x13,
    Gs0 = 0x14,
    A0 = 0x15,
    As0 = 0x16,
    B0 = 0x17,
    C1 = 0x18,
    Cs1 = 0x19,
    D1 = 0x1A,
    Ds1 = 0x1B,
    E1 = 0x1C,
    F1 = 0x1D,
    Fs1 = 0x1E,
    G1 = 0x1F,
    Gs1 = 0x20,
    A1 = 0x21,
    As1 = 0x22,
    B1 = 0x23,
    C2 = 0x24,
    Cs2 = 0x25,
    D2 = 0x26,
    Ds2 = 0x27,
    E2 = 0x28,
    F2 = 0x29,
    Fs2 = 0x2A,
    G2 = 0x2B,
    Gs2 = 0x2C,
    A2 = 0x2D,
    As2 = 0x2E,
    B2 = 0x2F,
    C3 = 0x30,
    Cs3 = 0x31,
    D3 = 0x32,
    Ds3 = 0x33,
    E3 = 0x34,
    F3 = 0x35,
    Fs3 = 0x36,
    G3 = 0x37,
    Gs3 = 0x38,
    A3 = 0x39,
    As3 = 0x3A,
    B3 = 0x3B,
    C4 = 0x3C,
    Cs4 = 0x3D,
    D4 = 0x3E,
    Ds4 = 0x3F,
    E4 = 0x40,
    F4 = 0x41,
    Fs4 = 0x42,
    G4 = 0x43,
    Gs4 = 0x44,
    A4 = 0x45,
    As4 = 0x46,
    B4 = 0x47,
    C5 = 0x48,
    Cs5 = 0x49,
    D5 = 0x4A,
    Ds5 = 0x4B,
    E5 = 0x4C,
    F5 = 0x4D,
    Fs5 = 0x4E,
    G5 = 0x4F,
    Gs5 = 0x50,
    A5 = 0x51,
    As5 = 0x52,
    B5 = 0x53,
    C6 = 0x54,
    Cs6 = 0x55,
    D6 = 0x56,
    Ds6 = 0x57,
    E6 = 0x58,
    F6 = 0x59,
    Fs6 = 0x5A,
    G6 = 0x5B,
    Gs6 = 0x5C,
    A6 = 0x5D,
    As6 = 0x5E,
    B6 = 0x5F,
    C7 = 0x60,
    Cs7 = 0x61,
    D7 = 0x62,
    Ds7 = 0x63,
    E7 = 0x64,
    F7 = 0x65,
    Fs7 = 0x66,
    G7 = 0x67,
    Gs7 = 0x68,
    A7 = 0x69,
    As7 = 0x6A,
    B7 = 0x6B,
    C8 = 0x6C,
    Cs8 = 0x6D,
    D8 = 0x6E,
    Ds8 = 0x6F,
    E8 = 0x70,
    F8 = 0x71,
    Fs8 = 0x72,
    G8 = 0x73,
    Gs8 = 0x74,
    A8 = 0x75,
    As8 = 0x76,
    B8 = 0x77,
    C9 = 0x78,
    Cs9 = 0x79,
    D9 = 0x7A,
    Ds9 = 0x7B,
    E9 = 0x7C,
    F9 = 0x7D,
    Fs9 = 0x7E,
    G9 = 0x7F,
    // This is NOT a MIDI note, but only internally used
    OFF = 0xFF,
}

impl Note {
    /// Test if this note value is a note-on.
    pub fn is_note_on(&self) -> bool {
        *self != Note::OFF
    }
    /// Test if this note value is a note-off.
    pub fn is_note_off(&self) -> bool {
        *self == Note::OFF
    }

    /// Try converting the given string to a Note
    pub fn try_from(s: &str) -> Result<Self, String> {
        Ok(Self::try_from_with_offset(s)?.0)
    }
    /// Try converting the given string to a Note.
    /// returns Self and the number of consumed characters read from the string.
    pub fn try_from_with_offset(s: &str) -> Result<(Self, usize), String> {
        fn is_note_off(s: &str) -> bool {
            s.to_lowercase() == "off"
        }
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
                            "Invalid note str '{}' - octave character '{}' is invalid.",
                            s, c
                        ))
                    }
                };
                Ok(octave)
            } else {
                Err(format!("Invalid note str '{}' - string is too short.", s))
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
                            "Invalid note str '{}' - note character '{}' is invalid.",
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
                return Err(format!("Invalid note str '{}' - string is too short.", s));
            }
        }

        // Note-Off
        if is_note_off(s) {
            return Ok((Note::OFF, 3));
        }

        // Note-On
        let consumed_chars;
        let note = note_value_at(s, 0)? as i32;
        let octave = if is_sharp_symbol(s, 1) || is_flat_symbol(s, 1) || is_empty_symbol(s, 1) {
            consumed_chars = 3;
            octave_value_at(s, 2)?
        } else {
            consumed_chars = 2;
            octave_value_at(s, 1)?
        };
        if !(-1..=9).contains(&octave) {
            return Err(format!(
                "Invalid note str '{}' - octave '{}' is out of range.",
                s, octave
            ));
        }
        Ok((((octave * 12 + 12 + note) as u8).into(), consumed_chars))
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

impl From<&rmt_note::Note> for Note {
    fn from(note: &rmt_note::Note) -> Self {
        Self::from(note.pitch_class.into_u8() + 12 * note.octave + 12)
    }
}

impl Add<u8> for Note {
    type Output = Self;
    fn add(self, rhs: u8) -> Self {
        Note::from(((self as u8) as i32 + rhs as i32).min(0x7f) as u8)
    }
}

impl Sub<u8> for Note {
    type Output = Self;
    fn sub(self, rhs: u8) -> Self {
        Note::from(((self as u8) as i32 - rhs as i32).max(0) as u8)
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

// --------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::Note;

    #[test]
    fn note_number_conversion() {
        assert_eq!(Note::from(0x0_u8), Note::Cm1);
        assert_eq!(Note::from(0x30_u8), Note::C3);
        assert_eq!(Note::from(0x80_u8), Note::Cm1); // wraps, probably should not be allowed
        assert_eq!(i8::from(Note::C4), 60);
        assert_eq!(u8::from(Note::C3), 0x30_u8);
    }

    #[test]
    fn note_serialization() {
        assert_eq!(Note::C4.to_string(), "C4");
        assert_eq!(Note::Csm1.to_string(), "C#-1");
        assert_eq!(Note::G9.to_string(), "G9");
    }

    #[test]
    fn note_deserialization() {
        assert!(Note::try_from("").is_err());
        assert!(Note::try_from("x4").is_err());
        assert!(Note::try_from("c-2").is_err());
        assert!(Note::try_from("cc2").is_err());
        assert!(Note::try_from("cbb2").is_err());
        assert!(Note::try_from("c##2").is_err());
        assert_eq!(Note::from("C4"), Note::C4);
        assert_eq!(Note::from("Cb4"), Note::B3);
        assert_eq!(Note::from("C#3"), Note::Cs3);
        assert_eq!(Note::from("D#-1"), Note::Dsm1);
        assert_eq!(Note::from("E_7"), Note::E7);
        assert_eq!(Note::from("f5"), Note::F5);
        assert_eq!(Note::from("g 9"), Note::G9);
        assert_eq!(Note::from("A 8"), Note::A8);
        assert_eq!(Note::from("bb2"), Note::As2);
    }
}
