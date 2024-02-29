use mlua::prelude::*;

use super::super::{unwrap::*, LuaTimeoutHook};
use crate::prelude::*;

// -------------------------------------------------------------------------------------------------

impl LuaUserData for BeatTimeRhythm {
    // BeatTimeRhythm is only passed through ATM
}

impl BeatTimeRhythm {
    // create a BeatTimeRhythm from the given Lua table value
    pub(crate) fn from_table(
        lua: &Lua,
        timeout_hook: &LuaTimeoutHook,
        time_base: &BeatTimeBase,
        table: LuaTable,
    ) -> LuaResult<BeatTimeRhythm> {
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
                "bars" | "1/1" => step = BeatTimeStep::Bar(resolution),
                "beats" | "1/4" => step = BeatTimeStep::Beats(resolution),
                "1/8" => step = BeatTimeStep::Eighth(resolution),
                "1/16" => step = BeatTimeStep::Sixteenth(resolution),
                "1/32" => step = BeatTimeStep::ThirtySecond(resolution),
                "1/64" => step = BeatTimeStep::SixtyFourth(resolution),
                _ => return Err(bad_argument_error("emit", "unit", 1, 
                "Invalid unit parameter. Expected one of 'ms|seconds' or 'bars|beats' or '1/1|1/2|1/4|1/8|1/16|1/32|1/64"))
            }
        }
        let mut rhythm = BeatTimeRhythm::new(*time_base, step);
        if table.contains_key("offset")? {
            let offset = table.get::<&str, f32>("offset")?;
            rhythm = rhythm.with_offset_in_step(offset);
        }
        if table.contains_key("pattern")? {
            let value = table.get::<&str, LuaValue>("pattern")?;
            let pattern = pattern_from_value(lua, timeout_hook, value, time_base)?;
            rhythm = rhythm.with_pattern_dyn(pattern);
        }
        if table.contains_key("emit")? {
            let value = table.get::<&str, LuaValue>("emit")?;
            let iter = event_iter_from_value(lua, timeout_hook, value, time_base)?;
            rhythm = rhythm.trigger_dyn(iter);
        }
        Ok(rhythm)
    }
}
