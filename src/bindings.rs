//! Lua bindings for the entire crate.

use std::{cell::RefCell, env, rc::Rc};

use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

use lazy_static::lazy_static;

use mlua::chunk;
use mlua::prelude::*;

use self::{
    cycle::CycleUserData,
    note::NoteUserData,
    rhythm::rhythm_from_userdata,
    sequence::SequenceUserData,
    unwrap::{bad_argument_error, validate_table_properties},
};

use crate::{
    event::InstrumentId,
    rhythm::{beat_time::BeatTimeRhythm, second_time::SecondTimeRhythm, Rhythm},
    time::BeatTimeBase,
    Scale,
};

// ---------------------------------------------------------------------------------------------

// private binding impls
mod callback;
mod cycle;
mod note;
mod rhythm;
mod scale;
mod sequence;
mod timeout;
mod unwrap;

// public re-exports
pub use callback::{clear_lua_callback_errors, has_lua_callback_errors, lua_callback_errors};

// internal re-exports
pub(crate) use callback::LuaCallback;
pub(crate) use timeout::LuaTimeoutHook;
pub(crate) use unwrap::{note_events_from_value, pattern_pulse_from_value};

// ---------------------------------------------------------------------------------------------

/// Global sharedLua data, unique to each new Lua instance.
#[derive(Debug, Clone)]
pub(crate) struct LuaAppData {
    /// Global random seed, set by math.randomseed() for each Lua instance and passed to
    /// newly created rhythm impls.
    pub(crate) rand_seed: Option<[u8; 32]>,
    /// Global random number generator, used for our math.random() impl.
    pub(crate) rand_rgn: Xoshiro256PlusPlus,
}

impl LuaAppData {
    fn new() -> Self {
        let rand_seed = None;
        let rand_rgn = Xoshiro256PlusPlus::from_seed(rand::thread_rng().gen());
        Self {
            rand_seed,
            rand_rgn,
        }
    }
}

// ---------------------------------------------------------------------------------------------

/// Create a new raw lua engine with preloaded packages, but no bindings. Also returns a timeout
/// hook instance to limit duration of script calls.
/// Use [`register_bindings`] to register the bindings for a newly created engine.
pub(crate) fn new_engine() -> LuaResult<(Lua, LuaTimeoutHook)> {
    // create a new lua instance with the allowed std libraries
    let lua = Lua::new_with(
        LuaStdLib::STRING | LuaStdLib::TABLE | LuaStdLib::MATH | LuaStdLib::PACKAGE,
        LuaOptions::default(),
    )
    .expect("Failed to create a new lua engine");
    // add cwd/lib to package path
    let cwd = env::current_dir()
        .unwrap_or(".".into())
        .to_string_lossy()
        .to_string();
    lua.load(chunk!(package.path = $cwd.."/assets/lib/?.lua;"..package.path))
        .exec()?;
    // install a timeout hook
    let timeout_hook = LuaTimeoutHook::new(&lua);
    // create new app data
    lua.set_app_data(LuaAppData::new());
    // return the lua instance and timeout manager
    Ok((lua, timeout_hook))
}

// -------------------------------------------------------------------------------------------------

/// Evaluate a lua script file which creates and returns a rhythm.
///
/// ### Errors
/// Will return `Err` if `file_name` does not exist, failed to load or the lua file at the given
/// path fails to evaulate to a valid rhythm.
pub fn new_rhythm_from_file(
    time_base: BeatTimeBase,
    instrument: Option<InstrumentId>,
    file_name: &str,
) -> Result<Rc<RefCell<dyn Rhythm>>, Box<dyn std::error::Error>> {
    // create a new engine and register bindings
    let (mut lua, mut timeout_hook) =
        new_engine().map_err(Into::<Box<dyn std::error::Error>>::into)?;
    register_bindings(&mut lua, &timeout_hook, &time_base)?;
    // restart the timeout hook
    timeout_hook.reset();
    // compile and evaluate script
    let chunk = lua.load(std::path::PathBuf::from(file_name));
    let result = chunk.eval::<LuaValue>()?;
    // convert result
    rhythm_from_userdata(&result, instrument).map_err(Into::into)
}

