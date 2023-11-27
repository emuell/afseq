use std::{cell::RefCell, ops::RangeBounds, rc::Rc, sync::Arc};

use mlua::prelude::*;

use crate::{event::scripted::ScriptedEventIter, prelude::*};

// ---------------------------------------------------------------------------------------------

// Error helpers
pub fn bad_argument_error<S1: Into<Option<&'static str>>, S2: Into<Option<&'static str>>>(
    func: S1,
    arg: S2,
    pos: usize,
    message: &str,
) -> mlua::Error {
    mlua::Error::BadArgument {
        to: func.into().map(String::from),
        name: arg.into().map(String::from),
        pos,
        cause: Arc::new(mlua::Error::RuntimeError(message.to_string())),
    }
}

// ---------------------------------------------------------------------------------------------

impl LuaUserData for Scale {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("notes", |lua, this| -> mlua::Result<LuaTable> {
            lua.create_sequence_from(
                this.notes()
                    .iter()
                    .map(|n| LuaValue::Integer(*n as u8 as i64)),
            )
        })
    }
}

// ---------------------------------------------------------------------------------------------

// Note Userdata in bindings
#[derive(Clone, Debug)]
pub struct NoteUserData {
    pub notes: Vec<Option<NoteEvent>>,
}

impl NoteUserData {
    pub fn from(
        args: LuaMultiValue,
        default_instrument: Option<InstrumentId>,
    ) -> mlua::Result<Self> {
        // a single value, probably a sequence
        let args = args.into_vec();
        if args.len() == 1 {
            let arg = args
                .first()
                .ok_or(mlua::Error::RuntimeError(
                    "Failed to access table content".to_string(),
                ))
                .cloned()?;
            if let Some(sequence) = sequence_from_value(&arg.clone()) {
                let mut notes = vec![];
                for (index, arg) in sequence.into_iter().enumerate() {
                    // flatten sequence events into a single array
                    notes.append(&mut note_events_from_value(
                        arg,
                        Some(index),
                        default_instrument,
                    )?);
                }
                Ok(NoteUserData { notes })
            } else {
                Ok(NoteUserData {
                    notes: note_events_from_value(arg, None, default_instrument)?,
                })
            }
        // multiple values, maybe of different type
        } else {
            let mut notes = vec![];
            for (index, arg) in args.into_iter().enumerate() {
                notes.append(&mut note_events_from_value(
                    arg,
                    Some(index),
                    default_instrument,
                )?);
            }
            Ok(NoteUserData { notes })
        }
    }
}

impl LuaUserData for NoteUserData {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("transpose", |lua, this, value: LuaValue| {
            let steps = transpose_steps_array_from_value(lua, value, this.notes.len())?;
            for (note, step) in this.notes.iter_mut().zip(steps.into_iter()) {
                if let Some(note) = note {
                    if note.note.is_note_on() {
                        let transposed_note = (u8::from(note.note) as i32 + step).max(0).min(0x7f);
                        note.note = Note::from(transposed_note as u8);
                    }
                }
            }
            Ok(this.clone())
        });

        methods.add_method_mut("with_volume", |lua, this, value: LuaValue| {
            let volumes = volume_array_from_value(lua, value, this.notes.len())?;
            for (note, volume) in this.notes.iter_mut().zip(volumes.into_iter()) {
                if let Some(note) = note {
                    note.volume = volume;
                }
            }
            Ok(this.clone())
        });
        methods.add_method_mut("amplify", |lua, this, value: LuaValue| {
            let volumes = volume_array_from_value(lua, value, this.notes.len())?;
            for (note, volume) in this.notes.iter_mut().zip(volumes.into_iter()) {
                if let Some(note) = note {
                    note.volume *= volume;
                }
            }
            Ok(this.clone())
        });
        methods.add_method_mut("with_panning", |lua, this, value: LuaValue| {
            let pannings = panning_array_from_value(lua, value, this.notes.len())?;
            for (note, panning) in this.notes.iter_mut().zip(pannings.into_iter()) {
                if let Some(note) = note {
                    note.panning = panning;
                }
            }
            Ok(this.clone())
        });

