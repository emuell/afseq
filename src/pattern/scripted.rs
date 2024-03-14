use std::{cell::RefCell, rc::Rc};

use mlua::prelude::*;

use crate::{
    bindings::{
        initialize_context_pulse_count, initialize_context_time_base, initialize_pattern_context,
        pattern_pulse_from_lua, LuaTimeoutHook,
    },
    BeatTimeBase, Pattern, Pulse, PulseIter, PulseIterItem,
};

// -------------------------------------------------------------------------------------------------

fn function_name(function: &LuaOwnedFunction) -> String {
    function
        .to_ref()
        .info()
        .name
        .unwrap_or("annonymous function".to_string())
}

// -------------------------------------------------------------------------------------------------

/// Pattern impl, which calls an existing lua script function to generate pulses.
#[derive(Debug, Clone)]
pub struct ScriptedPattern {
    timeout_hook: LuaTimeoutHook,
    function_environment: Option<LuaOwnedTable>,
    function_generator: Option<LuaOwnedFunction>,
    function_context: LuaOwnedTable,
    function: LuaOwnedFunction,
    pulse_count: usize,
    pulse_time_count: f64,
    pulse: Pulse,
    pulse_iter: Option<PulseIter>,
    last_emitted_pulse: PulseIterItem,
}

impl ScriptedPattern {
    pub(crate) fn new(
        lua: &Lua,
        timeout_hook: &LuaTimeoutHook,
        function: LuaFunction<'_>,
        time_base: &BeatTimeBase,
    ) -> LuaResult<Self> {
        // create a new function context
        let pulse_count = 0;
        let pulse_time_count = 0.0;
        let mut function_context = lua.create_table()?.into_owned();
        initialize_pattern_context(
            &mut function_context,
            time_base,
            pulse_count,
            pulse_time_count,
        )?;
        // create a new timeout_hook instance and reset it before calling the function
        let mut timeout_hook = timeout_hook.clone();
        timeout_hook.reset();
        // immediately fetch/evaluate the first event and get its environment, so we can immediately
        // show errors from the function and can reset the environment later on to this state.
        let result = function.call::<_, LuaValue>(function_context.to_ref())?;
        if let Some(inner_function) = result.as_function() {
            // function returned a function -> is an iterator. use inner function instead.
            let function_context = function_context.clone();
            let function_environment = function.environment().map(|env| env.into_owned());
            let function_generator = Some(function.into_owned());
            let function = inner_function.clone().into_owned();
            let pulse =
                pattern_pulse_from_lua(function.call::<_, LuaValue>(function_context.to_ref())?)?;
            let pulse_iter = Some(pulse.clone().into_iter());
            let last_emitted_pulse = pulse.clone().into_iter().next().unwrap_or_default();
            Ok(Self {
                timeout_hook,
                function_environment,
                function_generator,
                function_context,
                function,
                pulse_count,
                pulse_time_count,
                pulse,
                pulse_iter,
                last_emitted_pulse,
            })
        } else {
            // function returned an event. use this function.
            let function_environment = None;
            let function_generator = None;
            let function = function.into_owned();
            let pulse = pattern_pulse_from_lua(result)?;
            let pulse_iter = Some(pulse.clone().into_iter());
            let last_emitted_pulse = pulse.clone().into_iter().next().unwrap_or_default();
            Ok(Self {
                timeout_hook,
                function_environment,
                function_generator,
                function_context,
                function,
                pulse_count,
                pulse_time_count,
                pulse,
                pulse_iter,
                last_emitted_pulse,
            })
        }
    }

    fn next_pulse(&mut self, move_step: bool) -> LuaResult<Pulse> {
        // reset timeout
        self.timeout_hook.reset();
        // move step counter and update context
        if move_step {
            self.pulse_count += 1;
            self.pulse_time_count += self.last_emitted_pulse.step_time;
            initialize_context_pulse_count(
                &mut self.function_context,
                self.pulse_count,
                self.pulse_time_count,
            )?;
        }
        // call function with context and evaluate the result
        let result = self
            .function
            .call::<_, LuaValue>(self.function_context.to_ref())?;
        pattern_pulse_from_lua(result)
    }
}

