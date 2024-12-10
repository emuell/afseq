//! Lua bindings for the entire crate.

use std::{cell::RefCell, rc::Rc};

use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

use lazy_static::lazy_static;

use mlua::prelude::*;

use self::{
    cycle::CycleUserData,
    input::InputParameterUserData,
    note::NoteUserData,
    rhythm::rhythm_from_userdata,
    sequence::SequenceUserData,
    unwrap::{
        bad_argument_error, optional_string_from_value, string_from_value,
        validate_table_properties,
    },
};

use crate::{
    chord::unique_chord_names,
    event::InstrumentId,
    rhythm::{beat_time::BeatTimeRhythm, second_time::SecondTimeRhythm, Rhythm},
    time::BeatTimeBase,
    InputParameter, Scale,
};

// ---------------------------------------------------------------------------------------------

// private binding impls
mod callback;
mod cycle;
mod input;
mod note;
mod rhythm;
mod scale;
mod sequence;
mod timeout;
mod unwrap;

// public re-exports
pub use callback::{
    add_lua_callback_error, clear_lua_callback_errors, has_lua_callback_errors, lua_callback_errors,
};

// internal re-exports
pub(crate) use callback::{ContextPlaybackState, LuaCallback};
pub(crate) use timeout::LuaTimeoutHook;
pub(crate) use unwrap::{
    gate_trigger_from_value, note_events_from_value, pattern_pulse_from_value,
};

// ---------------------------------------------------------------------------------------------

/// Global sharedLua data, unique to each new Lua instance.
#[derive(Debug, Clone)]
pub(crate) struct LuaAppData {
    /// Global random seed, set by math.randomseed() for each Lua instance and passed to
    /// newly created rhythm impls.
    pub(crate) rand_seed: Option<u64>,
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
        // Only basics: no OS, IO, PACKAGE, DEBUG, FFI!
        LuaStdLib::STRING | LuaStdLib::TABLE | LuaStdLib::MATH,
        LuaOptions::default(),
    )
    .expect("Failed to create a new lua engine");
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
    register_parameter_bindings(lua)?;
    register_math_bindings(lua)?;
    register_table_bindings(lua)?;
    register_pattern_bindings(lua)?;
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
                    match Scale::try_from((note, mode.as_ref())) {
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

    globals.raw_set(
        "scale_names",
        lua.create_function(|lua, _args: ()| -> LuaResult<LuaTable> {
            lua.create_sequence_from(Scale::mode_names())
        })?,
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

    // function chord_names()
    globals.raw_set(
        "chord_names",
        lua.create_function(|lua, _args: LuaMultiValue| -> LuaResult<LuaTable> {
            lua.create_sequence_from(unique_chord_names())
        })?,
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
                const RHYTHM_PROPERTIES: [&str; 8] = [
                    "unit",
                    "resolution",
                    "offset",
                    "inputs",
                    "pattern",
                    "gate",
                    "repeats",
                    "emit",
                ];
                validate_table_properties(&table, &RHYTHM_PROPERTIES)?;
                // check which time unit is specified
                let second_time_unit = match table.get::<String>("unit") {
                    Ok(unit) => matches!(unit.as_str(), "seconds" | "ms"),
                    Err(_) => false,
                };
                if second_time_unit {
                    SecondTimeRhythm::from_table(lua, &timeout_hook, &time_base, &table)?
                        .into_lua(lua)
                } else {
                    BeatTimeRhythm::from_table(lua, &timeout_hook, &time_base, &table)?
                        .into_lua(lua)
                }
            }
        })?,
    )?;

    Ok(())
}

