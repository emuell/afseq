use std::{borrow::Cow, fmt::Debug};

use mlua::prelude::*;

use lazy_static::lazy_static;
use std::sync::RwLock;

use crate::{time::BeatTimeBase, PulseIterItem};

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

/// Lazily evaluates a lua function the first time it's called, to either use it as a iterator,
/// a function which returns a function, or directly as it is.
///
/// When calling the function the signature of the function is `fn(context): LuaResult`;
/// The passed context is created as an empty table with the callback, and should be filled up
/// with values before it's called.
///
/// Errors from callbacks should be handled by calling `self.handle_error` so external clients
/// can deal with them later, as apropriate.
///
/// By memorizing the original generator function and environment, it also can be reset to its
/// initial state by calling the original generator function again to fetch a new freshly
/// initialized function.
///
/// TODO: Upvalues of generators or simple functions could actuially be collected and restored
/// too, but this uses debug functionality and may break some upvalues.
#[derive(Debug, Clone)]
pub(crate) struct LuaCallback {
    environment: Option<LuaOwnedTable>,
    context: LuaOwnedTable,
    generator: Option<LuaOwnedFunction>,
    function: LuaOwnedFunction,
    initialized: bool,
}

impl LuaCallback {
    pub fn new(lua: &Lua, function: LuaFunction) -> LuaResult<Self> {
        // create an empty context and memorize the function without calling it
        let context = lua.create_table()?.into_owned();
        let environment = function.environment().map(LuaTable::into_owned);
        let generator = None;
        let function = function.into_owned();
        let initialized = false;
        Ok(Self {
            environment,
            context,
            generator,
            function,
            initialized,
        })
    }

    /// Sets the emitter time base context for the callback.
    pub fn set_context_time_base(&mut self, time_base: &BeatTimeBase) -> LuaResult<()> {
        let table = self.context.to_ref();
        table.raw_set("beats_per_min", time_base.beats_per_min)?;
        table.raw_set("beats_per_bar", time_base.beats_per_bar)?;
        table.raw_set("samples_per_sec", time_base.samples_per_sec)?;
        Ok(())
    }

    /// Sets external emitter context for the callback.
    pub fn set_context_external_data(&mut self, data: &[(Cow<str>, f64)]) -> LuaResult<()> {
        let table = self.context.to_ref();
        for (key, value) in data {
            table.raw_set(key as &str, *value)?;
        }
        Ok(())
    }

    /// Sets the pulse value emitter context for the callback.
    pub fn set_context_pulse_value(&mut self, pulse: PulseIterItem) -> LuaResult<()> {
        let table = self.context.to_ref();
        table.raw_set("pulse_value", pulse.value)?;
        table.raw_set("pulse_time", pulse.step_time)?;
        Ok(())
    }

    /// Sets the pulse step emitter context for the callback.
    pub fn set_context_pulse_step(
        &mut self,
        pulse_step: usize,
        pulse_time_step: f64,
    ) -> LuaResult<()> {
        let table = self.context.to_ref();
        table.raw_set("pulse_step", pulse_step + 1)?;
        table.raw_set("pulse_time_step", pulse_time_step)?;
        Ok(())
    }

    /// Sets the step emitter context for the callback.
    pub fn set_context_step(&mut self, step: usize) -> LuaResult<()> {
        let table = self.context.to_ref();
        table.raw_set("step", step + 1)?;
        Ok(())
    }

    /// Sets the emitter context for the callback. Only used for function callbacks.
    pub fn set_pattern_context(
        &mut self,
        time_base: &BeatTimeBase,
        pulse_step: usize,
        pulse_time_step: f64,
    ) -> LuaResult<()> {
        self.set_context_time_base(time_base)?;
        self.set_context_pulse_step(pulse_step, pulse_time_step)?;
        Ok(())
    }

    /// Sets the gate context for the callback. Only used for function callbacks.
    pub fn set_gate_context(
        &mut self,
        time_base: &BeatTimeBase,
        pulse: PulseIterItem,
        pulse_step: usize,
        pulse_time_step: f64,
    ) -> LuaResult<()> {
        self.set_pattern_context(time_base, pulse_step, pulse_time_step)?;
        self.set_context_pulse_value(pulse)?;
        Ok(())
    }

    /// Sets the emitter context for the callback. Only used for function callbacks.
    pub fn set_emitter_context(
        &mut self,
        time_base: &BeatTimeBase,
        pulse: PulseIterItem,
        pulse_step: usize,
        pulse_time_step: f64,
        step: usize,
    ) -> LuaResult<()> {
        self.set_gate_context(
            time_base,
            pulse,
            pulse_step,
            pulse_time_step,
        )?;
        self.set_context_step(step)?;
        Ok(())
    }

    /// Name of the inner function for errors. Usually will be an annonymous function.
    pub fn name(&self) -> String {
        self.function
            .to_ref()
            .info()
            .name
            .unwrap_or("annonymous function".to_string())
    }

    /// Invoke the Lua function callback or generator.
    pub fn call(&mut self) -> LuaResult<LuaValue> {
        if !self.initialized {
            self.initialized = true;
            let function = self.function.clone();
            let result = function.call::<_, LuaValue>(self.context.to_ref())?;
            if let Some(inner_function) = result.as_function() {
                // function returned a function -> is an iterator. use inner function instead.
                let function_environment = self
                    .function
                    .to_ref()
                    .environment()
                    .map(LuaTable::into_owned);
                let function_generator = Some(self.function.clone());
                self.environment = function_environment;
                self.generator = function_generator;
                self.function = inner_function.clone().into_owned();
            } else {
                // function returned not a function. use this function directly.
                self.environment = None;
                self.generator = None;
            }
        }
        self.function.call(self.context.to_ref())
    }

    /// Report a Lua callback errors. The error will be logged and usually cleared after
    /// the next callback call.
    pub fn handle_error(&self, err: &LuaError) {
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
    pub fn reset(&mut self) -> LuaResult<()> {
        // resetting only is necessary when we got initialized
        if self.initialized {
            if let Some(function_generator) = &self.generator {
                // restore generator environment
                if let Some(env) = &self.environment {
                    function_generator.to_ref().set_environment(env.to_ref())?;
                }
                // then fetch a new fresh function from the generator
                let value = function_generator
                    .to_ref()
                    .call::<_, LuaValue>(self.context.to_ref())?;
                if let Some(function) = value.as_function() {
                    self.function = function.clone().into_owned();
                } else {
                    return Err(LuaError::runtime(format!(
                        "Failed to reset custom generator function '{}' \
                         Expected a function as return value, got a '{}'",
                        self.name(),
                        value.type_name()
                    )));
                }
            }
        }
        Ok(())
    }
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
    fn callbacks() -> LuaResult<()> {
        let (lua, _) = new_test_engine(120.0, 4, 44100)?;

        let rhythm = lua
            .load(
                r#"
                return rhythm {
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
}
