//! Various lua->rust conversion helpers

use std::{cell::RefCell, ops::RangeBounds, rc::Rc, sync::Arc};

use mlua::prelude::*;

use crate::{
    bindings::{note::NoteUserData, sequence::SequenceUserData},
    prelude::*,
};

// ---------------------------------------------------------------------------------------------

// Error helpers
pub(crate) fn bad_argument_error<S1: Into<Option<&'static str>>, S2: Into<Option<&'static str>>>(
    func: S1,
    arg: S2,
    pos: usize,
    message: &str,
) -> LuaError {
    LuaError::BadArgument {
        to: func.into().map(String::from),
        name: arg.into().map(String::from),
        pos,
        cause: Arc::new(LuaError::RuntimeError(message.to_string())),
    }
}

// ---------------------------------------------------------------------------------------------

impl<'lua> IntoLua<'lua> for Note {
    fn into_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        self.to_string().into_lua(lua)
    }
}

impl<'lua> FromLua<'lua> for Note {
    fn from_lua(value: LuaValue<'lua>, _lua: &'lua Lua) -> LuaResult<Self> {
        match value {
            LuaValue::Integer(note_value) => Ok(Note::from(note_value as u8)),
            LuaValue::String(str) => {
                Note::try_from(&str.to_string_lossy() as &str).map_err(|err| {
                    LuaError::FromLuaConversionError {
                        from: "string",
                        to: "note",
                        message: Some(err.to_string()),
                    }
                })
            }
            _ => {
                return Err(LuaError::FromLuaConversionError {
                    from: value.type_name(),
                    to: "note",
                    message: Some("expected a note number or note string".to_string()),
                })
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------

impl<'lua> IntoLua<'lua> for NoteEvent {
    fn into_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        let table = lua.create_table()?;
        table.set("key", self.note.into_lua(lua)?)?;
        if let Some(instrument) = self.instrument {
            table.set("instrument", usize::from(instrument) as i64)?;
        }
        table.set("volume", self.volume as f64)?;
        table.set("panning", self.panning as f64)?;
        table.set("delay", self.delay as f64)?;
        Ok(LuaValue::Table(table))
    }
}

// ---------------------------------------------------------------------------------------------

// Check if a lua value is a sequence (an array alike table).
pub(crate) fn sequence_from_value<'lua>(
    value: &'lua LuaValue<'lua>,
) -> Option<Vec<LuaValue<'lua>>> {
    if let Some(table) = value.as_table() {
        sequence_from_table(table)
    } else {
        None
    }
}

// Check if a lua table is a sequence (an array alike table).
pub(crate) fn sequence_from_table<'lua>(
    table: &'lua LuaTable<'lua>,
) -> Option<Vec<LuaValue<'lua>>> {
    let sequence = table
        .clone()
        .sequence_values::<LuaValue>()
        .collect::<Vec<_>>();
    if !sequence.is_empty() {
        return Some(
            sequence
                .into_iter()
                .map(|value: LuaResult<LuaValue<'lua>>| value.unwrap())
                .collect(),
        );
    }
    None
}

// ---------------------------------------------------------------------------------------------

fn float_array_from_value<Range>(
    lua: &Lua,
    value: LuaValue,
    array_len: usize,
    name: &str,
    range: Range,
    _default: f32,
) -> LuaResult<Vec<f32>>
where
    Range: RangeBounds<f32> + std::fmt::Debug,
{
    let values;
    if let Some(value_table) = value.as_table() {
        values = value_table
            .clone()
            .sequence_values::<f32>()
            .collect::<LuaResult<Vec<f32>>>()?;
    } else {
        let value = f32::from_lua(value, lua)?;
        values = (0..array_len).map(|_| value).collect::<Vec<f32>>()
    }
    for value in values.iter() {
        if !range.contains(value) {
            return Err(bad_argument_error(
                None,
                "volume",
                1,
                format!("{} must be in range {:?} but is '{}'", name, range, value).as_str(),
            ));
        }
    }
    Ok(values)
}