fn register_parameter_bindings(lua: &mut Lua) -> LuaResult<()> {
    let parameter = lua.create_table()?;

    // function boolean(id, default, name?, description?)
    parameter.raw_set(
        "boolean",
        lua.create_function(
            |_lua,
             (id, default, name, description): (LuaValue, LuaValue, LuaValue, LuaValue)|
             -> LuaResult<InputParameterUserData> {
                let id = string_from_value(&id, "boolean", "id", 1)?;
                if id.is_empty() {
                    return Err(bad_argument_error(
                        "boolean",
                        "id",
                        1,
                        "ids can not be empty",
                    ));
                }
                let default = default.as_boolean().ok_or_else(|| {
                    bad_argument_error("boolean", "default", 2, "expecting a boolean value")
                })?;
                let name = optional_string_from_value(&name, "boolean", "name", 3)?;
                let description =
                    optional_string_from_value(&description, "boolean", "description", 4)?;
                Ok(InputParameterUserData {
                    parameter: InputParameter::new_boolean(&id, &name, &description, default),
                })
            },
        )?,
    )?;

    // function integer(id, default, range, name?, description?)
    parameter.raw_set(
        "integer",
        #[allow(clippy::unnecessary_cast)]
        lua.create_function(
            |_lua,
             (id, default, range, name, description): (
                LuaValue,
                LuaValue,
                Option<LuaTable>,
                LuaValue,
                LuaValue,
            )|
             -> LuaResult<InputParameterUserData> {
                let id = string_from_value(&id, "integer", "id", 1)?;
                if id.is_empty() {
                    return Err(bad_argument_error(
                        "integer",
                        "id",
                        1,
                        "ids can not be empty",
                    ));
                }
                let default = default.as_integer().ok_or_else(|| {
                    bad_argument_error("integer", "default", 1, "expecting an integer value")
                })? as i32;
                let range = {
                    if let Some(range) = range {
                        let start = range.get::<LuaInteger>(1)? as i32;
                        let end = range.get::<LuaInteger>(2)? as i32;
                        start..=end
                    } else {
                        0..=100
                    }
                };
                if !range.contains(&default) {
                    return Err(bad_argument_error(
                        "integer",
                        "range",
                        2,
                        &format!(
                            "default value must be within range {}..={}",
                            range.start(),
                            range.end()
                        ),
                    ));
                }
                let name = optional_string_from_value(&name, "integer", "name", 3)?;
                let description =
                    optional_string_from_value(&description, "integer", "description", 4)?;
                Ok(InputParameterUserData {
                    parameter: InputParameter::new_integer(
                        &id,
                        &name,
                        &description,
                        range,
                        default,
                    ),
                })
            },
        )?,
    )?;

    // function number_input(id, default, range, name?, description?)
    parameter.raw_set(
        "number",
        #[allow(clippy::unnecessary_cast)]
        lua.create_function(
            |_lua,
             (id, default, range, name, description): (
                LuaValue,
                LuaValue,
                Option<LuaTable>,
                LuaValue,
                LuaValue,
            )|
             -> LuaResult<InputParameterUserData> {
                let id = string_from_value(&id, "number", "id", 1)?;
                if id.is_empty() {
                    return Err(bad_argument_error(
                        "number",
                        "id",
                        1,
                        "ids can not be empty",
                    ));
                }
                let default = default
                    .as_number()
                    .or_else(|| default.as_integer().map(|v| v as LuaNumber))
                    .ok_or_else(|| {
                        bad_argument_error("number", "default", 1, "expecting a number value")
                    })? as f64;
                let range = {
                    if let Some(range) = range {
                        let start = range.get::<f64>(1)?;
                        let end = range.get::<f64>(2)?;
                        start..=end
                    } else {
                        0.0..=1.0
                    }
                };
                if !range.contains(&default) {
                    return Err(bad_argument_error(
                        "number",
                        "range",
                        2,
                        &format!(
                            "default value must be within range {}..={}",
                            range.start(),
                            range.end()
                        ),
                    ));
                }
                let name = optional_string_from_value(&name, "number", "name", 3)?;
                let description =
                    optional_string_from_value(&description, "number", "description", 4)?;
                Ok(InputParameterUserData {
                    parameter: InputParameter::new_float(&id, &name, &description, range, default),
                })
            },
        )?,
    )?;

    // function enum(id, default, values, name?, description?)
    parameter.raw_set(
        "enum",
        lua.create_function(
            |_lua,
             (id, default, value_table, name, description): (
                LuaValue,
                LuaValue,
                LuaTable,
                LuaValue,
                LuaValue,
            )|
             -> LuaResult<InputParameterUserData> {
                let id = string_from_value(&id, "enum", "id", 1)?;
                if id.is_empty() {
                    return Err(bad_argument_error("enum", "id", 1, "ids can not be empty"));
                }
                let default = string_from_value(&default, "enum", "default", 2)?;
                let mut values = Vec::with_capacity(value_table.len()? as usize);
                for value in value_table.sequence_values::<String>() {
                    values.push(value?);
                }
                if !values.iter().any(|v| v.eq_ignore_ascii_case(&default)) {
                    return Err(bad_argument_error(
                        "enum",
                        "values",
                        2,
                        "values must contain the default value",
                    ));
                }
                if (1..values.len()).any(|i| {
                    values[i..]
                        .iter()
                        .any(|v| v.eq_ignore_ascii_case(&values[i - 1]))
                }) {
                    return Err(bad_argument_error(
                        "enum",
                        "values",
                        2,
                        "values must not contain duplicate entries",
                    ));
                }
                let name = optional_string_from_value(&name, "enum", "name", 3)?;
                let description =
                    optional_string_from_value(&description, "enum", "description", 4)?;
                Ok(InputParameterUserData {
                    parameter: InputParameter::new_enum(&id, &name, &description, values, default),
                })
            },
        )?,
    )?;

    lua.globals().set("parameter", parameter)?;

    Ok(())
}

