use std::{cell::RefCell, rc::Rc};

use anyhow::anyhow;
use mlua::prelude::*;

use crate::prelude::*;

// ---------------------------------------------------------------------------------------------

pub(crate) mod beat_time;
pub(crate) mod second_time;

// ---------------------------------------------------------------------------------------------

// unwrap a BeatTimeRhythm or SecondTimeRhythm from the given LuaValue,
// which is expected to be a user data
pub(crate) fn rhythm_from_userdata(
    result: LuaValue,
) -> Result<Rc<RefCell<dyn Rhythm>>, Box<dyn std::error::Error>> {
    if let Some(user_data) = result.as_userdata() {
        if let Ok(beat_time_rhythm) = user_data.take::<BeatTimeRhythm>() {
            Ok(Rc::new(RefCell::new(beat_time_rhythm)))
        } else if let Ok(second_time_rhythm) = user_data.take::<SecondTimeRhythm>() {
            Ok(Rc::new(RefCell::new(second_time_rhythm)))
        } else {
            Err(anyhow!("Expected script to return a Rhythm, got some other custom type",).into())
        }
    } else {
        Err(anyhow!(
            "Expected script to return a Rhythm, got {}",
            result.type_name()
        )
        .into())
    }
}
