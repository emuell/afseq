use std::{cell::RefCell, rc::Rc};

use anyhow::anyhow;
use mlua::prelude::*;

use crate::{
    event::InstrumentId,
    rhythm::{beat_time::BeatTimeRhythm, second_time::SecondTimeRhythm, Rhythm},
};

// ---------------------------------------------------------------------------------------------

mod beat_time;
mod second_time;

// ---------------------------------------------------------------------------------------------

// unwrap a BeatTimeRhythm or SecondTimeRhythm from the given LuaValue,
// which is expected to be a user data
pub(crate) fn rhythm_from_userdata(
    result: LuaValue,
    instrument: Option<InstrumentId>,
) -> Result<Rc<RefCell<dyn Rhythm>>, Box<dyn std::error::Error>> {
    if let Some(user_data) = result.as_userdata() {
        if let Ok(beat_time_rhythm) = user_data.take::<BeatTimeRhythm>() {
            Ok(Rc::new(RefCell::new(
                beat_time_rhythm.with_instrument(instrument),
            )))
        } else if let Ok(second_time_rhythm) = user_data.take::<SecondTimeRhythm>() {
            Ok(Rc::new(RefCell::new(
                second_time_rhythm.with_instrument(instrument),
            )))
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

// --------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use crate::{bindings::*, event::Event, note::Note};

    #[test]
    fn beat_time() {
        // create a new engine and register bindings
        let (mut lua, mut timeout_hook) = new_engine();
        register_bindings(
            &mut lua,
            &timeout_hook,
            BeatTimeBase {
                beats_per_min: 120.0,
                beats_per_bar: 4,
                samples_per_sec: 44100,
            },
        )
        .unwrap();

        // reset timeout
        timeout_hook.reset();

        // BeatTimeRhythm
        let beat_time_rhythm = lua
            .load(
                r#"
                emitter {
                    unit = "beats",
                    resolution = 0.5,
                    offset = "2",
                    pattern = {1,0,1,0},
                    emit = "c6"
                }
            "#,
            )
            .eval::<LuaValue>()
            .unwrap();
        let beat_time_rhythm = beat_time_rhythm
            .as_userdata()
            .unwrap()
            .borrow_mut::<BeatTimeRhythm>();
        assert!(beat_time_rhythm.is_ok());
        let mut beat_time_rhythm = beat_time_rhythm.unwrap();
        assert_eq!(beat_time_rhythm.step(), BeatTimeStep::Beats(0.5));
        assert_eq!(beat_time_rhythm.offset(), BeatTimeStep::Beats(2.0));
        let pattern = beat_time_rhythm.pattern();
        let mut pattern = pattern.borrow_mut();
        assert_eq!(
            vec![pattern.run(), pattern.run(), pattern.run(), pattern.run()],
            vec![1.0, 0.0, 1.0, 0.0]
        );
        drop(pattern);
        let event = beat_time_rhythm.next();
        assert_eq!(
            event,
            Some((
                44100,
                Some(Event::NoteEvents(vec![Some(NoteEvent {
                    instrument: None,
                    note: Note::C6,
                    volume: 1.0,
                    panning: 0.0,
                    delay: 0.0
                })]))
            ))
        );
    }

    #[test]
    fn second_time() {
        // create a new lua and register bindings
        let (mut lua, mut timeout_hook) = new_engine();
        register_bindings(
            &mut lua,
            &timeout_hook,
            BeatTimeBase {
                beats_per_min: 120.0,
                beats_per_bar: 4,
                samples_per_sec: 44100,
            },
        )
        .unwrap();

        // reset timeout
        timeout_hook.reset();

        // SecondTimeRhythm
        let second_time_rhythm = lua
            .load(
                r#"
                emitter {
                    unit = "seconds",
                    resolution = 2,
                    offset = 3,
                    pattern = {1,0,1,0},
                    emit = {"c5", "c5 v0.4", {"c7", "c7 v2.0"}}
                }
            "#,
            )
            .eval::<LuaValue>()
            .unwrap();

        let second_time_rhythm = second_time_rhythm
            .as_userdata()
            .unwrap()
            .borrow::<SecondTimeRhythm>();
        assert!(second_time_rhythm.is_ok());
        let second_time_rhythm = second_time_rhythm.unwrap();
        assert_eq!(second_time_rhythm.step(), 2.0);
        assert_eq!(second_time_rhythm.offset(), 3.0);
        let pattern = second_time_rhythm.pattern();
        let mut pattern = pattern.borrow_mut();
        assert_eq!(
            vec![pattern.run(), pattern.run(), pattern.run(), pattern.run()],
            vec![1.0, 0.0, 1.0, 0.0]
        );
        drop(pattern);
    }
}
