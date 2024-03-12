//! Lua script bindings, to create rhythms dynamically.

use std::{
    cell::RefCell,
    env,
    rc::Rc,
    time::{Duration, Instant},
};

use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

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
    rhythm::{beat_time::BeatTimeRhythm, second_time::SecondTimeRhythm, Rhythm},
    time::BeatTimeBase,
    Pulse, PulseIterItem, Scale,
};

// -------------------------------------------------------------------------------------------------

// Limits script execution time and aborts execution when a script runs too long. This way e.g.
// never ending loops are stopped automatically with a timeout error.
//
// While constructed, it checks every few instructions if a timeout duration has been reached
// and then aborts the script by firing an error.
// When cloning and instance, it will use the existing hook, so ensure to call `reset` before
// invoking new lua functions. The last instance that get's dropped will then remove the hook.
#[derive(Debug)]
pub(crate) struct LuaTimeoutHook {
    active: Rc<RefCell<usize>>,
    start: Rc<RefCell<Instant>>,
}

impl LuaTimeoutHook {
    // default number of ms a script may run before a timeout error is fired.
    // assumes scripts are running in a real-time alike context.
    const DEFAULT_TIMEOUT: Duration = Duration::from_millis(200);

    fn new(lua: &Lua) -> Self {
        Self::new_with_timeout(lua, Self::DEFAULT_TIMEOUT)
    }

    fn new_with_timeout(lua: &Lua, timeout: Duration) -> Self {
        let active = Rc::new(RefCell::new(1));
        let start = Rc::new(RefCell::new(Instant::now()));
        lua.set_hook(LuaHookTriggers::new().every_nth_instruction(timeout.as_millis() as u32), {
            let active = Rc::clone(&active);
            let start = Rc::clone(&start);
            move |lua, _debug| {
                if *active.borrow() > 0 {
                    if start.borrow().elapsed() > timeout {
                        *start.borrow_mut() = Instant::now();
                        Err(LuaError::RuntimeError(
                            String::from("Script timeout. ")
                                + &format!("Execution took longer than {} ms to complete.\n", timeout.as_millis())
                                + "Please avoid overhead and check for never ending loops in your script. "
                                + "Also note that the script is running in real-time thread!",
                        ))
                    } else {
                        Ok(())
                    }
                } else {
                    lua.remove_hook();
                    Ok(())
                }
            }
        });
        Self { active, start }
    }

    // reset timestamp of the hook when running e.g. a callback again
    pub(crate) fn reset(&mut self) {
        *self.start.borrow_mut() = Instant::now();
    }
}

impl Clone for LuaTimeoutHook {
    fn clone(&self) -> Self {
        // increase active isntances refcount
        *self.active.borrow_mut() += 1;
        // return a direct clone otherwise
        Self {
            active: Rc::clone(&self.active),
            start: Rc::clone(&self.start),
        }
    }
}

impl Drop for LuaTimeoutHook {
    fn drop(&mut self) {
        // decrease active instance refcount.
        *self.active.borrow_mut() -= 1;
        // when reaching 0, this will remove the hook in the hook itself
    }
}

// ---------------------------------------------------------------------------------------------

/// Global sharedLua data, unique to each new Lua instance.
#[derive(Debug, Clone)]
pub(crate) struct LuaAppData {
    /// Global random seed, set by math.randomseed() for each Lua instance and passed to
    /// newly created rhythm impls.
    pub(crate) rand_seed: [u8; 32],
    /// Global random number generator, used for our math.random() impl.
    pub(crate) rand_rgn: Xoshiro256PlusPlus,
}

impl LuaAppData {
    fn new() -> Self {
        let rand_seed = rand::thread_rng().gen();
        let rand_rgn = Xoshiro256PlusPlus::from_seed(rand_seed);
        Self {
            rand_seed,
            rand_rgn,
        }
    }
}

// ---------------------------------------------------------------------------------------------

/// Create a new raw lua engine with preloaded packages, but no bindings. Also returns a timeout
/// hook instance to limit duration of script calls.
/// Use [register_bindings] to register the bindings for a newly created engine.
pub(crate) fn new_engine() -> (Lua, LuaTimeoutHook) {
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
        .exec()
        .unwrap_or_else(|err| log::warn!("Failed to initialize lua engine: {}", err));
    // install a timeout hook
    let timeout_hook = LuaTimeoutHook::new(&lua);
    // create new app data
    lua.set_app_data(LuaAppData::new());
    // return the lua instance and timeout manager
    (lua, timeout_hook)
}