/// Evaluate a Lua string expression which creates and returns a rhythm.
///
/// ### Errors
/// Will return `Err` if the lua string contents fail to evaluate to a valid rhythm.
pub fn new_rhythm_from_string(
    time_base: BeatTimeBase,
    instrument: Option<InstrumentId>,
    script: &str,
    script_name: &str,
) -> Result<Rc<RefCell<dyn Rhythm>>, Box<dyn std::error::Error>> {
    // create a new engine and register bindings
    let (mut lua, mut timeout_hook) =
        new_engine().map_err(Into::<Box<dyn std::error::Error>>::into)?;
    register_bindings(&mut lua, &timeout_hook, &time_base)?;
    // restart the timeout hook
    timeout_hook.reset();
    // compile and evaluate script
    let chunk = lua.load(script).set_name(script_name);
    let result = chunk.eval::<LuaValue>()?;
    // convert result
    rhythm_from_userdata(&result, instrument).map_err(Into::into)
}

// -------------------------------------------------------------------------------------------------

/// Register afseq bindings with the given lua engine.
/// Engine instance is expected to be one created via [`new_engine`].
pub(crate) fn register_bindings(
    lua: &mut Lua,
    timeout_hook: &LuaTimeoutHook,
    time_base: &BeatTimeBase,
) -> LuaResult<()> {
    register_global_bindings(lua, timeout_hook, time_base)?;
    register_math_bindings(lua)?;
    register_table_bindings(lua)?;
    register_pattern_module(lua)?;
    Ok(())
}

fn register_global_bindings(
    lua: &mut Lua,
    timeout_hook: &LuaTimeoutHook,
    time_base: &BeatTimeBase,
) -> LuaResult<()> {
    let globals = lua.globals();

    // function scale(note, mode|intervals)
    globals.raw_set(
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
                                "{}, valid modes are: {}",
                                err,
                                Scale::mode_names().join(", ")
                            )
                            .as_str(),
                        )),
                    }
                } else if let Some(table) = mode_or_intervals.as_table() {
                    let intervals = table
                        .clone()
                        .sequence_values::<i32>()
                        .collect::<LuaResult<Vec<i32>>>()
                        .map_err(|err| {
                            bad_argument_error(
                                "scale",
                                "interval",
                                2,
                                &format!("invalid interval values: {}", err),
                            )
                        })?;
                    Ok(Scale::try_from((note, &intervals)).map_err(|err| {
                        bad_argument_error("scale", "intervals", 1, &err.to_string())
                    })?)
                } else {
                    Err(bad_argument_error(
                        "scale",
                        "mode|interval",
                        2,
                        "expecting a mode string or an interval array as second argument",
                    ))
                }
            },
        )?,
    )?;

    // function note(args...)
    globals.raw_set(
        "note",
        lua.create_function(|_lua, args: LuaMultiValue| -> LuaResult<NoteUserData> {
            NoteUserData::from(args)
        })?,
    )?;

    // function chord(note, mode)
    globals.raw_set(
        "chord",
        lua.create_function(
            |_lua, (note, mode_or_intervals): (LuaValue, LuaValue)| -> LuaResult<NoteUserData> {
                NoteUserData::from_chord(&note, &mode_or_intervals)
            },
        )?,
    )?;

    // function sequence(args...)
    globals.raw_set(
        "sequence",
        lua.create_function(|_lua, args: LuaMultiValue| -> LuaResult<SequenceUserData> {
            SequenceUserData::from(args)
        })?,
    )?;

    // function cycle(input)
    globals.raw_set(
        "cycle",
        lua.create_function(|lua, arg: LuaString| -> LuaResult<CycleUserData> {
            // NB: don't keep borrowing app_data_ref here
            let rand_seed = {
                lua.app_data_ref::<LuaAppData>()
                    .expect("Failed to access Lua app data")
                    .rand_seed
            };
            CycleUserData::from(arg, rand_seed)
        })?,
    )?;

    // function rhythm { args... }
    globals.raw_set(
        "rhythm",
        lua.create_function({
            let timeout_hook = timeout_hook.clone();
            let time_base = *time_base;
            move |lua, table: LuaTable| -> LuaResult<LuaValue> {
                // error on unknown option keys
                let rhythm_properties =
                    ["unit", "resolution", "offset", "pattern", "repeats", "emit"];
                validate_table_properties(&table, &rhythm_properties)?;
                // check which time unit is specified
                let second_time_unit = match table.get::<&str, String>("unit") {
                    Ok(unit) => matches!(unit.as_str(), "seconds" | "ms"),
                    Err(_) => false,
                };
                // NB: don't keep borrowing app_data_ref here: Rhytm constructos may use random functions
                let rand_seed = {
                    lua.app_data_ref::<LuaAppData>()
                        .expect("Failed to access Lua app data")
                        .rand_seed
                };
                if second_time_unit {
                    SecondTimeRhythm::from_table(lua, &timeout_hook, &time_base, &table, rand_seed)?
                        .into_lua(lua)
                } else {
                    BeatTimeRhythm::from_table(lua, &timeout_hook, &time_base, &table, rand_seed)?
                        .into_lua(lua)
                }
            }
        })?,
    )?;

    Ok(())
}