        methods.add_method_mut("with_delay", |lua, this, value: LuaValue| {
            let delays = delay_array_from_value(lua, value, this.notes.len())?;
            for (note, delay) in this.notes.iter_mut().zip(delays.into_iter()) {
                if let Some(note) = note {
                    note.delay = delay;
                }
            }
            Ok(this.clone())
        });
    }
}

// ---------------------------------------------------------------------------------------------

// Sequence
#[derive(Clone, Debug)]
pub struct SequenceUserData {
    pub notes: Vec<Vec<Option<NoteEvent>>>,
}

impl SequenceUserData {
    pub fn from(
        args: LuaMultiValue,
        default_instrument: Option<InstrumentId>,
    ) -> mlua::Result<Self> {
        // a single value, probably a sequence array
        let args = args.into_vec();
        if args.len() == 1 {
            let arg = args
                .first()
                .ok_or(mlua::Error::RuntimeError(
                    "Failed to access table content".to_string(),
                ))
                .cloned()?;
            if let Some(sequence) = sequence_from_value(&arg.clone()) {
                let mut notes = vec![];
                for (index, arg) in sequence.into_iter().enumerate() {
                    // add each sequence item as separate sequence event
                    notes.push(note_events_from_value(
                        arg,
                        Some(index),
                        default_instrument,
                    )?);
                }
                Ok(SequenceUserData { notes })
            } else {
                Ok(SequenceUserData {
                    notes: vec![note_events_from_value(arg, None, default_instrument)?],
                })
            }
        // multiple values, maybe of different type
        } else {
            let mut notes = vec![];
            for (index, arg) in args.into_iter().enumerate() {
                notes.push(note_events_from_value(
                    arg,
                    Some(index),
                    default_instrument,
                )?);
            }
            Ok(SequenceUserData { notes })
        }
    }
}

impl LuaUserData for SequenceUserData {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("transpose", |lua, this, volume: LuaValue| {
            let steps = transpose_steps_array_from_value(lua, volume, this.notes.len())?;
            for (notes, step) in this.notes.iter_mut().zip(steps.into_iter()) {
                for note in notes.iter_mut().flatten() {
                    if note.note.is_note_on() {
                        let transposed_note = (u8::from(note.note) as i32 + step).max(0).min(0x7f);
                        note.note = Note::from(transposed_note as u8);
                    }
                }
            }
            Ok(this.clone())
        });

        methods.add_method_mut("with_volume", |lua, this, value: LuaValue| {
            let volumes = volume_array_from_value(lua, value, this.notes.len())?;
            for (notes, volume) in this.notes.iter_mut().zip(volumes) {
                for note in notes.iter_mut().flatten() {
                    note.volume = volume;
                }
            }
            Ok(this.clone())
        });
        methods.add_method_mut("amplify", |lua, this, value: LuaValue| {
            let volumes = volume_array_from_value(lua, value, this.notes.len())?;
            for (notes, volume) in this.notes.iter_mut().zip(volumes) {
                for note in notes.iter_mut().flatten() {
                    note.volume *= volume;
                }
            }
            Ok(this.clone())
        });

        methods.add_method_mut("with_panning", |lua, this, value: LuaValue| {
            let pannings = panning_array_from_value(lua, value, this.notes.len())?;
            for (notes, panning) in this.notes.iter_mut().zip(pannings) {
                for note in notes.iter_mut().flatten() {
                    note.panning = panning;
                }
            }
            Ok(this.clone())
        });

        methods.add_method_mut("with_delay", |lua, this, value: LuaValue| {
            let delays = delay_array_from_value(lua, value, this.notes.len())?;
            for (notes, delay) in this.notes.iter_mut().zip(delays) {
                for note in notes.iter_mut().flatten() {
                    note.delay = delay;
                }
            }
            Ok(this.clone())
        });
    }
}

// ---------------------------------------------------------------------------------------------

// Check if a lua value is a sequence alike table and return it.
fn sequence_from_value<'lua>(value: &'lua LuaValue<'lua>) -> Option<Vec<LuaValue<'lua>>> {
    if let Some(table) = value.as_table() {
        sequence_from_table(table)
    } else {
        None
    }
}

