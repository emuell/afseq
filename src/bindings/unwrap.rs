//! Helper functions to safely unwrap basic and afseq types from rhai Arrays or Dynamics

use rhai::{
    Array, Dynamic, EvalAltResult, ImmutableString, Map, NativeCallContext, Position, FLOAT, INT,
};

use crate::{
    event::{InstrumentId, NoteEvent},
    Note,
};

// ---------------------------------------------------------------------------------------------

/// Minimal context needed to report errors in bindings
pub struct ErrorCallContext<'a> {
    position: Position,
    fn_name: &'a str,
}

impl<'a> ErrorCallContext<'a> {
    pub fn new(fn_name: &'a str, position: Position) -> Self {
        Self { fn_name, position }
    }

    pub fn fn_name(&self) -> &str {
        self.fn_name
    }

    pub fn position(&self) -> Position {
        self.position
    }
}

/// Create ErrorCallContext from a NativeCallContext
impl<'a> From<&'a NativeCallContext<'a>> for ErrorCallContext<'a> {
    fn from(context: &'a NativeCallContext<'a>) -> ErrorCallContext<'a> {
        let position = context.position();
        let fn_name = context.fn_name();
        Self { position, fn_name }
    }
}

// ---------------------------------------------------------------------------------------------

pub fn unwrap_float(
    context: &ErrorCallContext,
    d: Dynamic,
    arg_name: &str,
) -> Result<FLOAT, Box<EvalAltResult>> {
    if let Some(float) = d.clone().try_cast::<FLOAT>() {
        Ok(float)
    } else if let Some(integer) = d.clone().try_cast::<INT>() {
        Ok(integer as f64)
    } else {
        Err(EvalAltResult::ErrorInModule(
            "bindings".to_string(),
            format!(
                "Invalid arg: '{}' in '{}' must be a number value, but is a '{}'",
                arg_name,
                context.fn_name(),
                d.type_name()
            )
            .into(),
            context.position(),
        )
        .into())
    }
}

pub fn unwrap_integer(
    context: &ErrorCallContext,
    d: Dynamic,
    arg_name: &str,
) -> Result<INT, Box<EvalAltResult>> {
    if let Some(float) = d.clone().try_cast::<FLOAT>() {
        Ok(float as INT)
    } else if let Some(integer) = d.clone().try_cast::<INT>() {
        Ok(integer)
    } else {
        Err(EvalAltResult::ErrorInModule(
            "bindings".to_string(),
            format!(
                "Invalid arg: '{}' in '{}' must be a number value, but is a '{}'",
                arg_name,
                context.fn_name(),
                d.type_name()
            )
            .into(),
            context.position(),
        )
        .into())
    }
}

pub fn is_empty_note_string(s: &str) -> bool {
    matches!(s, "" | "-" | "--" | "---" | "." | ".." | "...")
}

pub fn unwrap_note_from_string(
    context: &ErrorCallContext,
    s: &str,
) -> Result<Note, Box<EvalAltResult>> {
    match Note::try_from(s) {
        Ok(note) => Ok(note),
        Err(err) => Err(EvalAltResult::ErrorInModule(
            "bindings".to_string(),
            format!(
                "Failed to parse note in function '{}': {}",
                context.fn_name(),
                err
            )
            .into(),
            context.position(),
        )
        .into()),
    }
}

pub fn unwrap_note_from_int(
    context: &ErrorCallContext,
    note: INT,
) -> Result<Note, Box<EvalAltResult>> {
    if note == 0xff {
        Ok(Note::OFF)
    } else if (0..=0x7f).contains(&note) {
        Ok(Note::from(note as u8))
    } else {
        return Err(EvalAltResult::ErrorInModule(
            "bindings".to_string(),
            format!(
                "Expected a note value in range [0..128] in function '{}', got '{}'",
                context.fn_name(),
                note
            )
            .into(),
            context.position(),
        )
        .into());
    }
}

pub fn unwrap_note_event_from_string(
    context: &ErrorCallContext,
    s: &str,
    default_instrument: Option<InstrumentId>,
) -> Result<Option<NoteEvent>, Box<EvalAltResult>> {
    if is_empty_note_string(s) {
        return Ok(None);
    }
    let (note, offset) = match Note::try_from_with_offset(s) {
        Ok(result) => result,
        Err(err) => {
            return Err(EvalAltResult::ErrorInModule(
                "bindings".to_string(),
                format!(
                    "Failed to parse note in function '{}': {}",
                    context.fn_name(),
                    err
                )
                .into(),
                context.position(),
            )
            .into())
        }
    };
    let mut volume = 1.0;
    let remaining_s = &s[offset..].trim();
    if !remaining_s.is_empty() {
        if let Ok(int) = remaining_s.parse::<i32>() {
            volume = int as f32;
        } else if let Ok(float) = remaining_s.parse::<f32>() {
            volume = float;
        } else {
            return Err(EvalAltResult::ErrorInModule(
                "bindings".to_string(),
                format!(
                    "Failed to parse volume in function '{}': \
                        Argument is neither a float or int value",
                    context.fn_name(),
                )
                .into(),
                context.position(),
            )
            .into());
        }
        if volume < 0.0 {
            return Err(EvalAltResult::ErrorInModule(
                "bindings".to_string(),
                format!(
                    "Failed to parse 'volume' property in function '{}': \
                            Volume must be >= 0, but is '{}'.",
                    context.fn_name(),
                    volume
                )
                .into(),
                context.position(),
            )
            .into());
        }
    }
    Ok(Some(NoteEvent {
        instrument: default_instrument,
        note,
        volume,
    }))
}

