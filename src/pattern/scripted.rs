use mlua::prelude::*;

use crate::{
    bindings::{initialize_emitter_context, pattern_pulse_from_lua, LuaTimeoutHook},
    BeatTimeBase, Pattern,
};

// -------------------------------------------------------------------------------------------------

/// Pattern impl, which calls an existing lua script function to generate pulses.
#[derive(Debug, Clone)]
pub struct ScriptedPattern {
    environment: Option<LuaOwnedTable>,
    timeout_hook: LuaTimeoutHook,
    function: LuaOwnedFunction,
    function_context: LuaOwnedTable,
    step_count: usize,
    pulse: f32,
}

impl ScriptedPattern {
    pub fn new(
        lua: &Lua,
        timeout_hook: &LuaTimeoutHook,
        function: LuaFunction<'_>,
        time_base: &BeatTimeBase,
    ) -> LuaResult<Self> {
        // create a new function context
        let step_count = 0;
        let mut function_context = lua.create_table()?.into_owned();
        initialize_emitter_context(&mut function_context, step_count, time_base)?;
        // create a new timeout_hook instance and reset it before calling the function
        let mut timeout_hook = timeout_hook.clone();
        timeout_hook.reset();
        // immediately fetch/evaluate the first event and get its environment, so we can immediately
        // show errors from the function and can reset the environment later on to this state.
        let result = function.call::<_, LuaValue>(function_context.to_ref())?;
        if let Some(inner_function) = result.as_function() {
            // function returned a function -> is an iterator. use inner function instead.
            let function_context = function_context.clone();
            let environment = inner_function.environment().map(|env| env.into_owned());
            let function = inner_function.clone().into_owned();
            let result = function.call::<_, LuaValue>(function_context.to_ref())?;
            let pulse = pattern_pulse_from_lua(result)?;
            Ok(Self {
                environment,
                timeout_hook,
                function,
                function_context,
                step_count,
                pulse,
            })
        } else {
            // function returned a value. use this function as it is.
            let environment = function.environment().map(|env| env.into_owned());
            let function = function.into_owned();
            let pulse = pattern_pulse_from_lua(result)?;
            Ok(Self {
                environment,
                timeout_hook,
                function,
                function_context,
                step_count,
                pulse,
            })
        }
    }

    fn next_pulse(&mut self) -> LuaResult<f32> {
        // reset timeout
        self.timeout_hook.reset();
        // update context
        self.step_count += 1;
        self.function_context
            .to_ref()
            .raw_set("step", self.step_count + 1)?;
        // call function with context and evaluate the result
        let result = self
            .function
            .call::<_, LuaValue>(self.function_context.to_ref())?;
        pattern_pulse_from_lua(result)
    }
}

impl Pattern for ScriptedPattern {
    fn is_empty(&self) -> bool {
        // not empty
        false
    }

    fn len(&self) -> usize {
        // unknown length
        0
    }

    fn run(&mut self) -> f32 {
        // generate a new pulse
        let pulse = self.pulse;
        self.pulse = match self.next_pulse() {
            Ok(pulse) => pulse,
            Err(err) => {
                self.pulse = 0.0;
                log::warn!(
                    "Failed to run custom pattern func '{}': {}",
                    self.function
                        .to_ref()
                        .info()
                        .name
                        .unwrap_or("annonymous function".to_string()),
                    err
                );
                0.0
            }
        };
        pulse
    }

    fn update_time_base(&mut self, time_base: &BeatTimeBase) {
        // update function context from the new time base
        if let Err(err) =
            initialize_emitter_context(&mut self.function_context, self.step_count, time_base)
        {
            log::warn!(
                "Failed to update event iter context for custom function '{}': {}",
                self.function
                    .to_ref()
                    .info()
                    .name
                    .unwrap_or("annonymous function".to_string()),
                err
            );
        }
    }

    fn reset(&mut self) {
        // restore function environment
        if let Some(env) = &self.environment {
            if let Err(err) = self.function.to_ref().set_environment(env.to_ref()) {
                log::warn!(
                    "Failed to restore custom pattern func environment '{}': {}",
                    self.function
                        .to_ref()
                        .info()
                        .name
                        .unwrap_or("annonymous function".to_string()),
                    err
                );
            }
        }
        // reset step counter
        self.step_count = 0;
        // and set new initial pulse value, discarding the last one
        let _ = self.run();
    }
}