// Check if a lua table is a sequence and return it.
fn sequence_from_table<'lua>(table: &'lua LuaTable<'lua>) -> Option<Vec<LuaValue<'lua>>> {
    let sequence = table
        .clone()
        .sequence_values::<LuaValue>()
        .collect::<Vec<_>>();
    if !sequence.is_empty() {
        return Some(
            sequence
                .into_iter()
                .map(|value: mlua::Result<LuaValue<'lua>>| value.unwrap())
                .collect(),
        );
    }
    None
}

fn value_array_from_value<Range>(
    lua: &Lua,
    value: LuaValue,
    array_len: usize,
    name: &str,
    range: Range,
    _default: f32,
) -> mlua::Result<Vec<f32>>
where
    Range: RangeBounds<f32> + std::fmt::Debug,
{
    let values;
    if let Some(value_table) = value.as_table() {
        values = value_table
            .clone()
            .sequence_values::<f32>()
            .enumerate()
            .map(|(_, result)| result)
            .collect::<mlua::Result<Vec<f32>>>()?;
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

fn transpose_steps_array_from_value(
    lua: &Lua,
    step: LuaValue,
    array_len: usize,
) -> mlua::Result<Vec<i32>> {
    let steps;
    if let Some(volume_table) = step.as_table() {
        steps = volume_table
            .clone()
            .sequence_values::<i32>()
            .enumerate()
            .map(|(_, result)| result)
            .collect::<mlua::Result<Vec<i32>>>()?;
    } else {
        let step = i32::from_lua(step, lua)?;
        steps = (0..array_len).map(|_| step).collect::<Vec<i32>>()
    }
    Ok(steps)
}

fn volume_array_from_value(lua: &Lua, value: LuaValue, array_len: usize) -> mlua::Result<Vec<f32>> {
    value_array_from_value(lua, value, array_len, "volume", 0.0..=f32::MAX, 1.0)
}

fn panning_array_from_value(
    lua: &Lua,
    value: LuaValue,
    array_len: usize,
) -> mlua::Result<Vec<f32>> {
    value_array_from_value(lua, value, array_len, "panning", -1.0..=1.0, 0.0)
}

fn delay_array_from_value(lua: &Lua, value: LuaValue, array_len: usize) -> mlua::Result<Vec<f32>> {
    value_array_from_value(lua, value, array_len, "delay", 0.0..=1.0, 0.0)
}

// ---------------------------------------------------------------------------------------------

fn float_value_from_table<Range>(
    table: &LuaTable,
    name: &str,
    range: Range,
    default: f32,
) -> mlua::Result<f32>
where
    Range: RangeBounds<f32> + std::fmt::Debug,
{
    if table.contains_key::<&str>(name)? {
        if let Ok(value) = table.get::<&str, f32>(name) {
            if !range.contains(&value) {
                Err(mlua::Error::FromLuaConversionError {
                    from: "string",
                    to: "Note",
                    message: Some(format!(
                        "Invalid note {} value: Value must be in range {:?} but is '{}'",
                        name, range, value
                    )),
                })
            } else {
                Ok(value)
            }
        } else {
            Err(mlua::Error::FromLuaConversionError {
                from: "string",
                to: "Note",
                message: Some(format!(
                    "Invalid note {} value: Value is not a number",
                    name
                )),
            })
        }
    } else {
        Ok(default)
    }
}

fn volume_value_from_table(table: &LuaTable) -> mlua::Result<f32> {
    float_value_from_table(table, "volume", 0.0..=f32::MAX, 1.0)
}

fn panning_value_from_table(table: &LuaTable) -> mlua::Result<f32> {
    float_value_from_table(table, "panning", -1.0..=1.0, 0.0)
}

fn delay_value_from_table(table: &LuaTable) -> mlua::Result<f32> {
    float_value_from_table(table, "delay", 0.0..1.0, 0.0)
}

fn is_empty_float_value_string(str: &str) -> bool {
    str == ".." || str == "--"
}

fn float_value_from_string<Range>(
    str: &str,
    name: &str,
    range: Range,
    default: f32,
) -> mlua::Result<f32>
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
            return Err(mlua::Error::FromLuaConversionError {
                from: "string",
                to: "Note",
                message: Some(format!(
                    "Invalid note {} value: Value '{}' is not a number",
                    name, str
                )),
            });
        }
        if !range.contains(&value) {
            return Err(mlua::Error::FromLuaConversionError {
                from: "string",
                to: "Note",
                message: Some(format!(
                    "Invalid note {} value: Value must be in range {:?} but is '{}'",
                    name, range, value
                )),
            });
        }
    }
    Ok(value)
}