fn register_math_bindings(lua: &mut Lua) -> LuaResult<()> {
    let math = lua.globals().get::<_, LuaTable>("math")?;

    // function math.random([min], [max])
    math.raw_set(
        "random",
        lua.create_function(|lua, args: LuaMultiValue| -> LuaResult<LuaNumber> {
            let rand = &mut lua
                .app_data_mut::<LuaAppData>()
                .expect("Failed to access Lua app data")
                .rand_rgn;
            if args.is_empty() {
                Ok(rand.gen::<LuaNumber>())
            } else if args.len() == 1 {
                let max = args.get(0).unwrap().as_integer();
                if let Some(max) = max {
                    if max >= 1 {
                        let rand_int: LuaInteger = rand.gen_range(1..=max);
                        Ok(rand_int as LuaNumber)
                    } else {
                        Err(bad_argument_error(
                            "math.random",
                            "max",
                            1,
                            "invalid interval: max must be >= 1",
                        ))
                    }
                } else {
                    Err(bad_argument_error(
                        "math.random",
                        "max",
                        1,
                        "expecting an integer value",
                    ))
                }
            } else if args.len() == 2 {
                let min = args.get(0).unwrap().as_integer();
                let max = args.get(1).unwrap().as_integer();
                if let Some(min) = min {
                    if let Some(max) = max {
                        if max >= min {
                            let rand_int: LuaInteger = rand.gen_range(min..=max);
                            Ok(rand_int as LuaNumber)
                        } else {
                            Err(bad_argument_error(
                                "math.random",
                                "max",
                                1,
                                "invalid interval: max must be >= min",
                            ))
                        }
                    } else {
                        Err(bad_argument_error(
                            "math.random",
                            "max",
                            1,
                            "expecting an integer value",
                        ))
                    }
                } else {
                    Err(bad_argument_error(
                        "math.random",
                        "min",
                        1,
                        "expecting an integer value",
                    ))
                }
            } else {
                Err(bad_argument_error(
                    "math.random",
                    "undefined",
                    3,
                    "wrong number of arguments",
                ))
            }
        })?,
    )?;

    // function math.randomseed(seed)
    math.raw_set(
        "randomseed",
        lua.create_function(|lua, arg: LuaNumber| -> LuaResult<()> {
            let bytes = arg.to_le_bytes();
            let mut new_seed = [0; 32];
            for i in 0..32 {
                new_seed[i] = bytes[i % 8];
            }
            let mut app_data = lua
                .app_data_mut::<LuaAppData>()
                .expect("Failed to access Lua app data");
            app_data.rand_seed = Some(new_seed);
            app_data.rand_rgn = Xoshiro256PlusPlus::from_seed(new_seed);
            Ok(())
        })?,
    )?;

    Ok(())
}

