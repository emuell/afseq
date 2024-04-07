use std::{borrow::Cow, fmt::Debug};

use mlua::prelude::*;

use lazy_static::lazy_static;
use std::sync::RwLock;

use self::{function::LuaFunctionCallback, generator::LuaGeneratorCallback};
use crate::{time::BeatTimeBase, PulseIterItem};

// -------------------------------------------------------------------------------------------------

mod function;
mod generator;

// -------------------------------------------------------------------------------------------------

lazy_static! {
    static ref LUA_CALLBACK_ERRORS: RwLock<Vec<LuaError>> = Vec::new().into();
}

/// Returns some error if there are any Lua callback errors, with the !first! error that happened.
/// Use `lua_callback_errors` to get fetch all errors since the errors got cleared.
///
/// ### Panics
/// Panics if accessing the global lua callback error vector fails.
pub fn has_lua_callback_errors() -> Option<LuaError> {
    LUA_CALLBACK_ERRORS
        .read()
        .expect("Failed to lock Lua callback error vector")
        .first()
        .cloned()
}

/// Returns all Lua callback errors, if any. Check with `has_lua_callback_errors()` to avoid
/// possible vec clone overhead, if that's relevant.
///
/// ### Panics
/// Panics if accessing the global lua callback error vector failed.
pub fn lua_callback_errors() -> Vec<LuaError> {
    LUA_CALLBACK_ERRORS
        .read()
        .expect("Failed to lock Lua callback error vector")
        .clone()
}

/// Clears all Lua callback errors.
///
/// ### Panics
/// Panics if accessing the global lua callback error vector failed.
pub fn clear_lua_callback_errors() {
    LUA_CALLBACK_ERRORS
        .write()
        .expect("Failed to lock Lua callback error vector")
        .clear();
}

// -------------------------------------------------------------------------------------------------

/// Construct [`LuaCallback`] instances from a Lua function or fun generator.
pub(crate) struct LuaCallbackFactory {}

impl LuaCallbackFactory {
    /// Checks whether the table looks like a fun generator.
    pub fn is_fun_generator(table: &LuaTable) -> bool {
        table
            .contains_key("gen")
            .and(table.contains_key("param"))
            .and(table.contains_key("state"))
            .unwrap_or(false)
    }

    /// Tries creating a new Lua callback from a Lua function.
    pub fn from_function(lua: &Lua, function: LuaFunction) -> LuaResult<Box<dyn LuaCallback>> {
        Ok(Box::new(LuaFunctionCallback::new(lua, function)?))
    }

    /// Tries creating a new Lua callback from a fun generator table.
    pub fn from_fun_generator(lua: &Lua, table: LuaTable) -> LuaResult<Box<dyn LuaCallback>> {
        Ok(Box::new(LuaGeneratorCallback::new(lua, table)?))
    }
}

// -------------------------------------------------------------------------------------------------

/// A Lua callback with an optional context, to generate events in rhythms or patterns from
/// a lua function or fun iterator.
///
/// See [`LuaFunctionCallback`] and [`LuaGeneratorCallback`] for more information.
///
/// Use [`LuaCallbackFactory`] to create instances of this trait.
pub(crate) trait LuaCallback: Debug {
    /// Sets the emitter time base context for the callback.
    fn set_context_time_base(&mut self, time_base: &BeatTimeBase) -> LuaResult<()>;

    /// Sets external emitter context for the callback.
    fn set_context_external_data(&mut self, data: &[(Cow<str>, f64)]) -> LuaResult<()>;

    /// Sets the pulse value emitter context for the callback.
    fn set_context_pulse_value(&mut self, pulse: PulseIterItem) -> LuaResult<()>;

    /// Sets the pulse step emitter context for the callback.
    fn set_context_pulse_step(
        &mut self,
        pulse_step: usize,
        pulse_time_step: f64,
        pulse_pattern_length: usize,
    ) -> LuaResult<()>;

    /// Sets the step emitter context for the callback.
    fn set_context_step(&mut self, step: usize) -> LuaResult<()>;

    /// Sets the emitter context for the callback. Only used for function callbacks.
    fn set_pattern_context(
        &mut self,
        time_base: &BeatTimeBase,
        pulse_step: usize,
        pulse_time_step: f64,
        pulse_pattern_length: usize,
    ) -> LuaResult<()> {
        self.set_context_time_base(time_base)?;
        self.set_context_pulse_step(pulse_step, pulse_time_step, pulse_pattern_length)?;
        Ok(())
    }

