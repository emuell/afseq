use std::{borrow::Cow, collections::HashMap, fmt::Debug};

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

/// Add/signal a new Lua callback errors.
///
/// ### Panics
/// Panics if accessing the global lua callback error vector failed.
pub fn add_lua_callback_error(name: &str, err: &LuaError) {
    log::warn!("Lua callback '{}' failed to evaluate:\n{}", name, err);
    LUA_CALLBACK_ERRORS
        .write()
        .expect("Failed to lock Lua callback error vector")
        .push(err.clone());
}

// -------------------------------------------------------------------------------------------------

/// Playback state in LuaCallback context.
pub(crate) enum ContextPlaybackState {
    Seeking,
    Running,
}

impl ContextPlaybackState {
    fn into_bytes_string(self) -> &'static [u8] {
        match self {
            Self::Seeking => b"seeking",
            Self::Running => b"running",
        }
    }
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
/// can deal with them later, as appropriate.
///
/// By memorizing the original generator function and environment, it also can be reset to its
/// initial state by calling the original generator function again to fetch a new freshly
/// initialized function.
///
/// TODO: Upvalues of generators or simple functions could actually be collected and restored
/// too, but this uses debug functionality and may break some upvalues.
#[derive(Debug, Clone)]
pub(crate) struct LuaCallback {
    environment: Option<LuaOwnedTable>,
    context: LuaOwnedAnyUserData,
    generator: Option<LuaOwnedFunction>,
    function: LuaOwnedFunction,
    initialized: bool,
}

impl LuaCallback {
    /// Create a new Callback from an unowned lua function.
    pub fn new(lua: &Lua, function: LuaFunction) -> LuaResult<Self> {
        Self::with_owned(lua, function.into_owned())
    }

    /// Create a new Callback from an owned lua function.
    pub fn with_owned(lua: &Lua, function: LuaOwnedFunction) -> LuaResult<Self> {
        // create and register the callback context
        CallbackContext::register(lua)?;
        let context = lua
            .create_any_userdata(CallbackContext {
                values: HashMap::new(),
                external_values: HashMap::new(),
            })?
            .into_owned();
        // and memorize the function without calling it
        let environment = function.to_ref().environment().map(LuaTable::into_owned);
        let generator = None;
        let initialized = false;
        Ok(Self {
            environment,
            context,
            generator,
            function,
            initialized,
        })
    }

    /// Returns true if the callback is a generator.
    ///
    /// To test this, the callback must have run at least once, so it returns None if it never has.
    pub fn is_stateful(&self) -> Option<bool> {
        if self.initialized {
            Some(self.generator.is_some())
        } else {
            None
        }
    }

    /// Name of the inner function for errors. Usually will be an anonymous function.
    pub fn name(&self) -> String {
        self.function
            .to_ref()
            .info()
            .name
            .unwrap_or("anonymous function".to_string())
    }

    /// Sets the emitters playback state for the callback.
    pub fn set_context_playback_state(
        &mut self,
        playback_state: ContextPlaybackState,
    ) -> LuaResult<()> {
        let values = &mut self.context.borrow_mut::<CallbackContext>()?.values;
        values.insert(b"playback", playback_state.into_bytes_string().into());
        Ok(())
    }

    /// Sets the emitter time base context for the callback.
    pub fn set_context_time_base(&mut self, time_base: &BeatTimeBase) -> LuaResult<()> {
        let values = &mut self.context.borrow_mut::<CallbackContext>()?.values;
        values.insert(b"beats_per_min", time_base.beats_per_min.into());
        values.insert(b"beats_per_min", time_base.beats_per_min.into());
        values.insert(b"beats_per_bar", time_base.beats_per_bar.into());
        values.insert(b"samples_per_sec", time_base.samples_per_sec.into());
        Ok(())
    }

    /// Sets external emitter context for the callback.
    pub fn set_context_external_data(&mut self, data: &[(Cow<str>, f64)]) -> LuaResult<()> {
        let external_values = &mut self
            .context
            .borrow_mut::<CallbackContext>()?
            .external_values;
        for (key, value) in data {
            external_values.insert(key.to_string(), (*value).into());
        }
        Ok(())
    }

    /// Sets the pulse value emitter context for the callback.
    pub fn set_context_pulse_value(&mut self, pulse: PulseIterItem) -> LuaResult<()> {
        let values = &mut self.context.borrow_mut::<CallbackContext>()?.values;
        values.insert(b"pulse_value", pulse.value.into());
        values.insert(b"pulse_time", pulse.step_time.into());
        Ok(())
    }

    /// Sets the pulse step emitter context for the callback.
    pub fn set_context_pulse_step(
        &mut self,
        pulse_step: usize,
        pulse_time_step: f64,
    ) -> LuaResult<()> {
        let values = &mut self.context.borrow_mut::<CallbackContext>()?.values;
        values.insert(b"pulse_step", (pulse_step + 1).into());
        values.insert(b"pulse_time_step", pulse_time_step.into());
        Ok(())
    }

    /// Sets the step emitter context for the callback.
    pub fn set_context_step(&mut self, step: usize) -> LuaResult<()> {
        let values = &mut self.context.borrow_mut::<CallbackContext>()?.values;
        values.insert(b"step", (step + 1).into());
        Ok(())
    }

    /// Sets the cycle context step value for the callback.
    pub fn set_context_cycle_step(
        &mut self,
        channel: usize,
        step: usize,
        step_length: f64,
    ) -> LuaResult<()> {
        let values = &mut self.context.borrow_mut::<CallbackContext>()?.values;
        values.insert(b"channel", (channel + 1).into());
        values.insert(b"step", (step + 1).into());
        values.insert(b"step_length", step_length.into());
        Ok(())
    }