pub(crate) fn transpose_steps_array_from_value(
    lua: &Lua,
    step: LuaValue,
    array_len: usize,
) -> LuaResult<Vec<i32>> {
    let steps;
    if let Some(volume_table) = step.as_table() {
        steps = volume_table
            .clone()
            .sequence_values::<i32>()
            .collect::<LuaResult<Vec<i32>>>()?;
    } else {
        let step = i32::from_lua(step, lua)?;
        steps = (0..array_len).map(|_| step).collect::<Vec<i32>>()
    }
    Ok(steps)
}

pub(crate) fn volume_array_from_value(
    lua: &Lua,
    value: LuaValue,
    array_len: usize,
) -> LuaResult<Vec<f32>> {
    float_array_from_value(lua, value, array_len, "volume", 0.0..=f32::MAX, 1.0)
}

pub(crate) fn panning_array_from_value(
    lua: &Lua,
    value: LuaValue,
    array_len: usize,
) -> LuaResult<Vec<f32>> {
    float_array_from_value(lua, value, array_len, "panning", -1.0..=1.0, 0.0)
}

pub(crate) fn delay_array_from_value(
    lua: &Lua,
    value: LuaValue,
    array_len: usize,
) -> LuaResult<Vec<f32>> {
    float_array_from_value(lua, value, array_len, "delay", 0.0..=1.0, 0.0)
}

// ---------------------------------------------------------------------------------------------

fn float_value_from_table<Range>(
    table: &LuaTable,
    name: &'static str,
    range: Range,
    default: f32,
) -> LuaResult<f32>
where
    Range: RangeBounds<f32> + std::fmt::Debug,
{
    if table.contains_key::<&str>(name)? {
        if let Ok(value) = table.get::<&str, f32>(name) {
            if !range.contains(&value) {
                Err(LuaError::FromLuaConversionError {
                    from: "string",
                    to: "number",
                    message: Some(format!(
                        "'{}' property must be in range {:?} but is '{}'",
                        name, range, value
                    )),
                })
            } else {
                Ok(value)
            }
        } else {
            Err(LuaError::FromLuaConversionError {
                from: name,
                to: "number",
                message: Some(format!("'{}' property is missing but required", name)),
            })
        }
    } else {
        Ok(default)
    }
}

pub(crate) fn volume_value_from_table(table: &LuaTable) -> LuaResult<f32> {
    float_value_from_table(table, "volume", 0.0..=f32::MAX, 1.0)
}

pub(crate) fn panning_value_from_table(table: &LuaTable) -> LuaResult<f32> {
    float_value_from_table(table, "panning", -1.0..=1.0, 0.0)
}

pub(crate) fn delay_value_from_table(table: &LuaTable) -> LuaResult<f32> {
    float_value_from_table(table, "delay", 0.0..1.0, 0.0)
}

pub(crate) fn is_empty_float_value_string(str: &str) -> bool {
    str == ".." || str == "--"
}

fn float_value_from_string<Range>(
    str: &str,
    name: &'static str,
    range: Range,
    default: f32,
) -> LuaResult<f32>
where
    Range: RangeBounds<f32> + std::fmt::Debug,
{
    let mut value = default;
    if !str.is_empty() && !is_empty_float_value_string(str) {
        if let Ok(int) = str.parse::<i32>() {
            value = int as f32;
        } else if let Ok(float) = str.parse::<f32>() {
            value = float;
        } else {
            return Err(LuaError::FromLuaConversionError {
                from: "string",
                to: "number",
                message: Some(format!("'{}' property '{}' is not a number", name, str)),
            });
        }
        if !range.contains(&value) {
            return Err(LuaError::FromLuaConversionError {
                from: "string",
                to: "number",
                message: Some(format!(
                    "'{}' property must be in range {:?} but is '{}'",
                    name, range, value
                )),
            });
        }
    }
    Ok(value)
}

