//! Lua script bindings for the entire crate.

use std::{cell::RefCell, rc::Rc, sync::Arc};

use mlua::prelude::*;
use rust_music_theory::{note::Notes, scale};

use crate::prelude::*;

// ---------------------------------------------------------------------------------------------

/// Create a new rhai engine with preloaded packages and our default configuation
pub fn new_engine() -> Lua {
    Lua::new()
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
    to_rhytm(result)
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
    to_rhytm(result)
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

fn to_rhytm(result: LuaValue) -> Result<Rc<RefCell<dyn Rhythm>>, Box<dyn std::error::Error>> {
    // hande script result
    if let Some(user_data) = result.as_userdata() {
        if let Ok(beat_time_rhythm) = user_data.take::<BeatTimeRhythm>() {
            Ok(Rc::new(RefCell::new(beat_time_rhythm)))
        } else if let Ok(second_time_rhythm) = user_data.take::<SecondTimeRhythm>() {
            Ok(Rc::new(RefCell::new(second_time_rhythm)))
        } else {
            Err(string_error::new_err(
                "Expected script to return a Rhythm, got some other custom type",
            ))
        }
    } else {
        Err(string_error::new_err(&format!(
            "Expected script to return a Rhythm, got {}",
            result.type_name()
        )))
    }
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
    let volume = if let Ok(value) = table.get::<_, f32>("volume") {
        value
    } else {
        1.0
    };
    // { key = 60, [volume = 1.0] }
    if let Ok(note_value) = table.get::<_, u8>("key") {
        let note = crate::midi::Note::from(note_value);
        Ok(NoteEvent {
            note,
            volume,
            instrument: default_instrument,
        })
    }
    // { key = "C4", [volume = 1.0] }
    else if let Ok(note_str) = table.get::<_, String>("key") {
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
            message: Some("Table does not contain a valid 'note' property".to_string()),
        })
    }
}

fn notes_in_scale(lua: &Lua, string: String) -> mlua::Result<LuaTable> {
    match scale::Scale::from_regex(string.as_str()) {
        Ok(scale) => {
            let note_values = scale
                .notes()
                .into_iter()
                .map(|n| LuaValue::Integer(Note::from(&n) as u8 as i64)).enumerate();
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
}

// -------------------------------------------------------------------------------------------------

pub fn register_bindings(
    lua: &mut Lua,
    default_time_base: BeatTimeBase,
    default_instrument: Option<InstrumentId>,
) -> Result<(), Box<dyn std::error::Error>> {
    let globals = lua.globals();

    // Chord
    #[derive(Clone, Debug, FromLua)]
    struct Chord {
        notes: Vec<Option<NoteEvent>>,
    }
    impl LuaUserData for Chord {}

    // Sequence
    #[derive(Clone, Debug, FromLua)]
    struct Sequence {
        notes: Vec<Vec<Option<NoteEvent>>>,
    }
    impl LuaUserData for Sequence {}

    // function notes_in_scale(args...)
    globals.set(
        "notes_in_scale",
        lua.create_function(|lua, string: String| -> mlua::Result<LuaTable> {
            notes_in_scale(lua, string)
        })?,
    )?;

    // function chord(args...)
    globals.set(
        "chord",
        lua.create_function({
            let default_instrument = default_instrument;
            move |_, args: LuaMultiValue| -> mlua::Result<Chord> {
                let mut notes = vec![];
                for (index, arg) in args.into_iter().enumerate() {
                    match arg {
                        LuaValue::Nil => {
                            notes.push(None);
                        }
                        LuaValue::Integer(note_value) => {
                            notes.push(Some(note_event_from_number(
                                note_value,
                                default_instrument,
                            )?));
                        }
                        LuaValue::String(note_str) => {
                            if is_empty_note_string(&note_str.to_string_lossy()) {
                                notes.push(None);
                            } else {
                                notes.push(Some(note_event_from_string(
                                    &note_str.to_string_lossy(),
                                    default_instrument,
                                )?));
                            }
                        }
                        LuaValue::Table(table) => {
                            notes.push(Some(note_event_from_table(table, default_instrument)?));
                        }
                        _ => {
                            return Err(mlua::Error::FromLuaConversionError {
                                from: arg.type_name(),
                                to: "Note",
                                message: Some(
                                    format!(
                                        "Chord arg #{} does not contain a valid note property",
                                        index
                                    )
                                    .to_string(),
                                ),
                            });
                        }
                    }
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
            move |_, args: LuaMultiValue| -> mlua::Result<Sequence> {
                let mut notes = vec![];
                for (index, arg) in args.into_iter().enumerate() {
                    match arg {
                        LuaValue::Nil => {
                            notes.push(vec![]);
                        }
                        LuaValue::Integer(note_value) => {
                            notes.push(vec![Some(note_event_from_number(
                                note_value,
                                default_instrument,
                            )?)]);
                        }
                        LuaValue::String(note_str) => {
                            if is_empty_note_string(&note_str.to_string_lossy()) {
                                notes.push(vec![]);
                            } else {
                                notes.push(vec![Some(note_event_from_string(
                                    &note_str.to_string_lossy(),
                                    default_instrument,
                                )?)]);
                            }
                        }
                        LuaValue::Table(table) => {
                            notes.push(vec![Some(note_event_from_table(
                                table,
                                default_instrument,
                            )?)]);
                        }
                        LuaValue::UserData(userdata) => {
                            if let Ok(chord) = userdata.take::<Chord>() {
                                notes.push(chord.notes);
                            } else {
                                return Err(mlua::Error::FromLuaConversionError {
                                    from: "UserData",
                                    to: "Note",
                                    message: Some(format!(
                                        "Sequence arg #{} does not contain a valid note property", index)
                                            .to_string(),
                                    ),
                                });
                            }
                        }
                        _ => {
                            return Err(mlua::Error::FromLuaConversionError {
                                from: arg.type_name(),
                                to: "Note",
                                message: Some(format!(
                                    "Sequence arg #{} does not contain a valid note property", index)
                                        .to_string(),
                                ),
                            });
                        }
                    }
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
            move |_, table: LuaTable| -> mlua::Result<BeatTimeRhythm> {
                let resolution = table.get::<_, f32>("resolution")?;
                let mut rhythm =
                    BeatTimeRhythm::new(default_time_base, BeatTimeStep::Beats(resolution));
                if table.contains_key("pattern")? {
                    let pattern = table.get::<_, Vec<i32>>("pattern")?;
                    rhythm = rhythm.with_pattern_vector(pattern);
                }
                if table.contains_key("emit")? {
                    match table.get::<_, LuaValue>("emit").unwrap() {
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
                        LuaValue::Function(_fun) => {
                            return Err(mlua::Error::FromLuaConversionError {
                                from: "function",
                                to: "Note",
                                message: Some("Code missing".to_string()),
                            });
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

// ---------------------------------------------------------------------------------------------

impl LuaUserData for BeatTimeRhythm {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("resolution", |_, _rhythm, _value: f32| {
            // TODO: rhythm.step(value);
            Ok(())
        });
    }
}