    /// Sets the emitter context for the callback.
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

    /// Sets the gate context for the callback.
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

    /// Sets the emitter context for the callback.
    pub fn set_emitter_context(
        &mut self,
        playback_state: ContextPlaybackState,
        time_base: &BeatTimeBase,
        pulse: PulseIterItem,
        pulse_step: usize,
        pulse_time_step: f64,
        step: usize,
    ) -> LuaResult<()> {
        self.set_context_playback_state(playback_state)?;
        self.set_gate_context(time_base, pulse, pulse_step, pulse_time_step)?;
        self.set_context_step(step)?;
        Ok(())
    }

    /// Sets the cycle context for the callback.
    pub fn set_cycle_context(
        &mut self,
        playback_state: ContextPlaybackState,
        time_base: &BeatTimeBase,
        channel: usize,
        step: usize,
        step_length: f64,
    ) -> LuaResult<()> {
        self.set_context_playback_state(playback_state)?;
        self.set_context_time_base(time_base)?;
        self.set_context_cycle_step(channel, step, step_length)?;
        Ok(())
    }

    /// Invoke the Lua function or generator and return its result as LuaValue.
    pub fn call(&mut self) -> LuaResult<LuaValue> {
        self.call_with_arg(LuaValue::Nil)
    }

    /// Invoke the Lua function or generator with an additional argument and return its result as LuaValue.
    pub fn call_with_arg<'lua, A: IntoLua<'lua> + Clone>(
        &'lua mut self,
        arg: A,
    ) -> LuaResult<LuaValue<'lua>> {
        if self.initialized {
            self.function.call((&self.context, arg))
        } else {
            self.initialized = true;
            let result = {
                // HACK: don't borrow self here, so we can borrow mut again to assign the generator function
                // see https://stackoverflow.com/questions/73641155/how-to-force-rust-to-drop-a-mutable-borrow
                let function = unsafe { &*(&self.function as *const LuaOwnedFunction) };
                function.call::<_, LuaValue>((&self.context, arg.clone()))?
            };
            if let Some(inner_function) = result.as_function().cloned().map(|f| f.into_owned()) {
                // function returned a function -> is a generator. use the inner function instead.
                let environment = self
                    .function
                    .to_ref()
                    .environment()
                    .map(LuaTable::into_owned);
                self.environment = environment;
                self.generator = Some(std::mem::replace(&mut self.function, inner_function));
                self.function.call::<_, LuaValue>((&self.context, arg))
            } else {
                // function returned some value. use this function directly.
                self.environment = None;
                self.generator = None;
                Ok(result)
            }
        }
    }

    /// Report a Lua callback errors. The error will be logged and usually cleared after
    /// the next callback call.
    pub fn handle_error(&self, err: &LuaError) {
        add_lua_callback_error(&self.name(), err)
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
                    .call::<_, LuaValue>(&self.context)?;
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

// -------------------------------------------------------------------------------------------------

/// Memorizes an optional set of values that are passed along as context with the callback.
#[derive(Debug, Clone)]
struct CallbackContext {
    values: HashMap<&'static [u8], ContextValue>,
    external_values: HashMap<String, ContextValue>,
}

impl CallbackContext {
    fn register(lua: &Lua) -> LuaResult<()> {
        // NB: registering for a specific engine is faster than implementing the UserData impl
        // See https://github.com/mlua-rs/mlua/discussions/283
        lua.register_userdata_type::<CallbackContext>(|reg| {
            reg.add_meta_field_with("__index", |lua| {
                lua.create_function(
                    |lua, (this, key): (LuaUserDataRef<CallbackContext>, LuaString)| {
                        if let Some(value) = this.values.get(key.as_bytes()) {
                            // fast path (string bytes)
                            value.into_lua(lua)
                        } else if let Some(value) =
                            this.external_values.get(key.to_string_lossy().as_ref())
                        {
                            // slower path (string )
                            value.into_lua(lua)
                        } else {
                            Err(mlua::Error::RuntimeError(format!(
                                "undefined field '{}' in context",
                                key.to_string_lossy()
                            )))
                        }
                    },
                )
            })
        })
    }
}

// -------------------------------------------------------------------------------------------------

/// A to lua convertible value within a CallbackContext
#[derive(Debug, Copy, Clone, PartialEq)]
enum ContextValue {
    Number(LuaNumber),
    String(&'static [u8]),
}

impl<'lua> IntoLua<'lua> for &ContextValue {
    fn into_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        match *self {
            ContextValue::Number(num) => Ok(LuaValue::Number(num)),
            ContextValue::String(str) => Ok(LuaValue::String(lua.create_string(str)?)),
        }
    }
}

impl From<&'static [u8]> for ContextValue {
    fn from(val: &'static [u8]) -> Self {
        ContextValue::String(val)
    }
}

macro_rules! context_value_from_number_impl {
    ($type:ty) => {
        impl From<$type> for ContextValue {
            fn from(val: $type) -> Self {
                ContextValue::Number(val as LuaNumber)
            }
        }
    };
}

context_value_from_number_impl!(i32);
context_value_from_number_impl!(u32);
context_value_from_number_impl!(i64);
context_value_from_number_impl!(u64);
context_value_from_number_impl!(usize);
context_value_from_number_impl!(f32);
context_value_from_number_impl!(f64);

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
