//! Lua script bindings for the entire crate.

use std::{cell::RefCell, env, rc::Rc, sync::Arc};

use anyhow::anyhow;
use mlua::{chunk, prelude::*};
use rust_music_theory::{note::Notes, scale};

use crate::{event::scripted::lua::ScriptedEventIter, prelude::*, rhythm::euclidian::euclidean};

// ---------------------------------------------------------------------------------------------

/// Create a new rhai engine with preloaded packages and our default configuation
pub fn new_engine() -> Lua {
    let lua = Lua::new();
    // add cwd/lib to package path
    let cwd = env::current_dir()
        .unwrap_or(".".into())
        .to_string_lossy()
        .to_string();
    lua.load(chunk!(package.path = $cwd.."/assets/lib/?.lua;"..package.path))
        .exec()
        .unwrap_or_else(|err| log::warn!("Failed to initialize lua engine: {}", err));
    lua
}

// -------------------------------------------------------------------------------------------------

// evaluate a script which creates and returns a rhythm
pub fn new_rhythm_from_file(
    instrument: InstrumentId,
    time_base: BeatTimeBase,
    file_name: &str,
) -> Result<Rc<RefCell<dyn Rhythm>>, Box<dyn std::error::Error>> {
    // create a new engine
    let mut lua = new_engine();
    register_bindings(&mut lua, time_base, Some(instrument))?;
    // compile and evaluate script
    let chunk = lua.load(std::path::PathBuf::from(file_name));
    let result = chunk.eval::<LuaValue>()?;
    // convert result
    rhytm_from_value(result)
}

// evaluate a script which creates and returns a rhythm,
// returning a fallback rhythm on errors
pub fn new_rhythm_from_file_with_fallback(
    instrument: InstrumentId,
    time_base: BeatTimeBase,
    file_name: &str,
) -> Rc<RefCell<dyn Rhythm>> {
    new_rhythm_from_file(instrument, time_base, file_name).unwrap_or_else(|err| {
        log::warn!("Script '{}' failed to compile: {}", file_name, err);
        Rc::new(RefCell::new(BeatTimeRhythm::new(
            time_base,
            BeatTimeStep::Beats(1.0),
        )))
    })
}

// evaluate an expression which creates and returns a rhythm
pub fn new_rhythm_from_string(
    instrument: InstrumentId,
    time_base: BeatTimeBase,
    script: &str,
) -> Result<Rc<RefCell<dyn Rhythm>>, Box<dyn std::error::Error>> {
    // create a new engine
    let mut lua = new_engine();
    register_bindings(&mut lua, time_base, Some(instrument))?;
    // compile and evaluate script
    let chunk = lua.load(script);
    let result = chunk.eval::<LuaValue>()?;
    // convert result
    rhytm_from_value(result)
}

// evaluate an expression which creates and returns a rhythm,
// returning a fallback rhythm on errors
pub fn new_rhythm_from_string_with_fallback(
    instrument: InstrumentId,
    time_base: BeatTimeBase,
    expression: &str,
    expression_identifier: &str,
) -> Rc<RefCell<dyn Rhythm>> {
    new_rhythm_from_string(instrument, time_base, expression).unwrap_or_else(|err| {
        log::warn!(
            "Script '{}' failed to compile: {}",
            expression_identifier,
            err
        );
        Rc::new(RefCell::new(BeatTimeRhythm::new(
            time_base,
            BeatTimeStep::Beats(1.0),
        )))
    })
}

// ---------------------------------------------------------------------------------------------

// Chord
#[derive(Clone, Debug)]
struct Chord {
    notes: Vec<Option<NoteEvent>>,
}

// Sequence
#[derive(Clone, Debug)]
struct Sequence {
    notes: Vec<Vec<Option<NoteEvent>>>,
}

impl LuaUserData for Chord {}
impl LuaUserData for Sequence {}
impl LuaUserData for BeatTimeRhythm {
    // BeatTimeRhythm is only passed through ATM
}

// ---------------------------------------------------------------------------------------------

fn is_empty_note_string(s: &str) -> bool {
    matches!(s, "" | "-" | "--" | "---" | "." | ".." | "...")
}

fn note_event_from_number(
    note_value: i64,
    default_instrument: Option<InstrumentId>,
) -> mlua::Result<NoteEvent> {
    Ok(NoteEvent {
        note: crate::midi::Note::from(note_value as u8),
        volume: 1.0,
        instrument: default_instrument,
    })
}

fn note_event_from_string(
    note_str: &str,
    default_instrument: Option<InstrumentId>,
) -> mlua::Result<NoteEvent> {
    if let Ok(note) = crate::midi::Note::try_from(note_str) {
        Ok(NoteEvent {
            note,
            volume: 1.0,
            instrument: default_instrument,
        })
    } else {
        Err(mlua::Error::FromLuaConversionError {
            from: "string",
            to: "Note",
            message: Some(format!("Invalid note value: '{}'", note_str)),
        })
    }
}

