use mlua::prelude::*;

use super::super::{
    unwrap::{
        bad_argument_error, event_iter_from_value, gate_from_value, inputs_from_value,
        pattern_from_value, pattern_repeat_count_from_value,
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
    ) -> LuaResult<SecondTimeRhythm> {
        // resolution
        let mut resolution = 1.0;
        if table.contains_key("resolution")? {
            resolution = table.get::<_, f64>("resolution")?;
            if resolution <= 0.0 {
                return Err(bad_argument_error(
                    "rhythm",
                    "resolution",
                    1,
                    "resolution must be > 0",
                ));
            }
        }
        // unit
        if table.contains_key("unit")? {
            let unit = table.get::<_, String>("unit")?;
            match unit.as_str() {
                "seconds" => (),
                "ms" => resolution /= 1000.0,
                _ => return Err(bad_argument_error("emit", "unit", 1, 
                "expected one of 'ms|seconds' or 'bars|beats' or '1/1|1/2|1/4|1/8|1/16|1/32|1/64"))
            }
        }
        // create a new SecondTimeRhythm with the given time base and step
        let mut rhythm = SecondTimeRhythm::new(*time_base, resolution);
        // offset
        if table.contains_key("offset")? {
            let offset = table.get::<_, f32>("offset")? as SecondTimeStep;
            if offset >= 0.0 {
                rhythm = rhythm.with_offset(offset * resolution);
            } else {
                return Err(bad_argument_error(
                    "emit",
                    "offset",
                    1,
                    "offset must be a number >= 0",
                ));
            }
        }
        // inputs
        if table.contains_key("inputs")? {
            let value = table.get::<_, LuaTable>("inputs")?;
            let inputs = inputs_from_value(lua, &value)?;
            rhythm = rhythm.with_input_parameters(inputs);
        }
        // pattern
        if table.contains_key("pattern")? {
            let value = table.get::<_, LuaValue>("pattern")?;
            let pattern = pattern_from_value(lua, timeout_hook, &value, time_base)?;
            rhythm = rhythm.with_pattern_dyn(pattern);
        }
        // gate
        if table.contains_key("gate")? {
            let value = table.get::<_, LuaValue>("gate")?;
            let gate = gate_from_value(lua, timeout_hook, &value, time_base)?;
            rhythm = rhythm.with_gate_dyn(gate);
        }
        // repeat
        if table.contains_key("repeats")? {
            let value = table.get::<_, LuaValue>("repeats")?;
            let repeat = pattern_repeat_count_from_value(&value)?;
            rhythm = rhythm.with_repeat(repeat);
        }
        // emit
        if table.contains_key("emit")? {
            let value: LuaValue<'_> = table.get::<_, LuaValue>("emit")?;
            let event_iter = event_iter_from_value(lua, timeout_hook, &value, time_base)?;
            rhythm = rhythm.trigger_dyn(event_iter);
        }
        Ok(rhythm)
    }
}