fn register_math_bindings(lua: &mut Lua) -> LuaResult<()> {
    let math = lua.globals().get::<LuaTable>("math")?;

    // cache module bytecode to speed up initialization
    lazy_static! {
        static ref MATH_BYTECODE: LuaResult<Vec<u8>> = compile_chunk(
            include_str!("../types/nerdo/library/math.lua"),
            "[inbuilt:math.lua]"
        );
    }
    // implemented in lua: load and evaluate cached chunk
    match MATH_BYTECODE.as_ref() {
        Ok(bytecode) => lua
            .load(bytecode)
            .set_name("[inbuilt:math.lua]")
            .set_mode(mlua::ChunkMode::Binary)
            .exec()?,
        Err(err) => return Err(err.clone()),
    };

    // function math.random([min], [max])
    math.raw_set(
        "random",
        lua.create_function(|lua, args: LuaMultiValue| -> LuaResult<LuaNumber> {
            let rand = &mut lua
                .app_data_mut::<LuaAppData>()
                .expect("Failed to access Lua app data")
                .rand_rgn;
            generate_random_number("math.random", rand, args)
        })?,
    )?;

    // function math.randomseed(seed)
    math.raw_set(
        "randomseed",
        lua.create_function(|lua, arg: LuaNumber| -> LuaResult<()> {
            let new_seed = arg as u64;
            let mut app_data = lua
                .app_data_mut::<LuaAppData>()
                .expect("Failed to access Lua app data");
            app_data.rand_seed = Some(new_seed);
            app_data.rand_rgn = Xoshiro256PlusPlus::seed_from_u64(new_seed);
            Ok(())
        })?,
    )?;

    // function math.randomstate(seed?)
    math.raw_set(
        "randomstate",
        lua.create_function(|lua, seed: LuaValue| -> LuaResult<LuaFunction> {
            let seed = {
                if let Some(seed) = seed.as_number() {
                    seed.trunc() as u64
                } else if let Some(seed) = seed.as_integer() {
                    seed as u64
                } else if seed.is_nil() {
                    let app_data = lua
                        .app_data_mut::<LuaAppData>()
                        .expect("Failed to access Lua app data");
                    app_data.rand_seed.unwrap_or(rand::thread_rng().gen())
                } else {
                    return Err(bad_argument_error(
                        "randomstate",
                        "seed",
                        1,
                        "expecting an integer value",
                    ));
                }
            };
            lua.create_function_mut({
                let mut rand = Xoshiro256PlusPlus::seed_from_u64(seed);
                move |_lua: &Lua, args: LuaMultiValue| -> LuaResult<LuaNumber> {
                    generate_random_number("math.randomstate", &mut rand, args)
                }
            })
        })?,
    )?;

    Ok(())
}