pub(crate) fn volume_value_from_string(str: &str) -> LuaResult<f32> {
    float_value_from_string(str, "volume", 0.0..=f32::MAX, 1.0)
}

pub(crate) fn panning_value_from_string(str: &str) -> LuaResult<f32> {
    float_value_from_string(str, "panning", -1.0..=1.0, 0.0)
}

pub(crate) fn delay_value_from_string(str: &str) -> LuaResult<f32> {
    float_value_from_string(str, "delay", 0.0..1.0, 0.0)
}

// -------------------------------------------------------------------------------------------------

pub(crate) fn is_empty_note_string(s: &str) -> bool {
    matches!(s, "" | "-" | "--" | "---" | "." | ".." | "...")
}

// ---------------------------------------------------------------------------------------------

pub(crate) fn note_event_from_number(
    note_value: LuaInteger,
    default_instrument: Option<InstrumentId>,
) -> LuaResult<Option<NoteEvent>> {
    Ok(new_note((default_instrument, note_value as u8)))
}

pub(crate) fn note_event_from_string(
    str: &str,
    default_instrument: Option<InstrumentId>,
) -> LuaResult<Option<NoteEvent>> {
    let mut white_space_splits = str.split(' ').filter(|v| !v.is_empty());
    let note_part = white_space_splits.next().unwrap_or("");
    if is_empty_note_string(note_part) {
        Ok(None)
    } else {
        let note = Note::try_from(note_part).map_err(|err| LuaError::FromLuaConversionError {
            from: "string",
            to: "note",
            message: Some(err.to_string()),
        })?;
        let volume = volume_value_from_string(white_space_splits.next().unwrap_or(""))?;
        let panning = panning_value_from_string(white_space_splits.next().unwrap_or(""))?;
        let delay = delay_value_from_string(white_space_splits.next().unwrap_or(""))?;
        Ok(new_note((default_instrument, note, volume, panning, delay)))
    }
}

pub(crate) fn note_event_from_table_map(
    table: LuaTable,
    default_instrument: Option<InstrumentId>,
) -> LuaResult<Option<NoteEvent>> {
    if table.is_empty() {
        return Ok(None);
    }
    if table.contains_key("key")? {
        let volume = volume_value_from_table(&table)?;
        let panning = panning_value_from_table(&table)?;
        let delay = delay_value_from_table(&table)?;
        // { key = 60, [volume = 1.0, panning = 0.0, delay = 0.0] }
        if let Ok(note_value) = table.get::<&str, i32>("key") {
            Ok(new_note((
                default_instrument,
                Note::from(note_value as u8),
                volume,
                panning,
                delay,
            )))
        }
        // { key = "C4", [volume = 1.0, panning = 0.0, delay = 0.0] }
        else if let Ok(note_str) = table.get::<&str, String>("key") {
            let note = Note::try_from(note_str.as_str()).map_err(|err| {
                LuaError::FromLuaConversionError {
                    from: "string",
                    to: "note",
                    message: Some(err.to_string()),
                }
            })?;
            Ok(new_note((default_instrument, note, volume, panning, delay)))
        } else {
            Err(LuaError::FromLuaConversionError {
                from: "table",
                to: "note",
                message: Some("invalid 'key' property for note".to_string()),
            })
        }
    } else {
        Err(LuaError::FromLuaConversionError {
            from: "table",
            to: "note",
            message: Some("missing 'key' property for note".to_string()),
        })
    }
}

