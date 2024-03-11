use std::rc::Rc;

use mlua::prelude::*;

use super::super::{unwrap::*, LuaTimeoutHook};
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
        table: LuaTable,
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
        let mut rhythm = SecondTimeRhythm::new(SecondTimeBase::from(*time_base), resolution);
        // offset
        if table.contains_key("offset")? {
            let offset = table.get::<&str, f32>("offset")? as SecondTimeStep;
            rhythm = rhythm.with_offset(offset);
        }
        // pattern
        let mut pattern = rhythm.pattern();
        if table.contains_key("pattern")? {
            let value = table.get::<&str, LuaValue>("pattern")?;
            pattern = pattern_from_value(lua, timeout_hook, value, time_base)?;
            rhythm = rhythm.with_pattern_dyn(Rc::clone(&pattern));
        }
        // emit
        if table.contains_key("emit")? {
            let value: LuaValue<'_> = table.get::<&str, LuaValue>("emit")?;
            let iter = event_iter_from_value(lua, timeout_hook, value, time_base, pattern)?;
            rhythm = rhythm.trigger_dyn(iter);
        }
        Ok(rhythm)
    }
}
