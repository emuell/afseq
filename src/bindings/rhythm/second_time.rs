use std::rc::Rc;

use mlua::prelude::*;

use super::super::{
    unwrap::{
        bad_argument_error, event_iter_from_value, pattern_from_value,
        pattern_repeat_count_from_value,
    },
    LuaTimeoutHook,
};

use crate::prelude::*;

// -------------------------------------------------------------------------------------------------

impl LuaUserData for SecondTimeRhythm {
    // SecondTimeRhythm is only passed through ATM
}

impl SecondTimeRhythm {
    // create a SecondtimeRhythm from the given Lua table value
    pub(crate) fn from_table(
        lua: &Lua,
        timeout_hook: &LuaTimeoutHook,
        time_base: &BeatTimeBase,
        table: &LuaTable,
        rand_seed: Option<[u8; 32]>,
    ) -> LuaResult<SecondTimeRhythm> {
        // resolution
        let mut resolution = 1.0;
        if table.contains_key("resolution")? {
            resolution = table.get::<&str, f64>("resolution")?;
            if resolution <= 0.0 {
                return Err(bad_argument_error(
                    "emit",
                    "resolution",
                    1,
                    "resolution must be > 0",
                ));
            }
        }
        // unit
        if table.contains_key("unit")? {
            let unit = table.get::<&str, String>("unit")?;
            match unit.as_str() {
                "seconds" => (),
                "ms" => resolution /= 1000.0,
                _ => return Err(bad_argument_error("emit", "unit", 1, 
                "Invalid unit parameter. Expected one of 'ms|seconds' or 'bars|beats' or '1/1|1/2|1/4|1/8|1/16|1/32|1/64"))
            }
        }
        // create a new SecondTimeRhythm with the given time base and step
        let mut rhythm = SecondTimeRhythm::new(*time_base, resolution, rand_seed);
        // offset
        if table.contains_key("offset")? {
            let offset = table.get::<&str, f32>("offset")? as SecondTimeStep;
            if offset >= 0.0 {
                rhythm = rhythm.with_offset(offset * resolution);
            } else {
                return Err(bad_argument_error(
                    "emit",
                    "offset",
                    1,
                    "Offset must be a number >= 0",
                ));
            }
        }
        // pattern
        if table.contains_key("pattern")? {
            let value = table.get::<&str, LuaValue>("pattern")?;
            let pattern = pattern_from_value(lua, timeout_hook, &value, time_base)?;
            rhythm = rhythm.with_pattern_dyn(Rc::clone(&pattern));
        }
        // repeat
        if table.contains_key("repeats")? {
            let value = table.get::<&str, LuaValue>("repeats")?;
            let repeat = pattern_repeat_count_from_value(&value)?;
            rhythm = rhythm.with_repeat(repeat);
        }
        // emit
        if table.contains_key("emit")? {
            let value: LuaValue<'_> = table.get::<&str, LuaValue>("emit")?;
            let event_iter = event_iter_from_value(lua, timeout_hook, &value, time_base)?;
            rhythm = rhythm.trigger_dyn(event_iter);
        }
        Ok(rhythm)
    }
}