pub(crate) fn note_event_from_value(
    arg: LuaValue,
    arg_index: Option<usize>,
    default_instrument: Option<InstrumentId>,
) -> LuaResult<Option<NoteEvent>> {
    match arg {
        LuaValue::Nil => Ok(None),
        LuaValue::Integer(note_value) => note_event_from_number(note_value, default_instrument),
        LuaValue::String(str) => note_event_from_string(&str.to_string_lossy(), default_instrument),
        LuaValue::Table(table) => note_event_from_table_map(table, default_instrument),
        _ => {
            return Err(LuaError::FromLuaConversionError {
                from: arg.type_name(),
                to: "Note",
                message: if let Some(index) = arg_index {
                    Some(format!("arg #{} is not a valid note property", index + 1).to_string())
                } else {
                    Some("invalid note property".to_string())
                },
            });
        }
    }
}

pub(crate) fn note_events_from_value(
    arg: LuaValue,
    arg_index: Option<usize>,
    default_instrument: Option<InstrumentId>,
) -> LuaResult<Vec<Option<NoteEvent>>> {
    match arg {
        LuaValue::UserData(userdata) => {
            if let Ok(chord) = userdata.take::<NoteUserData>() {
                Ok(chord.notes)
            } else if userdata.is::<SequenceUserData>() {
                Err(LuaError::FromLuaConversionError {
                    from: "userdata",
                    to: "note",
                    message: Some("can't nest sequences in sequences".to_string()),
                })
            } else {
                Err(LuaError::FromLuaConversionError {
                    from: "userdata",
                    to: "note",
                    message: if let Some(index) = arg_index {
                        Some(
                            format!(
                                "user data at #{} can't be converted to note list",
                                index + 1
                            )
                            .to_string(),
                        )
                    } else {
                        Some("given user data can't be converted to note list".to_string())
                    },
                })
            }
        }
        LuaValue::Table(table) => {
            // array like { "C4", "C5" }
            if let Some(sequence) = sequence_from_table(&table.clone()) {
                let mut note_events = vec![];
                for (arg_index, arg) in sequence.into_iter().enumerate() {
                    // flatten sequence events into a single array
                    note_events.append(&mut note_events_from_value(
                        arg,
                        Some(arg_index),
                        default_instrument,
                    )?);
                }
                Ok(note_events)
            // { key = xxx } map
            } else {
                Ok(vec![note_event_from_value(
                    mlua::Value::Table(table),
                    arg_index,
                    default_instrument,
                )?])
            }
        }
        LuaValue::String(str) => {
            let str = str.to_string_lossy().to_string();
            // a string with ' is a chord
            if str.contains('\'') {
                Ok(chord_events_from_string(&str, default_instrument)?)
            } else {
                Ok(vec![note_event_from_string(&str, default_instrument)?])
            }
        }
        _ => Ok(vec![note_event_from_value(
            arg,
            arg_index,
            default_instrument,
        )?]),
    }
}

pub(crate) fn chord_events_from_string(
    chord_string: &str,
    default_instrument: Option<InstrumentId>,
) -> LuaResult<Vec<Option<NoteEvent>>> {
    let mut white_space_splits = chord_string.split(' ').filter(|v| !v.is_empty());
    let chord_part = white_space_splits.next().unwrap_or("");
    let chord = Chord::try_from(chord_part).map_err(|err| LuaError::FromLuaConversionError {
        from: "string",
        to: "chord",
        message: Some(format!("invalid chord '{}': {}", chord_part, err)),
    })?;
    let volume = volume_value_from_string(white_space_splits.next().unwrap_or(""))?;
    let panning = panning_value_from_string(white_space_splits.next().unwrap_or(""))?;
    let delay = delay_value_from_string(white_space_splits.next().unwrap_or(""))?;
    Ok(chord
        .intervals()
        .iter()
        .map(|i| {
            new_note((
                default_instrument,
                Note::from(chord.note() as u8 + i),
                volume,
                panning,
                delay,
            ))
        })
        .collect::<Vec<_>>())
}

// -------------------------------------------------------------------------------------------------

