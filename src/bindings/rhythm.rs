use std::{cell::RefCell, rc::Rc};

use mlua::prelude::*;

use crate::{
    bindings::{cycle::CycleUserData, unwrap::event_iter_from_value, LuaTimeoutHook},
    event::InstrumentId,
    rhythm::{beat_time::BeatTimeRhythm, second_time::SecondTimeRhythm, Rhythm},
    BeatTimeBase,
};

// ---------------------------------------------------------------------------------------------

mod beat_time;
mod second_time;

// ---------------------------------------------------------------------------------------------

// unwrap a BeatTimeRhythm or SecondTimeRhythm from the given LuaValue,
// which is expected to be a user data
pub(crate) fn rhythm_from_userdata(
    lua: &Lua,
    timeout_hook: &LuaTimeoutHook,
    value: &LuaValue,
    time_base: &BeatTimeBase,
    instrument: Option<InstrumentId>,
) -> LuaResult<Rc<RefCell<dyn Rhythm>>> {
    if let Some(user_data) = value.as_userdata() {
        if user_data.is::<BeatTimeRhythm>() {
            // NB: take instead of cloning: rhythm userdata has no other usage than being defined
            Ok(Rc::new(RefCell::new(
                user_data
                    .take::<BeatTimeRhythm>()?
                    .with_instrument(instrument),
            )))
        } else if user_data.is::<SecondTimeRhythm>() {
            Ok(Rc::new(RefCell::new(
                // NB: take instead of cloning: rhythm userdata has no other usage than being defined
                user_data
                    .take::<SecondTimeRhythm>()?
                    .with_instrument(instrument),
            )))
        } else if user_data.is::<CycleUserData>() {
            // create a default rhythm from the given cycle
            Ok(Rc::new(RefCell::new(
                BeatTimeRhythm::new(*time_base, crate::BeatTimeStep::Bar(1.0))
                    .with_instrument(instrument)
                    .trigger_dyn(event_iter_from_value(lua, timeout_hook, value, time_base)?),
            )))
        } else {
            Err(LuaError::FromLuaConversionError {
                from: "userdata",
                to: "rhythm".to_string(),
                message: Some(
                    "script must return a rhythm or cycle, got some other userdata instead"
                        .to_string(),
                ),
            })
        }
    } else {
        Err(LuaError::FromLuaConversionError {
            from: value.type_name(),
            to: "rhythm".to_string(),
            message: Some("script must return a rhythm or cycle".to_string()),
        })
    }
}

