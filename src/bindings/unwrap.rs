//! Helper functions to safely unwrap basic and afseq types from rhai Arrays or Dynamics

use rhai::{Array, Dynamic, EvalAltResult, NativeCallContext, Position, FLOAT, INT};

use crate::{event::InstrumentId, Note};

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

pub fn unwrap_array(context: &ErrorCallContext, d: Dynamic) -> Result<Array, Box<EvalAltResult>> {
    let array_result = d.into_array();
    if let Err(other_type) = array_result {
        return Err(EvalAltResult::ErrorInModule(
            "bindings".to_string(),
            format!(
                "Expected 'array' arg in function '{}', got '{}'",
                context.fn_name(),
                other_type
            )
            .into(),
            context.position(),
        )
        .into());
    }
    Ok(array_result.unwrap())
}

pub fn unwrap_note(context: &ErrorCallContext, d: Dynamic) -> Result<Note, Box<EvalAltResult>> {
    if let Ok(integer) = d.as_int() {
        unwrap_note_from_int(context, integer)
    } else if let Ok(string) = d.clone().into_string() {
        unwrap_note_from_string(context, string.as_str())
    } else {
        Err(EvalAltResult::ErrorInModule(
            "bindings".to_string(),
            format!(
                "Failed to parse note in function '{}': Argument is not a string, but a '{}'.",
                context.fn_name(),
                d.type_name()
            )
            .into(),
            context.position(),
        )
        .into())
    }
}

pub fn is_empty_note_value(d: &Dynamic) -> bool {
    d.is::<()>()
        || (d.type_name() == "string" && is_empty_note_string(d.clone().cast::<String>().as_str()))
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
                "Expected a note value in range [0..128] in function '{}', got note '{}'",
                context.fn_name(),
                note
            )
            .into(),
            context.position(),
        )
        .into());
    }
}

pub fn unwrap_note_event(
    context: &ErrorCallContext,
    array: Array,
    default_instrument: Option<InstrumentId>,
) -> Result<(Option<InstrumentId>, Note, f32), Box<EvalAltResult>> {
    if !(1..=3).contains(&array.len()) {
        return Err(EvalAltResult::ErrorInModule(
            "bindings".to_string(),
            format!(
                "Expected 1, 2 or 3 items in note array in function '{}', got '{}' items",
                context.fn_name(),
                array.len()
            )
            .into(),
            context.position(),
        )
        .into());
    }
    if array.len() == 3 {
        let instrument = unwrap_integer(context, array[0].clone(), "instrument")? as usize;
        let note = unwrap_note(context, array[1].clone())?;
        let velocity = unwrap_float(context, array[2].clone(), "velocity")? as f32;
        Ok((Some(InstrumentId::from(instrument)), note, velocity))
    } else if array.len() == 2 {
        let note = unwrap_note(context, array[0].clone())?;
        let velocity = unwrap_float(context, array[1].clone(), "velocity")? as f32;
        Ok((default_instrument, note, velocity))
    } else {
        let note = unwrap_note(context, array[0].clone())?;
        Ok((default_instrument, note, 1.0))
    }
}