fn register_table_bindings(lua: &mut Lua) -> LuaResult<()> {
    // cache module bytecode to speed up initialization
    lazy_static! {
        static ref TABLE_BYTECODE: LuaResult<Vec<u8>> = compile_chunk(
            include_str!("../types/nerdo/library/table.lua"),
            "[inbuilt:table.lua]"
        );
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

fn register_pattern_bindings(lua: &mut Lua) -> LuaResult<()> {
    // cache module bytecode to speed up requires
    lazy_static! {
        static ref PATTERN_BYTECODE: LuaResult<Vec<u8>> = compile_chunk(
            include_str!("../types/nerdo/library/pattern.lua"),
            "[inbuilt:pattern.lua]"
        );
    }
    // implemented in lua: load and evaluate cached chunk
    match PATTERN_BYTECODE.as_ref() {
        Ok(bytecode) => lua
            .load(bytecode)
            .set_name("[inbuilt:pattern.lua]")
            .set_mode(mlua::ChunkMode::Binary)
            .exec(),
        Err(err) => Err(err.clone()),
    }
}

// --------------------------------------------------------------------------------------------------

// Generate a new random number in Lua math.random style.
fn generate_random_number<R: rand::Rng>(
    func_name: &'static str,
    rand: &mut R,
    args: LuaMultiValue,
) -> LuaResult<LuaNumber> {
    if args.is_empty() {
        Ok(rand.gen::<LuaNumber>())
    } else if args.len() == 1 {
        #[allow(clippy::get_first)]
        let max = args.get(0).unwrap().as_integer();
        if let Some(max) = max {
            if max >= 1 {
                let rand_int: LuaInteger = rand.gen_range(1..=max);
                Ok(rand_int as LuaNumber)
            } else {
                Err(bad_argument_error(
                    func_name,
                    "max",
                    1,
                    "invalid interval: max must be >= 1",
                ))
            }
        } else {
            Err(bad_argument_error(
                func_name,
                "max",
                1,
                "expecting an integer value",
            ))
        }
    } else if args.len() == 2 {
        #[allow(clippy::get_first)]
        let min = args.get(0).unwrap().as_integer();
        let max = args.get(1).unwrap().as_integer();
        if let Some(min) = min {
            if let Some(max) = max {
                if max >= min {
                    let rand_int: LuaInteger = rand.gen_range(min..=max);
                    Ok(rand_int as LuaNumber)
                } else {
                    Err(bad_argument_error(
                        func_name,
                        "max",
                        1,
                        "invalid interval: max must be >= min",
                    ))
                }
            } else {
                Err(bad_argument_error(
                    func_name,
                    "max",
                    1,
                    "expecting an integer value",
                ))
            }
        } else {
            Err(bad_argument_error(
                func_name,
                "min",
                1,
                "expecting an integer value",
            ))
        }
    } else {
        Err(bad_argument_error(
            func_name,
            "undefined",
            3,
            "wrong number of arguments",
        ))
    }
}

// --------------------------------------------------------------------------------------------------

#[cfg(any(feature = "lua", feature = "lua-jit"))]
fn compile_chunk(chunk: &'static str, name: &'static str) -> LuaResult<Vec<u8>> {
    let strip = false;
    Lua::new_with(LuaStdLib::NONE, LuaOptions::default())?
        .load(chunk)
        .set_name(name)
        .into_function()
        .map(|x| x.dump(strip))
}

#[cfg(any(feature = "luau", feature = "luau-jit"))]
fn compile_chunk(chunk: &'static str, _name: &'static str) -> LuaResult<Vec<u8>> {
    Ok(mlua::Compiler::new().compile(chunk))
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

        // pattern.lua is present
        assert!(lua
            .load(r#"return pattern.new()"#)
            .eval::<LuaTable>()
            .is_ok());

        // math.randomstate is present
        assert!(lua
            .load(r#"return math.randomstate(123)(1, 10)"#)
            .eval::<LuaNumber>()
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
