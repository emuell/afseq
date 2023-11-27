use mlua::prelude::*;

use crate::prelude::*;
use super::super::unwrap::*;

// -------------------------------------------------------------------------------------------------

impl LuaUserData for BeatTimeRhythm {
    // BeatTimeRhythm is only passed through ATM
}

impl BeatTimeRhythm {
    // create a BeatTimeRhythm from the given Lua table value
    pub(crate) fn from_table(
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
                "beats" | "1/4" | "4th" => step = BeatTimeStep::Beats(resolution),
                "eighth" | "1/8" | "8th" => step = BeatTimeStep::Eighth(resolution),
                "sixteenth"|"1/16" | "16th" => step = BeatTimeStep::Sixteenth(resolution),
                _ => return Err(bad_argument_error("emit", "unit", 1, 
                "Invalid unit parameter. Expected one of 'seconds', 'bars', 'beats|4th', 'eighth|8th', 'sixteenth|8th'"))
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

