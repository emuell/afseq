use std::{cell::RefCell, rc::Rc};

use mlua::prelude::*;

use crate::{
    bindings::{
        callback::LuaFunctionCallback, initialize_context_pulse_count,
        initialize_context_time_base, initialize_pattern_context, pattern_pulse_from_lua,
        timeout::LuaTimeoutHook,
    },
    BeatTimeBase, Pattern, Pulse, PulseIter, PulseIterItem,
};

// -------------------------------------------------------------------------------------------------

/// Pattern impl, which calls an existing lua script function to generate pulses.
#[derive(Debug, Clone)]
pub struct ScriptedPattern {
    timeout_hook: LuaTimeoutHook,
    function: LuaFunctionCallback,
    pulse_count: usize,
    pulse_time_count: f64,
    pulse: Option<Pulse>,
    pulse_iter: Option<PulseIter>,
}

impl ScriptedPattern {
    pub(crate) fn new(
        lua: &Lua,
        timeout_hook: &LuaTimeoutHook,
        function: LuaFunction<'_>,
        time_base: &BeatTimeBase,
    ) -> LuaResult<Self> {
        // create a new timeout_hook instance and reset it before calling the function
        let mut timeout_hook = timeout_hook.clone();
        timeout_hook.reset();
        // create a new function
        let mut function = LuaFunctionCallback::new(lua, function)?;
        // initialize function context
        let pulse_count = 0;
        let pulse_time_count = 0.0;
        initialize_pattern_context(function.context(), time_base, pulse_count, pulse_time_count)?;
        let pulse = None;
        let pulse_iter = None;
        Ok(Self {
            timeout_hook,
            function,
            pulse_count,
            pulse_time_count,
            pulse,
            pulse_iter,
        })
    }

    fn next_pulse(&mut self) -> LuaResult<Pulse> {
        // reset timeout
        self.timeout_hook.reset();
        // update context
        initialize_context_pulse_count(
            self.function.context(),
            self.pulse_count,
            self.pulse_time_count,
        )?;
        // call function with context and evaluate the result
        pattern_pulse_from_lua(self.function.call()?)
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

    fn run(&mut self) -> PulseIterItem {
        // if we have a pulse iterator, consume it
        if let Some(pulse_iter) = &mut self.pulse_iter {
            if let Some(pulse) = pulse_iter.next() {
                // move step for the next iter call
                self.pulse_count += 1;
                self.pulse_time_count += pulse.step_time;
                return pulse;
            } else {
                self.pulse_iter = None;
            }
        }
        // call function with context and evaluate the result
        let pulse = match self.next_pulse() {
            Err(err) => {
                log::warn!(
                    "Failed to call custom pattern function '{}': {}",
                    self.function.name(),
                    err
                );
                Pulse::from(0.0)
            }
            Ok(pulse) => pulse,
        };
        let mut pulse_iter = pulse.clone().into_iter();
        let pulse_item = pulse_iter.next().unwrap_or(PulseIterItem::default());
        self.pulse_iter = Some(pulse_iter);
        self.pulse = Some(pulse);
        // move step for the next iter call
        self.pulse_count += 1;
        self.pulse_time_count += pulse_item.step_time;
        // return the next pulse item
        pulse_item
    }

    fn set_time_base(&mut self, time_base: &BeatTimeBase) {
        // update function context from the new time base
        if let Err(err) = initialize_context_time_base(self.function.context(), time_base) {
            log::warn!(
                "Failed to update context for custom pattern function '{}': {}",
                self.function.name(),
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
            self.function.context(),
            self.pulse_count,
            self.pulse_time_count,
        ) {
            log::warn!(
                "Failed to update context for custom pattern function '{}': {}",
                self.function.name(),
                err
            );
        }
        // reset function
        if let Err(err) = self.function.reset() {
            log::warn!(
                "Failed to call custom pattern emitter generator function '{}': {}",
                self.function.name(),
                err
            );
        }
        // reset pulse and pulse iter
        self.pulse = None;
        self.pulse_iter = None;
    }
}
