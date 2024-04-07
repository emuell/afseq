use std::{borrow::Cow, fmt::Debug};

use mlua::prelude::*;

use super::LuaCallback;
use crate::{time::BeatTimeBase, PulseIterItem};

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
pub(crate) struct LuaFunctionCallback {
    environment: Option<LuaOwnedTable>,
    context: LuaOwnedTable,
    generator: Option<LuaOwnedFunction>,
    function: LuaOwnedFunction,
    initialized: bool,
}

impl LuaFunctionCallback {
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
}

impl LuaCallback for LuaFunctionCallback {
    fn set_context_external_data(&mut self, data: &[(Cow<str>, f64)]) -> LuaResult<()> {
        let table = self.context.to_ref();
        for (key, value) in data {
            table.raw_set(key as &str, *value)?;
        }
        Ok(())
    }

    fn set_context_time_base(&mut self, time_base: &BeatTimeBase) -> LuaResult<()> {
        let table = self.context.to_ref();
        table.raw_set("beats_per_min", time_base.beats_per_min)?;
        table.raw_set("beats_per_bar", time_base.beats_per_bar)?;
        table.raw_set("sample_rate", time_base.samples_per_sec)?;
        Ok(())
    }

    fn set_context_pulse_step(
        &mut self,
        pulse_step: usize,
        pulse_time_step: f64,
        pulse_pattern_length: usize,
    ) -> LuaResult<()> {
        let table = self.context.to_ref();
        table.raw_set("pulse_step", pulse_step + 1)?;
        table.raw_set("pulse_time_step", pulse_time_step)?;
        table.raw_set("pattern_length", pulse_pattern_length)?;
        table.raw_set("pattern_pulse_step", pulse_step % pulse_pattern_length + 1)?;
        Ok(())
    }

    fn set_context_step(&mut self, step: usize) -> LuaResult<()> {
        let table = self.context.to_ref();
        table.raw_set("step", step + 1)?;
        Ok(())
    }

    fn set_context_pulse_value(&mut self, pulse: PulseIterItem) -> LuaResult<()> {
        let table = self.context.to_ref();
        table.raw_set("pulse_value", pulse.value)?;
        table.raw_set("pulse_time", pulse.step_time)?;
        Ok(())
    }

    fn name(&self) -> String {
        self.function
            .to_ref()
            .info()
            .name
            .unwrap_or("annonymous function".to_string())
    }

    fn call(&mut self) -> LuaResult<Option<LuaValue>> {
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
        Ok(Some(self.function.call(self.context.to_ref())?))
    }

    fn reset(&mut self) -> LuaResult<()> {
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

    fn duplicate(&self) -> Box<dyn LuaCallback> {
        Box::new(self.clone())
    }
}
