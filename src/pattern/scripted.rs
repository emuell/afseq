use std::{borrow::Cow, cell::RefCell, rc::Rc};

use mlua::prelude::*;

use crate::{
    bindings::{callback::LuaFunctionCallback, pattern_pulse_from_lua, timeout::LuaTimeoutHook},
    BeatTimeBase, Pattern, Pulse, PulseIter, PulseIterItem,
};

// -------------------------------------------------------------------------------------------------

/// Pattern impl, which calls an existing lua script function to generate pulses.
#[derive(Debug, Clone)]
pub struct ScriptedPattern {
    timeout_hook: LuaTimeoutHook,
    function: LuaFunctionCallback,
    repeat_count: Option<usize>,
    repeats: usize,
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
        let repeat_count = None;
        let repeats = 0;
        function.set_pattern_context(time_base, pulse_count, pulse_time_count)?;
        let pulse = None;
        let pulse_iter = None;
        Ok(Self {
            timeout_hook,
            function,
            repeat_count,
            repeats,
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
        self.function
            .set_context_pulse_count(self.pulse_count, self.pulse_time_count)?;
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

    fn run(&mut self) -> Option<PulseIterItem> {
        // if we have a pulse iterator, consume it
        if let Some(pulse_iter) = &mut self.pulse_iter {
            if let Some(pulse) = pulse_iter.next() {
                // move step for the next iter call
                self.pulse_count += 1;
                self.pulse_time_count += pulse.step_time;
                return Some(pulse);
            } else {
                self.pulse_iter = None;
            }
        }
        // apply pattern repeat count, unless this is the first run
        if self.pulse_count > 0 {
            self.repeats += 1;
            if self.repeat_count.is_some_and(|count| self.repeats > count) {
                return None;
            }
        }
        // call function with context and evaluate the result
        let pulse = match self.next_pulse() {
            Err(err) => {
                self.function.handle_error(err);
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
        Some(pulse_item)
    }

    fn set_time_base(&mut self, time_base: &BeatTimeBase) {
        // update function context from the new time base
        if let Err(err) = self.function.set_context_time_base(time_base) {
            self.function.handle_error(err);
        }
    }

    fn set_external_context(&mut self, data: &[(Cow<str>, f64)]) {
        // update function context from the new time base
        if let Err(err) = self.function.set_context_external_data(data) {
            self.function.handle_error(err);
        }
    }

    fn set_repeat_count(&mut self, count: Option<usize>) {
        self.repeat_count = count;
    }

    fn duplicate(&self) -> Rc<RefCell<dyn Pattern>> {
        Rc::new(RefCell::new(self.clone()))
    }

    fn reset(&mut self) {
        // reset timeout
        self.timeout_hook.reset();
        // reset repeat counter
        self.repeats = 0;
        // reset step counter
        self.pulse_count = 0;
        self.pulse_time_count = 0.0;
        // update step in context
        if let Err(err) = self
            .function
            .set_context_pulse_count(self.pulse_count, self.pulse_time_count)
        {
            self.function.handle_error(err);
        }
        // reset function
        if let Err(err) = self.function.reset() {
            self.function.handle_error(err);
        }
        // reset pulse and pulse iter
        self.pulse = None;
        self.pulse_iter = None;
    }
}
