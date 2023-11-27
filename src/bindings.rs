//! Lua script bindings for the entire crate.

use std::{cell::RefCell, env, rc::Rc};

use mlua::{chunk, prelude::*};

use crate::prelude::*;

// ---------------------------------------------------------------------------------------------

mod note;
mod rhythm;
mod scale;
mod sequence;
mod unwrap;

#[cfg(test)]
mod test;

use self::{
    note::NoteUserData, rhythm::rhythm_from_userdata, sequence::SequenceUserData, unwrap::*,
};

// ---------------------------------------------------------------------------------------------

/// Create a new raw lua engine with preloaded packages, but no bindings.
/// See also `register_bindings`
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

/// Evaluate a lua script file which creates and returns a rhythm.
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
    rhythm_from_userdata(result)
}

/// Evaluate a lua script file which creates and returns a rhythm,
/// returning a fallback rhythm on errors
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

/// Evaluate a Lua expression which creates and returns a rhythm.
pub fn new_rhythm_from_string(
    instrument: InstrumentId,
    time_base: BeatTimeBase,
    script: &str,
    script_name: &str,
) -> Result<Rc<RefCell<dyn Rhythm>>, Box<dyn std::error::Error>> {
    // create a new engine
    let mut lua = new_engine();
    register_bindings(&mut lua, time_base, Some(instrument))?;
    // compile and evaluate script
    let chunk = lua.load(script).set_name(script_name);
    let result = chunk.eval::<LuaValue>()?;
    // convert result
    rhythm_from_userdata(result)
}

/// Evaluate a Lua expression which creates and returns a rhythm,
/// returning a fallback rhythm on errors.
pub fn new_rhythm_from_string_with_fallback(
    instrument: InstrumentId,
    time_base: BeatTimeBase,
    script: &str,
    script_name: &str,
) -> Rc<RefCell<dyn Rhythm>> {
    new_rhythm_from_string(instrument, time_base, script, script_name).unwrap_or_else(|err| {
        log::warn!("Script '{}' failed to compile: {}", script_name, err);
        Rc::new(RefCell::new(BeatTimeRhythm::new(
            time_base,
            BeatTimeStep::Beats(1.0),
        )))
    })
}

// -------------------------------------------------------------------------------------------------

/// Try converting the given lua value to a note events vector.
pub fn new_note_events_from_lua(
    arg: LuaValue,
    arg_index: Option<usize>,
    default_instrument: Option<InstrumentId>,
) -> mlua::Result<Vec<Option<NoteEvent>>> {
    unwrap::note_events_from_value(arg, arg_index, default_instrument)
}

// -------------------------------------------------------------------------------------------------

/// Register afseq bindings with the given lua engine.
pub fn register_bindings(
    lua: &mut Lua,
    default_time_base: BeatTimeBase,
    default_instrument: Option<InstrumentId>,
) -> Result<(), Box<dyn std::error::Error>> {
    register_global_bindings(lua, default_time_base, default_instrument)?;
    register_pattern_bindings(lua)?;
    register_fun_bindings(lua)?;
    Ok(())
}

fn register_global_bindings(
    lua: &mut Lua,
    default_time_base: BeatTimeBase,
    default_instrument: Option<InstrumentId>,
) -> mlua::Result<()> {
    let globals = lua.globals();

    // function euclidean(pulses, steps, [offset])
    globals.set(
        "euclidean",
        lua.create_function(
            |lua, (pulses, steps, offset): (i32, i32, Option<i32>)| -> mlua::Result<LuaTable> {
                let offset = offset.unwrap_or(0);
                if pulses <= 0 {
                    return Err(bad_argument_error(
                        "euclidean",
                        "pulses",
                        1,
                        "pulses must be > 0",
                    ));
                }
                if steps <= 0 {
                    return Err(bad_argument_error(
                        "euclidean",
                        "steps",
                        2,
                        "steps must be > 0",
                    ));
                }
                if pulses > steps {
                    return Err(bad_argument_error(
                        "euclidean",
                        "steps",
                        1,
                        "pulse must be <= step",
                    ));
                }
                lua.create_sequence_from(
                    euclidean(pulses as u32, steps as u32, offset)
                        .iter()
                        .map(|v| *v as i32),
                )
            },
        )?,
    )?;

    // function scale(note, mode|intervals)
    globals.set(
        "scale",
        lua.create_function(
            |_lua, (note, mode_or_intervals): (LuaValue, LuaValue)| -> mlua::Result<Scale> {
                let note = note_from_value(note, Some(0))?;
                if let Some(mode) = mode_or_intervals.as_str() {
                    match Scale::try_from((note, mode)) {
                        Ok(scale) => Ok(scale),
                        Err(err) => Err(bad_argument_error(
                            "scale",
                            "mode",
                            1,
                            format!(
                                "{}. Valid modes are: {}",
                                err,
                                Scale::mode_names().join(", ")
                            )
                            .as_str(),
                        )),
                    }
                } else if let Some(table) = mode_or_intervals.as_table() {
                    let intervals = table
                        .clone()
                        .sequence_values::<usize>()
                        .enumerate()
                        .map(|(_, result)| result)
                        .collect::<mlua::Result<Vec<usize>>>()?;
                    Ok(Scale::try_from((note, &intervals)).map_err(|err| {
                        bad_argument_error("scale", "intervals", 1, err.to_string().as_str())
                    })?)
                } else {
                    Err(bad_argument_error(
                        "scale",
                        "mode|interval",
                        1,
                        "Expecting either a mode string or interval table",
                    ))
                }
            },
        )?,
    )?;

    // function note(args...)
    globals.set(
        "note",
        lua.create_function({
            let default_instrument = default_instrument;
            move |_lua, args: LuaMultiValue| -> mlua::Result<NoteUserData> {
                NoteUserData::from(args, default_instrument)
            }
        })?,
    )?;

    // function sequence(args...)
    globals.set(
        "sequence",
        lua.create_function({
            let default_instrument = default_instrument;
            move |_lua, args: LuaMultiValue| -> mlua::Result<SequenceUserData> {
                SequenceUserData::from(args, default_instrument)
            }
        })?,
    )?;

    // function Emitter { args... }
    globals.set(
        "Emitter",
        lua.create_function({
            let default_time_base = default_time_base;
            move |lua, table: LuaTable| -> mlua::Result<LuaValue> {
                let second_time_unit = match table.get::<&str, String>("unit") {
                    Ok(unit) => matches!(unit.as_str(), "seconds" | "ms"),
                    Err(_) => false,
                };
                if second_time_unit {
                    let time_base = SecondTimeBase {
                        samples_per_sec: default_time_base.samples_per_sec,
                    };
                    let instrument = default_instrument;
                    SecondTimeRhythm::from_table(table, time_base, instrument)?.into_lua(lua)
                } else {
                    let time_base = default_time_base;
                    let instrument = default_instrument;
                    BeatTimeRhythm::from_table(table, time_base, instrument)?.into_lua(lua)
                }
            }
        })?,
    )?;

    Ok(())
}

fn register_pattern_bindings(lua: &mut Lua) -> mlua::Result<()> {
    // implemented in lua: load and evaluate chunk
    let chunk = lua
        .load(include_str!("./bindings/lua/pattern.lua"))
        .set_name("[inbuilt:pattern.lua]");
    chunk.exec()
}

fn register_fun_bindings(lua: &mut Lua) -> mlua::Result<()> {
    // implemented in lua: load and evaluate chunk
    let chunk = lua
        .load(include_str!("./bindings/lua/fun.lua"))
        .set_name("[inbuilt:fun.lua]");
    chunk.exec()
}