    /// Sets the emitter context for the callback. Only used for function callbacks.
    fn set_emitter_context(
        &mut self,
        time_base: &BeatTimeBase,
        pulse: PulseIterItem,
        pulse_step: usize,
        pulse_time_step: f64,
        pulse_pattern_length: usize,
        step: usize,
    ) -> LuaResult<()> {
        self.set_pattern_context(time_base, pulse_step, pulse_time_step, pulse_pattern_length)?;
        self.set_context_step(step)?;
        self.set_context_pulse_value(pulse)?;
        Ok(())
    }

    /// Name of the inner function for errors. Usually will be an annonymous function.
    fn name(&self) -> String;

    /// Invoke the Lua function callback or generator.
    /// Returns `None` for iterators, when the iteration finished.
    fn call(&mut self) -> LuaResult<Option<LuaValue>>;

    /// Report a Lua callback errors. The error will be logged and usually cleared after
    /// the next callback call.
    fn handle_error(&self, err: &LuaError) {
        log::warn!(
            "Lua callback '{}' failed to evaluate:\n{}",
            self.name(),
            err
        );
        LUA_CALLBACK_ERRORS
            .write()
            .expect("Failed to lock Lua callback error vector")
            .push(err.clone());
    }

    /// Reset the callback function or iterator to its initial state.
    fn reset(&mut self) -> LuaResult<()>;

    /// Create a new cloned instance of this callback. This actualy is a clone(), wrapped into
    /// a `Box<dyn LuaCallback>`, but called 'duplicate' to avoid conflicts with Clone trait impls.
    fn duplicate(&self) -> Box<dyn LuaCallback>;
}

// --------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use std::borrow::BorrowMut;

    use super::*;
    use crate::{bindings::*, Event, Note, RhythmIterItem};

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
    fn function() -> LuaResult<()> {
        let (lua, _) = new_test_engine(120.0, 4, 44100)?;

        let rhythm = lua
            .load(
                r#"
                require "fun"()
                return emitter {
                    unit = "seconds",
                    pattern = function(context) 
                      return (context.pulse_step == 2) and 0 or 1 
                    end,
                    emit = function(context)
                      local notes = {"c4", "d#4", "g4"}
                      local step = 1
                      return function(context) 
                        local note = notes[step - 1 % #notes + 1]
                        step = step + 1
                        return note
                      end
                    end
                }
            "#,
            )
            .eval::<LuaValue>()?;

        let mut rhythm = rhythm
            .as_userdata()
            .unwrap()
            .borrow_mut::<SecondTimeRhythm>()?;
        let rhythm = rhythm.borrow_mut();
        for _ in 0..2 {
            let events = rhythm.clone().take(4).collect::<Vec<_>>();
            rhythm.reset();
            assert_eq!(
                events,
                vec![
                    RhythmIterItem {
                        event: Some(Event::NoteEvents(vec![Some((Note::C4).into())])),
                        time: 0,
                        duration: 44100
                    },
                    RhythmIterItem {
                        time: 44100,
                        event: None,
                        duration: 44100
                    },
                    RhythmIterItem {
                        time: 88200,
                        event: Some(Event::NoteEvents(vec![Some((Note::Ds4).into())])),
                        duration: 44100
                    },
                    RhythmIterItem {
                        time: 132300,
                        event: Some(Event::NoteEvents(vec![Some((Note::G4).into())])),
                        duration: 44100
                    }
                ]
            );
        }
        Ok(())
    }

    #[test]
    fn generator() -> LuaResult<()> {
        let (lua, _) = new_test_engine(120.0, 4, 44100)?;

        let rhythm = lua
            .load(
                r#"
                require "fun"()
                return emitter {
                    unit = "seconds",
                    pattern = iter{1,0,1}:map(function(x) return x end):cycle():take(8),
                    emit = iter{"c4", "d#4", "g4"}:cycle()
                }
            "#,
            )
            .eval::<LuaValue>()?;

        let mut rhythm = rhythm
            .as_userdata()
            .unwrap()
            .borrow_mut::<SecondTimeRhythm>()?;
        let rhythm = rhythm.borrow_mut();
        for _ in 0..2 {
            let events = rhythm.clone().take(4).collect::<Vec<_>>();
            rhythm.reset();
            assert_eq!(
                events,
                vec![
                    RhythmIterItem {
                        event: Some(Event::NoteEvents(vec![Some((Note::C4).into())])),
                        time: 0,
                        duration: 44100
                    },
                    RhythmIterItem {
                        time: 44100,
                        event: None,
                        duration: 44100
                    },
                    RhythmIterItem {
                        time: 88200,
                        event: Some(Event::NoteEvents(vec![Some((Note::Ds4).into())])),
                        duration: 44100
                    },
                    RhythmIterItem {
                        time: 132300,
                        event: Some(Event::NoteEvents(vec![Some((Note::G4).into())])),
                        duration: 44100
                    }
                ]
            );
        }
        Ok(())
    }
}