impl Pattern for ScriptedPattern {
    fn is_empty(&self) -> bool {
        false
    }

    fn len(&self) -> usize {
        if let Some(pulse_iter) = &self.pulse_iter {
            pulse_iter.len()
        } else {
            1
        }
    }

    fn peek(&self) -> PulseIterItem {
        if let Some(mut pulse_iter) = self.pulse_iter.clone() {
            pulse_iter.next().unwrap_or_default()
        } else {
            PulseIterItem::default()
        }
    }

    fn run(&mut self) -> PulseIterItem {
        // if we have a pulse iterator, consume it
        if let Some(pulse_iter) = &mut self.pulse_iter {
            if let Some(pulse) = pulse_iter.next() {
                self.last_emitted_pulse = pulse;
                return pulse;
            } else {
                self.pulse_iter = None;
            }
        }
        // else move on and generate a new pulse
        let move_step = true;
        self.pulse = match self.next_pulse(move_step) {
            Ok(pulse) => pulse,
            Err(err) => {
                log::warn!(
                    "Failed to run custom pattern func '{}': {}",
                    function_name(&self.function),
                    err
                );
                Pulse::from(0.0)
            }
        };
        let mut pulse_iter = self.pulse.clone().into_iter();
        let pulse = pulse_iter.next().unwrap_or(PulseIterItem::default());
        self.last_emitted_pulse = pulse;
        self.pulse_iter = Some(pulse_iter);
        pulse
    }

    fn set_time_base(&mut self, time_base: &BeatTimeBase) {
        // update function context from the new time base
        if let Err(err) = initialize_context_time_base(&mut self.function_context, time_base) {
            log::warn!(
                "Failed to update context for custom pattern function '{}': {}",
                function_name(&self.function),
                err
            );
        }
    }

    fn duplicate(&self) -> Rc<RefCell<dyn Pattern>> {
        Rc::new(RefCell::new(self.clone()))
    }

    fn reset(&mut self) {
        // reset timeout
        self.timeout_hook.reset();
        // reset step counter
        self.pulse_count = 0;
        self.pulse_time_count = 0.0;
        // update step in context
        if let Err(err) = initialize_context_pulse_count(
            &mut self.function_context,
            self.pulse_count,
            self.pulse_time_count,
        ) {
            log::warn!(
                "Failed to update context for custom pattern function '{}': {}",
                function_name(&self.function),
                err
            );
        }
        // restore generator function environment
        if let Some(function_generator) = &self.function_generator {
            if let Some(env) = &self.function_environment {
                if let Err(err) = function_generator.to_ref().set_environment(env.to_ref()) {
                    log::warn!(
                        "Failed to restore custom pattern emitter func environment '{}': {}",
                        function_name(function_generator),
                        err
                    );
                }
            }
            // then fetch a new fresh function from the generator
            let result = function_generator
                .to_ref()
                .call::<_, LuaValue>(self.function_context.to_ref());
            match result {
                Err(err) => {
                    log::warn!(
                        "Failed to call custom pattern emitter generator function '{}': {}",
                        function_name(function_generator),
                        err
                    );
                }
                Ok(value) => {
                    if let Some(function) = value.as_function() {
                        self.function = function.clone().into_owned();
                    } else {
                        log::warn!(
                            "Failed to call custom pattern emitter generator function '{}': {}",
                            function_name(function_generator),
                            "Generator does not return a valid emitter function"
                        );
                    }
                }
            };
        }
        // and set new initial pulse value
        let move_step = false;
        self.pulse = match self.next_pulse(move_step) {
            Ok(pulse) => pulse,
            Err(err) => {
                log::warn!(
                    "Failed to run custom pattern func '{}': {}",
                    function_name(&self.function),
                    err
                );
                Pulse::from(0.0)
            }
        };
        self.pulse_iter = Some(self.pulse.clone().into_iter());
        self.last_emitted_pulse = self.pulse.clone().into_iter().next().unwrap_or_default();
    }
}