fn volume_value_from_string(str: &str) -> mlua::Result<f32> {
    float_value_from_string(str, "volume", 0.0..=f32::MAX, 1.0)
}

fn panning_value_from_string(str: &str) -> mlua::Result<f32> {
    float_value_from_string(str, "panning", -1.0..=1.0, 0.0)
}

fn delay_value_from_string(str: &str) -> mlua::Result<f32> {
    float_value_from_string(str, "delay", 0.0..1.0, 0.0)
}

fn is_empty_note_string(s: &str) -> bool {
    matches!(s, "" | "-" | "--" | "---" | "." | ".." | "...")
}

pub fn note_from_value(arg: LuaValue, arg_index: Option<usize>) -> mlua::Result<Note> {
    match arg {
        LuaValue::Integer(note_value) => Ok(Note::from(note_value as u8)),
        LuaValue::String(str) => Note::try_from(&str.to_string_lossy() as &str).map_err(|err| {
            mlua::Error::FromLuaConversionError {
                from: "string",
                to: "Note",
                message: Some(format!(
                    "Invalid note value '{}': {}",
                    str.to_string_lossy(),
                    err
                )),
            }
        }),
        _ => {
            return Err(mlua::Error::FromLuaConversionError {
                from: arg.type_name(),
                to: "Note",
                message: if let Some(index) = arg_index {
                    Some(
                        format!(
                            "Note arg #{} does not contain a valid note property",
                            index + 1
                        )
                        .to_string(),
                    )
                } else {
                    Some("Argument does not contain a valid note property".to_string())
                },
            });
        }
    }
}

pub fn note_event_from_number(
    note_value: i64,
    default_instrument: Option<InstrumentId>,
) -> mlua::Result<Option<NoteEvent>> {
    Ok(new_note((default_instrument, note_value as u8)))
}

pub fn note_event_from_string(
    str: &str,
    default_instrument: Option<InstrumentId>,
) -> mlua::Result<Option<NoteEvent>> {
    let mut white_space_splits = str.split(' ').filter(|v| !v.is_empty());
    let note_part = white_space_splits.next().unwrap_or("");
    if is_empty_note_string(note_part) {
        Ok(None)
    } else {
        let note =
            Note::try_from(note_part).map_err(|err| mlua::Error::FromLuaConversionError {
                from: "string",
                to: "Note",
                message: Some(format!("Invalid note value '{}': {}", note_part, err)),
            })?;
        let volume = volume_value_from_string(white_space_splits.next().unwrap_or(""))?;
        let panning = panning_value_from_string(white_space_splits.next().unwrap_or(""))?;
        let delay = delay_value_from_string(white_space_splits.next().unwrap_or(""))?;
        Ok(new_note((default_instrument, note, volume, panning, delay)))
    }
}

pub fn chord_events_from_string(
    str: &str,
    default_instrument: Option<InstrumentId>,
) -> mlua::Result<Vec<Option<NoteEvent>>> {
    let mut white_space_splits = str.split(' ').filter(|v| !v.is_empty());
    let chord_part = white_space_splits.next().unwrap_or("");
    let chord = Chord::try_from(chord_part).map_err(|err| mlua::Error::FromLuaConversionError {
        from: "string",
        to: "Note",
        message: Some(format!("Invalid chord value '{}': {}", chord_part, err)),
    })?;
    let volume = volume_value_from_string(white_space_splits.next().unwrap_or(""))?;
    let panning = panning_value_from_string(white_space_splits.next().unwrap_or(""))?;
    let delay = delay_value_from_string(white_space_splits.next().unwrap_or(""))?;
    Ok(chord
        .intervals
        .iter()
        .map(|i| {
            new_note((
                default_instrument,
                Note::from(chord.note as u8 + i),
                volume,
                panning,
                delay,
            ))
        })
        .collect::<Vec<_>>())
}

