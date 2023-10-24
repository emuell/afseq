use mlua::prelude::*;

use crate::prelude::*;

// ---------------------------------------------------------------------------------------------

// Chord Userdata in bindings
#[derive(Clone, Debug)]
pub struct ChordUserData {
    pub notes: Vec<Option<NoteEvent>>,
}

impl ChordUserData {
    pub fn from(
        args: LuaMultiValue,
        default_instrument: Option<InstrumentId>,
    ) -> mlua::Result<Self> {
        let mut notes = vec![];
        for (index, arg) in args.into_iter().enumerate() {
            notes.push(note_event_from_value(arg, Some(index), default_instrument)?);
        }
        Ok(ChordUserData { notes })
    }
}

impl LuaUserData for ChordUserData {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("set_volume", |_lua, (ud, volume): (LuaAnyUserData, f32)| {
            let mut this = ud.borrow::<Self>()?.clone();
            for note in this.notes.iter_mut().flatten() {
                note.volume = volume;
            }
            Ok(this)
        });

        methods.add_function("amplify", |_lua, (ud, volume): (LuaAnyUserData, f32)| {
            let mut this = ud.borrow::<Self>()?.clone();
            for note in this.notes.iter_mut().flatten() {
                note.volume *= volume;
            }
            Ok(this)
        });
    }
}

// Sequence
#[derive(Clone, Debug)]
pub struct SequenceUserData {
    pub notes: Vec<Vec<Option<NoteEvent>>>,
}

impl SequenceUserData {
    pub fn from(args: LuaMultiValue, default_instrument: Option<InstrumentId>) -> mlua::Result<Self> {
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

impl LuaUserData for SequenceUserData {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("set_volume", |_lua, (ud, volume): (LuaAnyUserData, f32)| {
            let mut this = ud.borrow::<Self>()?.clone();
            for note in this.notes.iter_mut().flatten().flatten() {
                note.volume = volume;
            }
            Ok(this)
        });

        methods.add_function("amplify", |_lua, (ud, volume): (LuaAnyUserData, f32)| {
            let mut this = ud.borrow::<Self>()?.clone();
            for note in this.notes.iter_mut().flatten().flatten() {
                note.volume *= volume;
            }
            Ok(this)
        });
    }
}

impl LuaUserData for BeatTimeRhythm {
    // BeatTimeRhythm is only passed through ATM
}

// ---------------------------------------------------------------------------------------------

pub fn bad_argument_error(func: &str, arg: &str, pos: usize, message: &str) -> mlua::Error {
    mlua::Error::BadArgument {
        to: Some(func.to_string()),
        name: Some(arg.to_string()),
        pos,
        cause: mlua::Error::RuntimeError(message.to_string()).into(),
    }
}

// ---------------------------------------------------------------------------------------------

fn is_empty_note_string(s: &str) -> bool {
    matches!(s, "" | "-" | "--" | "---" | "." | ".." | "...")
}

pub fn note_event_from_number(
    note_value: i64,
    default_instrument: Option<InstrumentId>,
) -> mlua::Result<NoteEvent> {
    Ok(NoteEvent {
        note: crate::midi::Note::from(note_value as u8),
        volume: 1.0,
        instrument: default_instrument,
    })
}

pub fn note_event_from_string(
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

pub fn note_event_from_table(
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

pub fn note_event_from_value(
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
            } else if let Ok(chord) = userdata.take::<ChordUserData>() {
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
