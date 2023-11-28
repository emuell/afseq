use mlua::prelude::*;

use super::super::unwrap::*;
use crate::prelude::*;

// -------------------------------------------------------------------------------------------------

impl LuaUserData for SecondTimeRhythm {
    // SecondTimeRhythm is only passed through ATM
}

impl SecondTimeRhythm {
    // create a SecondtimeRhythm from the given Lua table value
    pub(crate) fn from_table(
        table: LuaTable,
        default_time_base: SecondTimeBase,
        default_instrument: Option<InstrumentId>,
    ) -> LuaResult<SecondTimeRhythm> {
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
                "Invalid unit parameter. Expected one of 'seconds', 'bars', 'beats|4th', 'eighth|8th', 'sixteenth|8th'"))
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
