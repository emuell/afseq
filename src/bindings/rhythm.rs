use std::{cell::RefCell, rc::Rc};

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
) -> LuaResult<Rc<RefCell<dyn Rhythm>>> {
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
            Err(LuaError::ToLuaConversionError {
                from: "userdata",
                to: "rhythm",
                message: Some(
                    "Expected script to return an emitter, got some other userdata".to_string(),
                ),
            })
        }
    } else {
        Err(LuaError::ToLuaConversionError {
            from: "userdata",
            to: "rhythm",
            message: Some(format!(
                "Expected script to return a emitter, got {}",
                result.type_name()
            )),
        })
    }
}

// --------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use crate::{bindings::*, event::Event, note::Note, time::BeatTimeStep};

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
            vec![
                PulseIterItem {
                    value: 1.0,
                    step_time: 1.0,
                },
                PulseIterItem {
                    value: 0.0,
                    step_time: 1.0,
                },
                PulseIterItem {
                    value: 1.0,
                    step_time: 1.0,
                },
                PulseIterItem {
                    value: 0.0,
                    step_time: 1.0,
                }
            ]
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
                })])),
                11025
            ))
        );

        // BeatTimeRhythm function Context
        let beat_time_rhythm = lua
            .load(
                r#"
                local pattern_step, emitter_step = 1, 1
                local function validate_context(context, step) 
                    assert(context.beats_per_min == 120)
                    assert(context.beats_per_bar == 4)
                    assert(context.sample_rate == 44100)
                    assert(context.step == step)
                end 
                return emitter {
                    unit = "1/4",
                    pattern = function(context)
                      validate_context(context, pattern_step)
                      pattern_step = pattern_step + 1
                      return 1
                    end,
                    emit = function(context)
                      validate_context(context, emitter_step)
                      emitter_step = emitter_step + 1
                      return "c4"
                    end
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
        let event = beat_time_rhythm.unwrap().next();
        assert_eq!(
            event,
            Some((
                0,
                Some(Event::NoteEvents(vec![Some(NoteEvent {
                    instrument: None,
                    note: Note::C4,
                    volume: 1.0,
                    panning: 0.0,
                    delay: 0.0
                })])),
                22050,
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
                beats_per_min: 130.0,
                beats_per_bar: 8,
                samples_per_sec: 48000,
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
            vec![
                PulseIterItem {
                    value: 1.0,
                    step_time: 1.0,
                },
                PulseIterItem {
                    value: 0.0,
                    step_time: 1.0,
                },
                PulseIterItem {
                    value: 1.0,
                    step_time: 1.0,
                },
                PulseIterItem {
                    value: 0.0,
                    step_time: 1.0,
                }
            ]
        );
        drop(pattern);

        // SecondTimeRhythm function Context
        let second_time_rhythm = lua
            .load(
                r#"
                local pattern_step, emitter_step = 1, 1
                local function validate_context(context, step) 
                    assert(context.beats_per_min == 130)
                    assert(context.beats_per_bar == 8)
                    assert(context.sample_rate == 48000)
                    assert(context.step == step)
                end 
                return emitter {
                    unit = "ms",
                    pattern = function(context)
                      validate_context(context, pattern_step)
                      pattern_step = pattern_step + 1
                      return 1
                    end,
                    emit = function(context)
                      validate_context(context, emitter_step)
                      emitter_step = emitter_step + 1
                      return "c4"
                    end
                }
            "#,
            )
            .eval::<LuaValue>()
            .unwrap();
        let second_time_rhythm = second_time_rhythm
            .as_userdata()
            .unwrap()
            .borrow_mut::<SecondTimeRhythm>();
        assert!(second_time_rhythm.is_ok());
        let event = second_time_rhythm.unwrap().next();
        assert_eq!(
            event,
            Some((
                0,
                Some(Event::NoteEvents(vec![Some(NoteEvent {
                    instrument: None,
                    note: Note::C4,
                    volume: 1.0,
                    panning: 0.0,
                    delay: 0.0
                })],),),
                48
            ))
        );
    }
}
