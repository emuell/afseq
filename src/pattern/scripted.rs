use mlua::prelude::*;

use crate::{bindings::pattern_pulse_from_lua, Pattern};

// -------------------------------------------------------------------------------------------------

/// Pattern impl, which calls an existing lua script function to generate pulses.
#[derive(Debug, Clone)]
pub struct ScriptedPattern {
    environment: Option<LuaOwnedTable>,
    function: LuaOwnedFunction,
    pulse: f32,
}

impl ScriptedPattern {
    pub fn new(function: LuaFunction<'_>) -> LuaResult<Self> {
        // immediately fetch/evaluate the first event and get its environment, so we can immediately
        // show errors from the function and can reset the environment later on to this state.
        let result = function.call::<(), LuaValue>(())?;
        if let Some(inner_function) = result.as_function() {
            // function returned a function -> is an iterator. use inner function instead.
            let environment = inner_function.environment().map(|env| env.into_owned());
            let function = inner_function.clone().into_owned();
            let result = function.call::<(), LuaValue>(())?;
            let pulse = pattern_pulse_from_lua(result)?;
            Ok(Self {
                environment,
                function,
                pulse,
            })
        } else {
            // function returned a value. use this function as it is.
            let environment = function.environment().map(|env| env.into_owned());
            let function = function.into_owned();
            let pulse = pattern_pulse_from_lua(result)?;
            Ok(Self {
                environment,
                function,
                pulse,
            })
        }
    }

    fn next_pulse(&self) -> LuaResult<f32> {
        pattern_pulse_from_lua(self.function.call::<(), LuaValue>(())?)
    }
}

impl Pattern for ScriptedPattern {
    fn is_empty(&self) -> bool {
        false
    }

    fn len(&self) -> usize {
        0
    }

    fn run(&mut self) -> f32 {
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
        // fetch initial pulse value
        self.pulse = self.next_pulse().unwrap_or(0.0);
    }
}