pub fn pattern_pulse_from_value(value: LuaValue) -> LuaResult<f32> {
    match value {
        LuaValue::Nil => Ok(0.0),
        LuaValue::Boolean(bool) => Ok(bool as u8 as f32),
        LuaValue::Integer(integer) => Ok(integer as f32),
        LuaValue::Number(number) => Ok(number as f32),
        LuaValue::String(str) => {
            let str = str.to_string_lossy();
            if let Ok(number) = str.parse::<f32>() {
                Ok(number)
            } else if let Ok(integer) = str.parse::<i32>() {
                Ok(integer as f32)
            } else if let Ok(bool) = str.parse::<bool>() {
                Ok(bool as u8 as f32)
            } else {
                Err(LuaError::FromLuaConversionError {
                    from: "string",
                    to: "pattern pulse",
                    message: Some("Invalid pattern pulse string value".to_string()),
                })
            }
        }
        _ => Err(LuaError::FromLuaConversionError {
            from: value.type_name(),
            to: "pattern pulse",
            message: Some("Invalid pattern pulse value".to_string()),
        }),
    }
}

// -------------------------------------------------------------------------------------------------

pub(crate) fn pattern_from_value(value: LuaValue) -> LuaResult<Rc<RefCell<dyn Pattern>>> {
    match value {
        LuaValue::Function(func) => {
            let pattern = ScriptedPattern::new(func)?;
            Ok(Rc::new(RefCell::new(pattern)))
        }
        LuaValue::Table(table) => {
            let pulses = table
                .clone()
                .sequence_values::<LuaValue>()
                .map(|result| pattern_pulse_from_value(result?))
                .collect::<LuaResult<Vec<f32>>>()?;
            Ok(Rc::new(RefCell::new(pulses.to_pattern())))
        }
        _ => Err(LuaError::FromLuaConversionError {
            from: value.type_name(),
            to: "pattern",
            message: Some(
                "Expected either an array or a function as pattern generator".to_string(),
            ),
        }),
    }
}

// -------------------------------------------------------------------------------------------------

pub(crate) fn event_iter_from_value(
    value: LuaValue,
    default_instrument: Option<InstrumentId>,
) -> LuaResult<Rc<RefCell<dyn EventIter>>> {
    match value {
        LuaValue::UserData(userdata) => {
            if userdata.is::<NoteUserData>() {
                let note = userdata.take::<NoteUserData>()?;
                Ok(Rc::new(RefCell::new(note.notes.to_event())))
            } else if userdata.is::<SequenceUserData>() {
                let sequence = userdata.take::<SequenceUserData>()?;
                Ok(Rc::new(RefCell::new(sequence.notes.to_event_sequence())))
            } else {
                Err(LuaError::FromLuaConversionError {
                    from: "userdata",
                    to: "note",
                    message: Some(
                        "given user data can't be converted to note event list".to_string(),
                    ),
                })
            }
        }
        LuaValue::Function(function) => {
            let iter = ScriptedEventIter::new(function, default_instrument)?;
            Ok(Rc::new(RefCell::new(iter)))
        }
        LuaValue::Table(ref table) => {
            // convert an array alike table to a event sequence
            if let Some(sequence) = sequence_from_table(&table.clone()) {
                let mut note_event_sequence = vec![];
                for (arg_index, arg) in sequence.into_iter().enumerate() {
                    note_event_sequence.push(note_events_from_value(
                        arg,
                        Some(arg_index),
                        default_instrument,
                    )?);
                }
                let iter = note_event_sequence.to_event_sequence();
                Ok(Rc::new(RefCell::new(iter)))
            }
            // convert table to a single note event
            else {
                let iter = note_event_from_value(value, None, default_instrument)?.to_event();
                Ok(Rc::new(RefCell::new(iter)))
            }
        }
        _ => {
            // try converting a note number or note/chord string to an event iter
            let iter = note_events_from_value(value, None, default_instrument)?.to_event();
            Ok(Rc::new(RefCell::new(iter)))
        }
    }
}
