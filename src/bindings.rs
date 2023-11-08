//! Lua script bindings for the entire crate.

use std::{cell::RefCell, env, rc::Rc};

use anyhow::anyhow;
use mlua::{chunk, prelude::*};
use rust_music_theory::{note::Notes, scale};

use crate::{prelude::*, rhythm::euclidean::euclidean};

// ---------------------------------------------------------------------------------------------

pub(crate) mod unwrap;
use unwrap::*;

#[cfg(test)]
mod test;

// ---------------------------------------------------------------------------------------------

/// Create a new lua engine with preloaded packages and our default configuation
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
    rhythm_from_userdata(result)
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

// evaluate an expression which creates and returns a rhythm,
// returning a fallback rhythm on errors
pub fn new_rhythm_from_string_with_fallback(
    instrument: InstrumentId,
    time_base: BeatTimeBase,
    script: &str,
    script_name: &str,
) -> Rc<RefCell<dyn Rhythm>> {
    new_rhythm_from_string(instrument, time_base, script, script_name).unwrap_or_else(|err| {
        log::warn!(
            "Script '{}' failed to compile: {}",
            script_name,
            err
        );
        Rc::new(RefCell::new(BeatTimeRhythm::new(
            time_base,
            BeatTimeStep::Beats(1.0),
        )))
    })
}

// unwrap a BeatTimeRhythm or SecondTimeRhythm from the given LuaValue,
// which is expected to be a user data
fn rhythm_from_userdata(
    result: LuaValue,
) -> Result<Rc<RefCell<dyn Rhythm>>, Box<dyn std::error::Error>> {
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

// -------------------------------------------------------------------------------------------------

impl LuaUserData for BeatTimeRhythm {
    // BeatTimeRhythm is only passed through ATM
}

impl BeatTimeRhythm {
    // create a BeatTimeRhythm from the given Lua table value
    fn from_table(
        table: LuaTable,
        default_time_base: BeatTimeBase,
        default_instrument: Option<InstrumentId>,
    ) -> mlua::Result<BeatTimeRhythm> {
        let resolution = table.get::<&str, f32>("resolution").unwrap_or(1.0);
        if resolution <= 0.0 {
            return Err(bad_argument_error(
                "emit",
                "resolution",
                1,
                "resolution must be > 0",
            ));
        }
        let mut step = BeatTimeStep::Beats(resolution);
        if table.contains_key("unit")? {
            let unit = table.get::<&str, String>("unit")?;
            match unit.as_str() {
                "bars" => step = BeatTimeStep::Bar(resolution),
                "beats"|"quarter"|"1/4" => step = BeatTimeStep::Beats(resolution),
                "eighth"|"1/8" => step = BeatTimeStep::Eighth(resolution),
                "sixteenth"|"1/16" => step = BeatTimeStep::Sixteenth(resolution),
                _ => return Err(bad_argument_error("emit", "unit", 1, 
                "Invalid unit parameter. Expected 'seconds', 'bars', 'beats', 'eighth', 'sixteenth'"))
            }
        }
        let mut rhythm = BeatTimeRhythm::new(default_time_base, step);
        if table.contains_key("offset")? {
            let offset = table.get::<&str, f32>("offset")?;
            rhythm = rhythm.with_offset_in_step(offset);
        }
        if table.contains_key("pattern")? {
            let pattern = table.get::<&str, Vec<i32>>("pattern")?;
            rhythm = rhythm.with_pattern_vector(pattern);
        }
        if table.contains_key("emit")? {
            let iter =
                event_iter_from_value(table.get::<&str, LuaValue>("emit")?, default_instrument)?;
            rhythm = rhythm.trigger_dyn(iter);
        }
        Ok(rhythm)
    }
}

impl LuaUserData for SecondTimeRhythm {
    // SecondTimeRhythm is only passed through ATM
}

impl SecondTimeRhythm {
    // create a SecondtimeRhythm from the given Lua table value
    fn from_table(
        table: LuaTable,
        default_time_base: SecondTimeBase,
        default_instrument: Option<InstrumentId>,
    ) -> mlua::Result<SecondTimeRhythm> {
        let mut resolution = table.get::<&str, f64>("resolution").unwrap_or(1.0);
        if resolution <= 0.0 {
            return Err(bad_argument_error(
                "emit",
                "resolution",
                1,
                "resolution must be > 0",
            )); 
        }
        if table.contains_key("unit")? {
            let unit = table.get::<&str, String>("unit")?;
            match unit.as_str() {
                "seconds" => (),
                "ms" => resolution /= 1000.0,
                _ => return Err(bad_argument_error("emit", "unit", 1, 
                "Invalid unit parameter. Expected 'seconds', 'bars', 'beats', 'eighth', 'sixteenth'")),
            }
        }
        let mut rhythm = SecondTimeRhythm::new(default_time_base, resolution);

        if table.contains_key("offset")? {
            let offset = table.get::<&str, f32>("offset")? as SecondTimeStep;
            rhythm = rhythm.with_offset(offset);
        }
        if table.contains_key("pattern")? {
            let pattern = table.get::<&str, Vec<i32>>("pattern")?;
            rhythm = rhythm.with_pattern_vector(pattern);
        }
        if table.contains_key("emit")? {
            let iter =
                event_iter_from_value(table.get::<&str, LuaValue>("emit")?, default_instrument)?;
            rhythm = rhythm.trigger_dyn(iter);
        }
        Ok(rhythm)
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
    register_fun_bindings(lua)?;
    Ok(())
}

fn register_global_bindings(
    lua: &mut Lua,
    default_time_base: BeatTimeBase,
    default_instrument: Option<InstrumentId>,
) -> mlua::Result<()> {
    let globals = lua.globals();

    // function notes_in_scale(expression)
    globals.set(
        "notes_in_scale",
        lua.create_function(|lua, string: String| -> mlua::Result<LuaTable> {
            match scale::Scale::from_regex(string.as_str()) {
                Ok(scale) => Ok(lua.create_sequence_from(
                    scale
                        .notes()
                        .iter()
                        .map(|n| LuaValue::Integer(Note::from(n) as u8 as i64)),
                )?),
                Err(err) => Err(bad_argument_error(
                    "notes_in_scale",
                    "scale",
                    1,
                    format!("{}. Valid modes are e.g. 'c major'", err).as_str(),
                )),
            }
        })?,
    )?;

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
    let chunk = lua.load(include_str!("./bindings/lua/pattern.lua"));
    chunk.exec()
}

fn register_fun_bindings(lua: &mut Lua) -> mlua::Result<()> {
    // implemented in lua: load and evaluate chunk
    let chunk = lua.load(include_str!("./bindings/lua/fun.lua"));
    chunk.exec()
}