fn note_event_from_table(
    table: LuaTable,
    default_instrument: Option<InstrumentId>,
) -> mlua::Result<NoteEvent> {
    let volume = if let Ok(value) = table.get::<&str, f32>("volume") {
        value
    } else {
        1.0
    };
    // { key = 60, [volume = 1.0] }
    if let Ok(note_value) = table.get::<&str, u8>("key") {
        let note = crate::midi::Note::from(note_value);
        Ok(NoteEvent {
            note,
            volume,
            instrument: default_instrument,
        })
    }
    // { key = "C4", [volume = 1.0] }
    else if let Ok(note_str) = table.get::<&str, String>("key") {
        if let Ok(note) = crate::midi::Note::try_from(note_str.as_str()) {
            Ok(NoteEvent {
                note,
                volume,
                instrument: default_instrument,
            })
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
}

pub(crate) fn note_event_from_value(
    arg: LuaValue,
    arg_index: Option<usize>,
    default_instrument: Option<InstrumentId>,
) -> mlua::Result<Option<NoteEvent>> {
    match arg {
        LuaValue::Nil => Ok(None),
        LuaValue::Integer(note_value) => Ok(Some(note_event_from_number(
            note_value,
            default_instrument,
        )?)),
        LuaValue::String(note_str) => {
            if is_empty_note_string(&note_str.to_string_lossy()) {
                Ok(None)
            } else {
                Ok(Some(note_event_from_string(
                    &note_str.to_string_lossy(),
                    default_instrument,
                )?))
            }
        }
        LuaValue::Table(table) => Ok(Some(note_event_from_table(table, default_instrument)?)),
        _ => {
            return Err(mlua::Error::FromLuaConversionError {
                from: arg.type_name(),
                to: "Note",
                message: if let Some(index) = arg_index {
                    Some(
                        format!(
                            "Chord arg #{} does not contain a valid note property",
                            index
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

pub(crate) fn note_events_from_value(
    arg: LuaValue,
    arg_index: Option<usize>,
    default_instrument: Option<InstrumentId>,
) -> mlua::Result<Vec<Option<NoteEvent>>> {
    match arg {
        LuaValue::UserData(userdata) => {
            if userdata.is::<Sequence>() {
                Err(mlua::Error::FromLuaConversionError {
                    from: "Sequence",
                    to: "Note",
                    message: Some("Can not nest sequences into sequences".to_string()),
                })
            } else if let Ok(chord) = userdata.take::<Chord>() {
                Ok(chord.notes)
            } else {
                Err(mlua::Error::FromLuaConversionError {
                    from: "UserData",
                    to: "Note",
                    message: if let Some(index) = arg_index {
                        Some(
                            format!(
                                "Sequence arg #{} does not contain a valid note property",
                                index
                            )
                            .to_string(),
                        )
                    } else {
                        Some("Argument does not contain a valid note property".to_string())
                    },
                })
            }
        }
        _ => Ok(vec![note_event_from_value(
            arg,
            arg_index,
            default_instrument,
        )?]),
    }
}

fn rhytm_from_value(
    result: LuaValue,
) -> Result<Rc<RefCell<dyn Rhythm>>, Box<dyn std::error::Error>> {
    // hande script result
    if let Some(user_data) = result.as_userdata() {
        if let Ok(beat_time_rhythm) = user_data.take::<BeatTimeRhythm>() {
            Ok(Rc::new(RefCell::new(beat_time_rhythm)))
        } else if let Ok(second_time_rhythm) = user_data.take::<SecondTimeRhythm>() {
            Ok(Rc::new(RefCell::new(second_time_rhythm)))
        } else {
            Err(anyhow!("Expected script to return a Rhythm, got some other custom type",).into())
        }
    } else {
        Err(anyhow!(
            "Expected script to return a Rhythm, got {}",
            result.type_name()
        )
        .into())
    }
}

fn bad_argument_error(func: &str, arg: &str, pos: usize, message: &str) -> mlua::Error {
    mlua::Error::BadArgument {
        to: Some(func.to_string()),
        name: Some(arg.to_string()),
        pos,
        cause: mlua::Error::RuntimeError(message.to_string()).into(),
    }
}

// -------------------------------------------------------------------------------------------------

pub fn register_bindings(
    lua: &mut Lua,
    default_time_base: BeatTimeBase,
    default_instrument: Option<InstrumentId>,
) -> Result<(), Box<dyn std::error::Error>> {
    register_global_bindings(lua, default_time_base, default_instrument)?;
    register_pattern_bindings(lua)?;
    Ok(())
}

fn register_global_bindings(
    lua: &mut Lua,
    default_time_base: BeatTimeBase,
    default_instrument: Option<InstrumentId>,
) -> mlua::Result<()> {
    let globals = lua.globals();

    // function notes_in_scale(args...)
    globals.set(
        "notes_in_scale",
        lua.create_function(|lua, string: String| -> mlua::Result<LuaTable> {
            match scale::Scale::from_regex(string.as_str()) {
                Ok(scale) => {
                    let note_values = scale
                        .notes()
                        .into_iter()
                        .map(|n| LuaValue::Integer(Note::from(&n) as u8 as i64))
                        .enumerate();
                    Ok(lua.create_table_from(note_values)?)
                }
                Err(_) => Err(mlua::Error::BadArgument {
                    to: Some("Scale".to_string()),
                    pos: 1,
                    name: Some("scale".to_string()),
                    cause: Arc::new(mlua::Error::RuntimeError(format!(
                        "Invalid scale arg: '{}'. Valid scale args are e.g. 'c major'",
                        string,
                    ))),
                }),
            }
        })?,
    )?;

    // function euclidian(pulses, steps, offset or nil)
    globals.set(
        "euclidian",
        lua.create_function(
            |lua, (pulses, steps, offset): (i32, i32, Option<i32>)| -> mlua::Result<LuaTable> {
                let offset = offset.unwrap_or(0);
                if pulses <= 0 {
                    return Err(bad_argument_error(
                        "euclidian",
                        "pulses",
                        1,
                        "pulses must be > 0",
                    ));
                }
                if steps <= 0 {
                    return Err(bad_argument_error(
                        "euclidian",
                        "steps",
                        2,
                        "steps must be > 0",
                    ));
                }
                if pulses > steps {
                    return Err(bad_argument_error(
                        "euclidian",
                        "steps",
                        1,
                        "pulse must be <= step",
                    ));
                }
                lua.create_table_from(
                    euclidean(pulses as u32, steps as u32, offset)
                        .into_iter()
                        .map(|v| v as i32)
                        .enumerate(),
                )
            },
        )?,
    )?;

    // function chord(args...)
    globals.set(
        "chord",
        lua.create_function({
            let default_instrument = default_instrument;
            move |_lua, args: LuaMultiValue| -> mlua::Result<Chord> {
                let mut notes = vec![];
                for (index, arg) in args.into_iter().enumerate() {
                    notes.push(note_event_from_value(arg, Some(index), default_instrument)?);
                }
                Ok(Chord { notes })
            }
        })?,
    )?;

    // function sequence(args...)
    globals.set(
        "sequence",
        lua.create_function({
            let default_instrument = default_instrument;
            move |_lua, args: LuaMultiValue| -> mlua::Result<Sequence> {
                let mut notes = vec![];
                for (index, arg) in args.into_iter().enumerate() {
                    notes.push(note_events_from_value(
                        arg,
                        Some(index),
                        default_instrument,
                    )?);
                }
                Ok(Sequence { notes })
            }
        })?,
    )?;

    // function Emitter { args... }
    globals.set(
        "Emitter",
        lua.create_function({
            let default_time_base = default_time_base;
            move |_lua, table: LuaTable| -> mlua::Result<BeatTimeRhythm> {
                let resolution = table.get::<&str, f32>("resolution")?;
                let mut rhythm =
                    BeatTimeRhythm::new(default_time_base, BeatTimeStep::Beats(resolution));
                if table.contains_key("pattern")? {
                    let pattern = table.get::<&str, Vec<i32>>("pattern")?;
                    rhythm = rhythm.with_pattern_vector(pattern);
                }
                if table.contains_key("emit")? {
                    match table.get::<&str, LuaValue>("emit").unwrap() {
                        LuaValue::String(note_str) => {
                            let event = note_event_from_string(
                                &note_str.to_string_lossy(),
                                default_instrument,
                            )?;
                            rhythm = rhythm.trigger(event.to_event());
                        }
                        LuaValue::Table(table) => {
                            // { key = "C4", volume = 1.0 }
                            if table.contains_key("key")? {
                                let event = note_event_from_table(table, default_instrument)?;
                                rhythm = rhythm.trigger(event.to_event());
                            } else {
                                return Err(mlua::Error::FromLuaConversionError {
                                    from: "table",
                                    to: "Note",
                                    message: Some("Invalid event table argument".to_string()),
                                });
                            }
                        }
                        LuaValue::UserData(userdata) => {
                            if userdata.is::<Chord>() {
                                let chord = userdata.take::<Chord>().unwrap();
                                rhythm = rhythm.trigger(chord.notes.to_event());
                            } else if userdata.is::<Sequence>() {
                                let sequence = userdata.take::<Sequence>().unwrap();
                                rhythm = rhythm.trigger(sequence.notes.to_event_sequence());
                            } else {
                                return Err(mlua::Error::FromLuaConversionError {
                                    from: "table",
                                    to: "Note",
                                    message: Some("Invalid note table argument".to_string()),
                                });
                            }
                        }
                        LuaValue::Function(function) => {
                            rhythm = rhythm
                                .trigger(ScriptedEventIter::new(function, default_instrument)?);
                        }
                        _ => {
                            return Err(mlua::Error::FromLuaConversionError {
                                from: "table",
                                to: "Note",
                                message: Some("Invalid note table argument".to_string()),
                            });
                        }
                    }
                }
                Ok(rhythm)
            }
        })?,
    )?;

    Ok(())
}

fn register_pattern_bindings(lua: &mut Lua) -> mlua::Result<()> {
    // implemented in lua: load and evaluate chunk
    let chunk = lua.load(include_str!("./lua/pattern.lua"));
    chunk.exec()
}