pub fn note_event_from_table_map(
    table: LuaTable,
    default_instrument: Option<InstrumentId>,
) -> mlua::Result<Option<NoteEvent>> {
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
            if let Ok(note) = Note::try_from(note_str.as_str()) {
                Ok(new_note((default_instrument, note, volume, panning, delay)))
            } else {
                Err(mlua::Error::FromLuaConversionError {
                    from: "string",
                    to: "Note",
                    message: Some(format!("Invalid note value: '{}'", note_str)),
                })
            }
        } else {
            Err(mlua::Error::FromLuaConversionError {
                from: "table",
                to: "Note",
                message: Some("Table does not contain a valid 'key' property".to_string()),
            })
        }
    } else {
        Err(mlua::Error::FromLuaConversionError {
            from: "table",
            to: "Note",
            message: Some("Table does not contain valid note properties".to_string()),
        })
    }
}

pub fn note_event_from_value(
    arg: LuaValue,
    arg_index: Option<usize>,
    default_instrument: Option<InstrumentId>,
) -> mlua::Result<Option<NoteEvent>> {
    match arg {
        LuaValue::Nil => Ok(None),
        LuaValue::Integer(note_value) => note_event_from_number(note_value, default_instrument),
        LuaValue::String(str) => note_event_from_string(&str.to_string_lossy(), default_instrument),
        LuaValue::Table(table) => note_event_from_table_map(table, default_instrument),
        _ => {
            return Err(mlua::Error::FromLuaConversionError {
                from: arg.type_name(),
                to: "Note",
                message: if let Some(index) = arg_index {
                    Some(
                        format!(
                            "Note arg #{} does not contain a valid note property",
                            index + 1
                        )
                        .to_string(),
                    )
                } else {
                    Some("Argument does not contain a valid note property".to_string())
                },
            });
        }
    }
}

pub fn note_events_from_value(
    arg: LuaValue,
    arg_index: Option<usize>,
    default_instrument: Option<InstrumentId>,
) -> mlua::Result<Vec<Option<NoteEvent>>> {
    match arg {
        LuaValue::UserData(userdata) => {
            if userdata.is::<SequenceUserData>() {
                Err(mlua::Error::FromLuaConversionError {
                    from: "Sequence",
                    to: "Note",
                    message: Some("Can not nest sequences into sequences".to_string()),
                })
            } else if let Ok(chord) = userdata.take::<NoteUserData>() {
                Ok(chord.notes)
            } else {
                Err(mlua::Error::FromLuaConversionError {
                    from: "UserData",
                    to: "Note",
                    message: if let Some(index) = arg_index {
                        Some(
                            format!(
                                "Sequence arg #{} does not contain a valid note property",
                                index + 1
                            )
                            .to_string(),
                        )
                    } else {
                        Some("Argument does not contain a valid note property".to_string())
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

// -------------------------------------------------------------------------------------------------

pub fn event_iter_from_value(
    value: LuaValue,
    default_instrument: Option<InstrumentId>,
) -> mlua::Result<Rc<RefCell<dyn EventIter>>> {
    match value {
        LuaValue::UserData(userdata) => {
            if userdata.is::<NoteUserData>() {
                let note = userdata.take::<NoteUserData>()?;
                Ok(Rc::new(RefCell::new(note.notes.to_event())))
            } else if userdata.is::<SequenceUserData>() {
                let sequence = userdata.take::<SequenceUserData>()?;
                Ok(Rc::new(RefCell::new(sequence.notes.to_event_sequence())))
            } else {
                Err(mlua::Error::FromLuaConversionError {
                    from: "table",
                    to: "Note",
                    message: Some("Invalid note table argument".to_string()),
                })
            }
        }
        LuaValue::Function(function) => {
            let iter = ScriptedEventIter::new(function, default_instrument)?;
            Ok(Rc::new(RefCell::new(iter)))
        }
        _ => {
            let iter = note_event_from_value(value, None, default_instrument)?.to_event();
            Ok(Rc::new(RefCell::new(iter)))
        }
    }
}
