use std::rc::Rc;

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
        rand_seed: &[u8; 32],
    ) -> LuaResult<BeatTimeRhythm> {
        // resolution
        let mut resolution = 1.0;
        if table.contains_key("resolution")? {
            resolution = table.get::<&str, f32>("resolution")?;
            if resolution <= 0.0 {
                return Err(bad_argument_error(
                    "emit",
                    "resolution",
                    1,
                    "resolution must be > 0",
                ));
            }
        }
        // step
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
                "expected one of 'ms|seconds' or 'bars|beats' or '1/1|1/2|1/4|1/8|1/16|1/32|1/64"))
            }
        }
        // create a new BeatTimeRhythm with the given time base and step
        let mut rhythm = BeatTimeRhythm::new_with_seed(*time_base, step, rand_seed);
        // offset
        if table.contains_key("offset")? {
            let offset = table.get::<&str, f32>("offset")?;
            if offset >= 0.0 {
                let mut new_step = rhythm.step();
                new_step.set_steps(offset * resolution);
                rhythm = rhythm.with_offset(new_step);
            } else {
                return Err(bad_argument_error(
                    "emit",
                    "offset",
                    1,
                    "offset must be >= 0",
                ));
            }
        }
        // pattern
        if table.contains_key("pattern")? {
            let value = table.get::<&str, LuaValue>("pattern")?;
            let pattern = pattern_from_value(lua, timeout_hook, value, time_base)?;
            rhythm = rhythm.with_pattern_dyn(Rc::clone(&pattern));
        }
        // repeat
        if table.contains_key("repeats")? {
            let value = table.get::<&str, LuaValue>("repeats")?;
            let repeat = pattern_repeat_count_from_value(value)?;
            rhythm = rhythm.with_repeat(repeat);
        }
        // emit
        if table.contains_key("emit")? {
            let value = table.get::<&str, LuaValue>("emit")?;
            let event_iter = event_iter_from_value(lua, timeout_hook, value, time_base)?;
            rhythm = rhythm.trigger_dyn(event_iter);
        }
        Ok(rhythm)
    }
}