fn register_table_bindings(lua: &mut Lua) -> LuaResult<()> {
    // cache module bytecode to speed up initialization
    lazy_static! {
        static ref TABLE_BYTECODE: LuaResult<Vec<u8>> = {
            let strip = true;
            Lua::new_with(LuaStdLib::NONE, LuaOptions::default())?
                .load(include_str!("../types/nerdo/library/table.lua"))
                .into_function()
                .map(|x| x.dump(strip))
        };
    }
    // implemented in lua: load and evaluate cached chunk
    match TABLE_BYTECODE.as_ref() {
        Ok(bytecode) => lua
            .load(bytecode)
            .set_name("[inbuilt:table.lua]")
            .set_mode(mlua::ChunkMode::Binary)
            .exec(),
        Err(err) => Err(err.clone()),
    }
}

fn register_pattern_module(lua: &mut Lua) -> LuaResult<()> {
    // cache module bytecode to speed up requires
    lazy_static! {
        static ref FUN_BYTECODE: LuaResult<Vec<u8>> = {
            let strip = true;
            Lua::new_with(LuaStdLib::NONE, LuaOptions::default())?
                .load(include_str!("../types/nerdo/library/extras/pattern.lua"))
                .into_function()
                .map(|x| x.dump(strip))
        };
    }
    // see https://github.com/khvzak/mlua/discussions/322
    let package: LuaTable = lua.globals().get("package")?;
    let loaders: LuaTable = package.get("loaders")?; // NB: "searchers" in lua 5.2
    loaders.push(LuaFunction::wrap(|lua, path: String| {
        if path == "pattern" {
            LuaFunction::wrap(|lua, ()| match FUN_BYTECODE.as_ref() {
                Ok(bytecode) => lua
                    .load(bytecode)
                    .set_name("[inbuilt:pattern.lua]")
                    .set_mode(mlua::ChunkMode::Binary)
                    .call::<_, LuaValue>(()),
                Err(err) => Err(err.clone()),
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
    fn extensions() -> LuaResult<()> {
        // create a new engine and register bindings
        let (mut lua, mut timeout_hook) = new_engine()?;
        register_bindings(
            &mut lua,
            &timeout_hook,
            &BeatTimeBase {
                beats_per_min: 160.0,
                beats_per_bar: 6,
                samples_per_sec: 96000,
            },
        )
        .unwrap();

        // reset timeout
        timeout_hook.reset();

        // table.lua is present
        assert!(lua.load(r#"return table.new()"#).eval::<LuaTable>().is_ok());

        // pattern.lua is present, but only when required
        assert!(lua
            .load(r#"return pattern.new()"#)
            .eval::<LuaTable>()
            .is_err());
        assert!(lua
            .load(
                r#"
                local pattern = require "pattern"
                return pattern.new()
                "#
            )
            .eval::<LuaTable>()
            .is_ok());

        // timeout hook is installed and does its job
        assert!(lua
            .load(
                r#"
                local i = 0
                while true do 
                    i = i + 1
                end
                "#,
            )
            .exec()
            .is_err_and(|e| e.to_string().contains("Script timeout")));

        // timeout is reset now, so further execution should work
        assert!(lua
            .load(
                r#"
                local i = 0
                while i < 100 do 
                    i = i + 1
                end
                "#,
            )
            .exec()
            .is_ok());
        Ok(())
    }
}