pub fn unwrap_note_event_from_map(
    context: &ErrorCallContext,
    map: Map,
    default_instrument: Option<InstrumentId>,
) -> Result<Option<NoteEvent>, Box<EvalAltResult>> {
    if map.is_empty() {
        return Ok(None);
    }
    if let Some(key) = map.get("key") {
        // key
        let note;
        if key.is::<()>() {
            return Ok(None);
        } else if key.is_string() {
            let note_string = key.clone().into_immutable_string()?;
            if is_empty_note_string(note_string.as_str()) {
                return Ok(None);
            }
            note = unwrap_note_from_string(context, note_string.as_str())?;
        } else if key.is_int() {
            note = unwrap_note_from_int(context, key.as_int()?)?;
        } else {
            return Err(EvalAltResult::ErrorInModule(
                "bindings".to_string(),
                format!(
                    "Failed to parse 'key' property in function '{}': \
                        Argument is neither (), number, or a string, but is a '{}'.",
                    context.fn_name(),
                    key.type_name()
                )
                .into(),
                context.position(),
            )
            .into());
        }
        // volume
        let volume;
        if let Some(vol) = map.get("volume") {
            if vol.is_float() {
                volume = vol.as_float()? as f32;
            } else if vol.is_int() {
                volume = vol.as_int()? as f32;
            } else {
                return Err(EvalAltResult::ErrorInModule(
                    "bindings".to_string(),
                    format!(
                        "Failed to parse 'volume' property in function '{}': \
                            Argument is not a number, but is a '{}'.",
                        context.fn_name(),
                        vol.type_name()
                    )
                    .into(),
                    context.position(),
                )
                .into());
            }
            if volume < 0.0 {
                return Err(EvalAltResult::ErrorInModule(
                    "bindings".to_string(),
                    format!(
                        "Failed to parse 'volume' property in function '{}': \
                            Volume must be >= 0, but is '{}'.",
                        context.fn_name(),
                        volume
                    )
                    .into(),
                    context.position(),
                )
                .into());
            }
        } else {
            volume = 1.0;
        }
        Ok(Some(NoteEvent {
            instrument: default_instrument,
            note,
            volume,
        }))
    } else {
        Err(EvalAltResult::ErrorInModule(
            "bindings".to_string(),
            format!(
                "Failed to parse note in function '{}': \
                    Missing key property in object map.",
                context.fn_name(),
            )
            .into(),
            context.position(),
        )
        .into())
    }
}

pub fn unwrap_note_events_from_dynamic(
    context: &ErrorCallContext,
    d: Dynamic,
    default_instrument: Option<InstrumentId>,
) -> Result<Vec<Option<NoteEvent>>, Box<EvalAltResult>> {
    if d.is::<()>() {
        Ok(vec![None])
    } else if d.is_array() {
        Ok(unwrap_note_events_from_array(
            context,
            d.cast::<Array>(),
            default_instrument,
        )?)
    } else if d.is_map() {
        Ok(vec![unwrap_note_event_from_map(
            context,
            d.cast::<Map>(),
            default_instrument,
        )?])
    } else if d.is_string() {
        Ok(vec![unwrap_note_event_from_string(
            context,
            d.cast::<ImmutableString>().as_str(),
            default_instrument,
        )?])
    } else if d.is_int() {
        let note = unwrap_note_from_int(context, d.cast::<INT>())?;
        let volume = 1.0;
        Ok(vec![Some(NoteEvent {
            instrument: default_instrument,
            note,
            volume,
        })])
    } else {
        Err(EvalAltResult::ErrorInModule(
            "bindings".to_string(),
            format!(
                "Invalid arguments in fn '{}': \
                    expecting an array, object map, string or integer as note value, got '{}'",
                context.fn_name(),
                d.type_name()
            )
            .into(),
            context.position(),
        )
        .into())
    }
}

pub fn unwrap_note_events_from_array(
    context: &ErrorCallContext,
    array: Array,
    default_instrument: Option<InstrumentId>,
) -> Result<Vec<Option<NoteEvent>>, Box<EvalAltResult>> {
    let mut sequence = Vec::with_capacity(array.len());
    if array.is_empty() {
        sequence.push(None);
    } else {
        for item in array {
            if item.is::<()>() {
                sequence.push(None);
            } else if item.is::<Map>() {
                sequence.push(unwrap_note_event_from_map(
                    context,
                    item.cast::<Map>(),
                    default_instrument,
                )?);
            } else if item.is_string() {
                sequence.push(unwrap_note_event_from_string(
                    context,
                    item.cast::<ImmutableString>().as_str(),
                    default_instrument,
                )?);
            } else if item.is_int() {
                let note = unwrap_note_from_int(context, item.cast::<INT>())?;
                let volume = 1.0;
                sequence.push(Some(NoteEvent {
                    instrument: default_instrument,
                    note,
                    volume,
                }));
            } else {
                return Err(EvalAltResult::ErrorInModule(
                    "bindings".to_string(),
                    format!(
                        "Invalid arguments in fn '{}': \
                            expecting an object map, string or integer as note value, got '{}'",
                        context.fn_name(),
                        item.type_name()
                    )
                    .into(),
                    context.position(),
                )
                .into());
            }
        }
    }
    Ok(sequence)
}