// -------------------------------------------------------------------------------------------------

/// Evaluate a lua script file which creates and returns a rhythm.
pub fn new_rhythm_from_file(
    time_base: BeatTimeBase,
    instrument: Option<InstrumentId>,
    file_name: &str,
) -> Result<Rc<RefCell<dyn Rhythm>>, Box<dyn std::error::Error>> {
    // create a new engine and register bindings
    let (mut lua, mut timeout_hook) = new_engine();
    register_bindings(&mut lua, &timeout_hook, time_base)?;
    // restart the timeout hook
    timeout_hook.reset();
    // compile and evaluate script
    let chunk = lua.load(std::path::PathBuf::from(file_name));
    let result = chunk.eval::<LuaValue>()?;
    // convert result
    rhythm_from_userdata(result, instrument).map_err(|err| err.into())
}

/// Evaluate a Lua string expression which creates and returns a rhythm.
pub fn new_rhythm_from_string(
    time_base: BeatTimeBase,
    instrument: Option<InstrumentId>,
    script: &str,
    script_name: &str,
) -> Result<Rc<RefCell<dyn Rhythm>>, Box<dyn std::error::Error>> {
    // create a new engine and register bindings
    let (mut lua, mut timeout_hook) = new_engine();
    register_bindings(&mut lua, &timeout_hook, time_base)?;
    // restart the timeout hook
    timeout_hook.reset();
    // compile and evaluate script
    let chunk = lua.load(script).set_name(script_name);
    let result = chunk.eval::<LuaValue>()?;
    // convert result
    rhythm_from_userdata(result, instrument).map_err(|err| err.into())
}

/// Evaluate a precompiled Lua expression which creates and returns a rhythm.
pub fn new_rhythm_from_bytecode(
    time_base: BeatTimeBase,
    instrument: Option<InstrumentId>,
    script: &Vec<u8>,
    script_name: &str,
) -> Result<Rc<RefCell<dyn Rhythm>>, Box<dyn std::error::Error>> {
    // create a new engine and register bindings
    let (mut lua, mut timeout_hook) = new_engine();
    register_bindings(&mut lua, &timeout_hook, time_base)?;
    // restart the timeout hook
    timeout_hook.reset();
    // compile and evaluate script
    let chunk: LuaChunk = lua.load(script).set_name(script_name);
    let result = chunk.eval::<LuaValue>()?;
    // convert result
    rhythm_from_userdata(result, instrument).map_err(|err| err.into())
}

// -------------------------------------------------------------------------------------------------

/// Try converting the given lua value to a note events vector.
pub(crate) fn new_note_events_from_lua(
    arg: LuaValue,
    arg_index: Option<usize>,
) -> LuaResult<Vec<Option<NoteEvent>>> {
    unwrap::note_events_from_value(arg, arg_index)
}

/// Try converting the given lua value to a pattern pulse value.
pub(crate) fn pattern_pulse_from_lua(value: LuaValue) -> LuaResult<Pulse> {
    unwrap::pattern_pulse_from_value(value)
}

// -------------------------------------------------------------------------------------------------

/// Set or update the time base of the given emitter context.
pub(crate) fn initialize_context_time_base(
    table: &mut LuaOwnedTable,
    time_info: &BeatTimeBase,
) -> LuaResult<()> {
    let table = table.to_ref();
    table.raw_set("beats_per_min", time_info.beats_per_min)?;
    table.raw_set("beats_per_bar", time_info.beats_per_bar)?;
    table.raw_set("sample_rate", time_info.samples_per_sec)?;
    Ok(())
}

/// Set or update the step counter of the given emitter context.
pub(crate) fn initialize_context_step_count(
    table: &mut LuaOwnedTable,
    pulse_step: usize,
) -> LuaResult<()> {
    let table = table.to_ref();
    table.raw_set("step", pulse_step + 1)?;
    Ok(())
}

/// Set or update the time base and step counter of the given emitter context.
pub(crate) fn initialize_pattern_context(
    table: &mut LuaOwnedTable,
    time_info: &BeatTimeBase,
    pulse_step: usize,
) -> LuaResult<()> {
    initialize_context_time_base(table, time_info)?;
    initialize_context_step_count(table, pulse_step)?;
    Ok(())
}