// --------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use crate::{
        bindings::*,
        event::{Event, NoteEvent},
        note::Note,
        rhythm::{beat_time::BeatTimeRhythm, second_time::SecondTimeRhythm, RhythmIterItem},
        time::BeatTimeStep,
        PulseIterItem,
    };

    fn new_test_engine(
        beats_per_min: f32,
        beats_per_bar: u32,
        samples_per_sec: u32,
    ) -> Result<(Lua, LuaTimeoutHook), LuaError> {
        let (mut lua, mut timeout_hook) = new_engine()?;
        register_bindings(
            &mut lua,
            &timeout_hook,
            &BeatTimeBase {
                beats_per_min,
                beats_per_bar,
                samples_per_sec,
            },
        )?;
        timeout_hook.reset();
        Ok((lua, timeout_hook))
    }

    #[test]
    fn beat_time() -> LuaResult<()> {
        let (lua, _) = new_test_engine(120.0, 4, 44100)?;

        // BeatTimeRhythm
        let beat_time_rhythm = lua
            .load(
                r#"
                rhythm {
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
        assert_eq!(beat_time_rhythm.offset(), BeatTimeStep::Beats(1.0));
        let pattern = beat_time_rhythm.pattern_mut();
        assert_eq!(
            vec![pattern.run(), pattern.run(), pattern.run(), pattern.run()],
            vec![
                Some(PulseIterItem {
                    value: 1.0,
                    step_time: 1.0,
                }),
                Some(PulseIterItem {
                    value: 0.0,
                    step_time: 1.0,
                }),
                Some(PulseIterItem {
                    value: 1.0,
                    step_time: 1.0,
                }),
                Some(PulseIterItem {
                    value: 0.0,
                    step_time: 1.0,
                })
            ]
        );

        let event = beat_time_rhythm.next();
        assert_eq!(
            event,
            Some(RhythmIterItem {
                time: 22050,
                event: Some(Event::NoteEvents(vec![Some(NoteEvent {
                    instrument: None,
                    note: Note::C6,
                    volume: 1.0,
                    panning: 0.0,
                    delay: 0.0
                })])),
                duration: 11025
            })
        );
        Ok(())
    }

    #[test]
    fn beat_time_callbacks() -> LuaResult<()> {
        let (lua, _) = new_test_engine(120.0, 4, 44100)?;

        // BeatTimeRhythm function Context
        let beat_time_rhythm = lua
            .load(
                r#"
                return rhythm {
                    unit = "1/4",
                    pattern = function()
                      local pulse_step, pulse_time_step = 1, 0.0 
                      local function validate_context(context) 
                        assert(context.beats_per_min == 120)
                        assert(context.beats_per_bar == 4)
                        assert(context.samples_per_sec == 44100)
                        assert(context.pulse_step == pulse_step)
                        assert(context.pulse_time_step == pulse_time_step)
                      end
                      return function(context)
                        validate_context(context)
                        pulse_step = pulse_step + 2
                        pulse_time_step = pulse_time_step + 1.0
                        return {1, 0}
                      end
                    end,
                    gate = function(context) 
                      assert(context.beats_per_min == 120)
                      assert(context.beats_per_bar == 4)
                      assert(context.samples_per_sec == 44100)
                      local pulse_step, pulse_time_step = 1, 0.0 
                      local function validate_context(context) 
                        assert(context.beats_per_min == 120)
                        assert(context.beats_per_bar == 4)
                        assert(context.samples_per_sec == 44100)
                        assert(context.pulse_step == pulse_step)
                        assert(context.pulse_time_step == pulse_time_step)
                      end
                      return function(context)
                        validate_context(context)
                        pulse_step = pulse_step + 2
                        pulse_time_step = pulse_time_step + 1
                        return true
                      end
                    end,
                    emit = function(context)
                      assert(context.beats_per_min == 120)
                      assert(context.beats_per_bar == 4)
                      assert(context.samples_per_sec == 44100)
                      local pulse_step, pulse_time_step = 1, 0.0 
                      local step = 1 
                      local function validate_context(context) 
                        assert(context.playback == "running")
                        assert(context.beats_per_min == 120)
                        assert(context.beats_per_bar == 4)
                        assert(context.samples_per_sec == 44100)
                        assert(context.pulse_step == pulse_step)
                        assert(context.pulse_time_step == pulse_time_step)
                        assert(context.step == step)
                      end
                      return function(context)
                        validate_context(context)
                        pulse_step = pulse_step + 2
                        pulse_time_step = pulse_time_step + 1
                        step = step + 1
                        return "c4"
                      end
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
        let mut beat_time_rhythm = beat_time_rhythm.unwrap();
        let event = beat_time_rhythm.next();
        assert_eq!(
            event,
            Some(RhythmIterItem {
                time: 0,
                event: Some(Event::NoteEvents(vec![Some(NoteEvent {
                    instrument: None,
                    note: Note::C4,
                    volume: 1.0,
                    panning: 0.0,
                    delay: 0.0
                })])),
                duration: 11025,
            })
        );
        assert!(beat_time_rhythm.next().unwrap().event.is_none());
        for _ in 0..10 {
            assert!(beat_time_rhythm.next().unwrap().event.is_some());
            assert!(beat_time_rhythm.next().unwrap().event.is_none());
        }
        Ok(())
    }

    #[test]
    fn second_time() -> LuaResult<()> {
        let (lua, _) = new_test_engine(130.0, 8, 48000)?;

        // SecondTimeRhythm
        let second_time_rhythm = lua
            .load(
                r#"
                rhythm {
                    unit = "seconds",
                    resolution = 2,
                    offset = 3,
                    pattern = {1,0,1,0},
                    emit = {"c5", "c5 v0.4", {"c7", "c7 v1.0"}}
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
        let mut second_time_rhythm = second_time_rhythm.unwrap();
        assert!((second_time_rhythm.step() - 2.0).abs() < f64::EPSILON);
        assert!((second_time_rhythm.offset() - 6.0).abs() < f64::EPSILON);
        let pattern = second_time_rhythm.pattern_mut();
        assert_eq!(
            vec![pattern.run(), pattern.run(), pattern.run(), pattern.run()],
            vec![
                Some(PulseIterItem {
                    value: 1.0,
                    step_time: 1.0,
                }),
                Some(PulseIterItem {
                    value: 0.0,
                    step_time: 1.0,
                }),
                Some(PulseIterItem {
                    value: 1.0,
                    step_time: 1.0,
                }),
                Some(PulseIterItem {
                    value: 0.0,
                    step_time: 1.0,
                })
            ]
        );
        Ok(())
    }

    #[test]
    fn second_time_callbacks() -> LuaResult<()> {
        let (lua, _) = new_test_engine(130.0, 8, 48000)?;

        // SecondTimeRhythm function Context
        let second_time_rhythm = lua
            .load(
                r#"
                return rhythm {
                    unit = "ms",
                    pattern = function(context)
                      return 1
                    end,
                    gate = function(context)
                      return true
                    end,
                    emit = function(context)
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
            Some(RhythmIterItem {
                time: 0,
                event: Some(Event::NoteEvents(vec![Some(NoteEvent {
                    instrument: None,
                    note: Note::C4,
                    volume: 1.0,
                    panning: 0.0,
                    delay: 0.0
                })],),),
                duration: 48
            })
        );
        Ok(())
    }
}
