//! Lua script bindings, to create rhythms dynamically.

use std::{cell::RefCell, env, rc::Rc};

use lazy_static::lazy_static;
use mlua::{chunk, prelude::*};

// ---------------------------------------------------------------------------------------------

mod rhythm;
use rhythm::rhythm_from_userdata;

mod scale;

mod note;
use note::NoteUserData;

mod sequence;
use sequence::SequenceUserData;

mod unwrap;
use unwrap::*;

// ---------------------------------------------------------------------------------------------

use crate::{
    event::{InstrumentId, NoteEvent},
    rhythm::{
        beat_time::BeatTimeRhythm, euclidean::euclidean, second_time::SecondTimeRhythm, Rhythm,
    },
    time::{BeatTimeBase, BeatTimeStep},
    Scale, SecondTimeBase,
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
) -> LuaResult<Vec<Option<NoteEvent>>> {
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
    register_table_bindings(lua)?;
    register_pattern_module(lua)?;
    register_fun_module(lua)?;
    Ok(())
}

fn register_global_bindings(
    lua: &mut Lua,
    default_time_base: BeatTimeBase,
    default_instrument: Option<InstrumentId>,
) -> LuaResult<()> {
    let globals = lua.globals();

    // function euclidean(pulses, steps, [offset])
    globals.set(
        "euclidean",
        lua.create_function(
            |lua, (pulses, steps, offset): (i32, i32, Option<i32>)| -> LuaResult<LuaTable> {
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
            |lua, (note, mode_or_intervals): (LuaValue, LuaValue)| -> LuaResult<Scale> {
                let note = FromLua::from_lua(note, lua)?;
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
                        .collect::<LuaResult<Vec<usize>>>()?;
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
            move |_lua, args: LuaMultiValue| -> LuaResult<NoteUserData> {
                NoteUserData::from(args, default_instrument)
            }
        })?,
    )?;

    // function sequence(args...)
    globals.set(
        "sequence",
        lua.create_function({
            let default_instrument = default_instrument;
            move |_lua, args: LuaMultiValue| -> LuaResult<SequenceUserData> {
                SequenceUserData::from(args, default_instrument)
            }
        })?,
    )?;

    // function Emitter { args... }
    globals.set(
        "Emitter",
        lua.create_function({
            let default_time_base = default_time_base;
            move |lua, table: LuaTable| -> LuaResult<LuaValue> {
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

fn register_table_bindings(lua: &mut Lua) -> LuaResult<()> {
    // implemented in lua: load and evaluate chunk
    let chunk = lua
        .load(include_str!("./bindings/lua/table.lua"))
        .set_name("[inbuilt:table.lua]");
    chunk.exec()
}

fn register_pattern_module(lua: &mut Lua) -> LuaResult<()> {
    // cache module bytecode to speed up requires
    lazy_static! {
        static ref FUN_BYTECODE: LuaResult<Vec<u8>> = {
            let strip = true;
            Lua::new()
                .load(include_str!("./bindings/lua/pattern.lua"))
                .into_function()
                .map(|x| x.dump(strip))
        };
    }
    // see https://github.com/khvzak/mlua/discussions/322
    let package: LuaTable = lua.globals().get("package")?;
    let loaders: LuaTable = package.get("searchers")?; // NB: "loaders" in lua 5.1
    loaders.push(LuaFunction::wrap(|lua, path: String| {
        if path == "pattern" {
            LuaFunction::wrap(|lua, ()| match FUN_BYTECODE.clone() {
                Ok(bytecode) => lua
                    .load(bytecode)
                    .set_name("[inbuilt:pattern.lua]")
                    .set_mode(mlua::ChunkMode::Binary)
                    .call::<_, LuaValue>(()),
                Err(err) => err.into_lua(lua),
            })
            .into_lua(lua)
        } else {
            "not found".into_lua(lua)
        }
    }))
}

fn register_fun_module(lua: &mut Lua) -> LuaResult<()> {
    // cache module bytecode to speed up requires
    lazy_static! {
        static ref FUN_BYTECODE: LuaResult<Vec<u8>> = {
            let strip = true;
            Lua::new()
                .load(include_str!("./bindings/lua/fun.lua"))
                .into_function()
                .map(|x| x.dump(strip))
        };
    }
    // see https://github.com/khvzak/mlua/discussions/322
    let package: LuaTable = lua.globals().get("package")?;
    let loaders: LuaTable = package.get("searchers")?; // NB: "loaders" in lua 5.1
    loaders.push(LuaFunction::wrap(|lua, path: String| {
        if path == "fun" {
            LuaFunction::wrap(|lua, ()| match FUN_BYTECODE.clone() {
                Ok(bytecode) => lua
                    .load(bytecode)
                    .set_name("[inbuilt:fun.lua]")
                    .set_mode(mlua::ChunkMode::Binary)
                    .call::<_, LuaValue>(()),
                Err(err) => err.into_lua(lua),
            })
            .into_lua(lua)
        } else {
            "not found".into_lua(lua)
        }
    }))
}

// --------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn extensions() {
        // create a new engine and register bindings
        let mut engine = new_engine();
        register_bindings(
            &mut engine,
            BeatTimeBase {
                beats_per_min: 160.0,
                beats_per_bar: 6,
                samples_per_sec: 96000,
            },
            Some(InstrumentId::from(76)),
        )
        .unwrap();

        // table.lua is present
        assert!(engine
            .load(r#"return table.new()"#)
            .eval::<LuaTable>()
            .is_ok());

        // pattern.lua is present, but only when required
        assert!(engine
            .load(r#"return pattern.new()"#)
            .eval::<LuaTable>()
            .is_err());
        assert!(engine
            .load(
                r#"
                local pattern = require "pattern"
                return pattern.new()
                "#
            )
            .eval::<LuaTable>()
            .is_ok());

        // fun.lua is present, but only when required
        assert!(engine
            .load(r#"return fun.iter {1,2,3}:map(function(v) return v*2 end):totable()"#)
            .eval::<LuaTable>()
            .is_err());
        assert!(engine
            .load(
                r#"
                local fun = require "fun"
                return fun.iter {1,2,3}:map(function(v) return v*2 end):totable()
                "#
            )
            .eval::<LuaTable>()
            .is_ok());
    }
}