/// Set or update the pulse value of the given emitter context.
pub(crate) fn initialize_context_pulse_value(
    table: &mut LuaOwnedTable,
    pulse: PulseIterItem,
    pulse_count: usize,
) -> LuaResult<()> {
    let table = table.to_ref();
    table.raw_set("step_value", pulse.value)?;
    table.raw_set("step_time", pulse.step_time)?;
    table.raw_set("step_count", pulse_count)?;
    Ok(())
}

/// Set or update the time base, step counter and pattern context of the given emitter context.
pub(crate) fn initialize_emitter_context(
    table: &mut LuaOwnedTable,
    time_info: &BeatTimeBase,
    pulse_step: usize,
    pulse: PulseIterItem,
    pulse_count: usize,
) -> LuaResult<()> {
    initialize_pattern_context(table, time_info, pulse_step)?;
    initialize_context_pulse_value(table, pulse, pulse_count)?;
    Ok(())
}

// -------------------------------------------------------------------------------------------------

/// Register afseq bindings with the given lua engine.
/// Engine instance is expected to be one created via [new_engine].
pub(crate) fn register_bindings(
    lua: &mut Lua,
    timeout_hook: &LuaTimeoutHook,
    time_base: BeatTimeBase,
) -> Result<(), Box<dyn std::error::Error>> {
    register_global_bindings(lua, timeout_hook, time_base)?;
    register_math_bindings(lua)?;
    register_table_bindings(lua)?;
    register_pattern_module(lua)?;
    register_fun_module(lua)?;
    Ok(())
}

fn register_global_bindings(
    lua: &mut Lua,
    timeout_hook: &LuaTimeoutHook,
    time_base: BeatTimeBase,
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
    globals.raw_set(
        "note",
        lua.create_function(|_lua, args: LuaMultiValue| -> LuaResult<NoteUserData> {
            NoteUserData::from(args)
        })?,
    )?;

    // function sequence(args...)
    globals.raw_set(
        "sequence",
        lua.create_function(|_lua, args: LuaMultiValue| -> LuaResult<SequenceUserData> {
            SequenceUserData::from(args)
        })?,
    )?;

    // function emitter { args... }
    globals.raw_set(
        "emitter",
        lua.create_function({
            let timeout_hook = timeout_hook.clone();
            move |lua, table: LuaTable| -> LuaResult<LuaValue> {
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
                    SecondTimeRhythm::from_table(lua, &timeout_hook, &time_base, table, &rand_seed)?
                        .into_lua(lua)
                } else {
                    BeatTimeRhythm::from_table(lua, &timeout_hook, &time_base, table, &rand_seed)?
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
            app_data.rand_seed = new_seed;
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

fn register_fun_module(lua: &mut Lua) -> LuaResult<()> {
    // cache module bytecode to speed up requires
    lazy_static! {
        static ref FUN_BYTECODE: LuaResult<Vec<u8>> = {
            let strip = true;
            Lua::new_with(LuaStdLib::NONE, LuaOptions::default())?
                .load(include_str!("../types/nerdo/library/extras/fun.lua"))
                .into_function()
                .map(|x| x.dump(strip))
        };
    }
    // see https://github.com/khvzak/mlua/discussions/322
    let package: LuaTable = lua.globals().get("package")?;
    let loaders: LuaTable = package.get("loaders")?; // NB: "searchers" in lua 5.2
    loaders.push(LuaFunction::wrap(|lua, path: String| {
        if path == "fun" {
            LuaFunction::wrap(|lua, ()| match FUN_BYTECODE.as_ref() {
                Ok(bytecode) => lua
                    .load(bytecode)
                    .set_name("[inbuilt:fun.lua]")
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
    fn extensions() {
        // create a new engine and register bindings
        let (mut lua, mut timeout_hook) = new_engine();
        register_bindings(
            &mut lua,
            &timeout_hook,
            BeatTimeBase {
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

        // fun.lua is present, but only when required
        assert!(lua
            .load(r#"return fun.iter {1,2,3}:map(function(v) return v*2 end):totable()"#)
            .eval::<LuaTable>()
            .is_err());
        assert!(lua
            .load(
                r#"
                local fun = require "fun"
                return fun.iter {1,2,3}:map(function(v) return v*2 end):totable()
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
    }
}
