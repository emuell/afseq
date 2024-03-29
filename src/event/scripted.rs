use std::borrow::Cow;

use mlua::prelude::*;

use crate::{
    bindings::{callback::LuaFunctionCallback, new_note_events_from_lua, timeout::LuaTimeoutHook},
    BeatTimeBase, Event, EventIter, PulseIterItem,
};

// -------------------------------------------------------------------------------------------------

/// eventiter impl, which calls an existing lua script function to generate new events.
#[derive(Debug, Clone)]
pub struct ScriptedEventIter {
    timeout_hook: LuaTimeoutHook,
    function: LuaFunctionCallback,
    pulse_step: usize,
    pulse_time_step: f64,
    step: usize,
}

impl ScriptedEventIter {
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
        // initialize emitter context for the function
        let pulse = PulseIterItem::default();
        let pulse_step = 0;
        let pulse_time_step = 0.0;
        let pulse_pattern_length = 1;
        let step = 0;
        function.set_emitter_context(
            time_base,
            pulse,
            pulse_step,
            pulse_time_step,
            pulse_pattern_length,
            step,
        )?;
        Ok(Self {
            timeout_hook,
            function,
            pulse_step,
            pulse_time_step,
            step,
        })
    }

    fn next_event(
        &mut self,
        pulse: PulseIterItem,
        pulse_pattern_length: usize,
    ) -> LuaResult<Event> {
        // reset timeout
        self.timeout_hook.reset();
        // update function context
        self.function.set_context_pulse_value(pulse)?;
        self.function.set_context_pulse_step(
            self.pulse_step,
            self.pulse_time_step,
            pulse_pattern_length,
        )?;
        self.function.set_context_step(self.step)?;
        // call function with the context and evaluate the result
        Ok(Event::NoteEvents(new_note_events_from_lua(
            &self.function.call()?,
            None,
        )?))
    }
}

impl EventIter for ScriptedEventIter {
    fn set_time_base(&mut self, time_base: &BeatTimeBase) {
        // reset timeout
        self.timeout_hook.reset();
        // update function context with the new time base
        if let Err(err) = self.function.set_context_time_base(time_base) {
            self.function.handle_error(&err);
        }
    }

    fn set_external_context(&mut self, data: &[(Cow<str>, f64)]) {
        // update function context from the new time base
        if let Err(err) = self.function.set_context_external_data(data) {
            self.function.handle_error(&err);
        }
    }

    fn run(
        &mut self,
        pulse: PulseIterItem,
        pulse_pattern_length: usize,
        emit_event: bool,
    ) -> Option<Event> {
        // generate a new event and move or only update pulse counters
        if emit_event {
            let event = match self.next_event(pulse, pulse_pattern_length) {
                Ok(event) => Some(event),
                Err(err) => {
                    self.function.handle_error(&err);
                    None
                }
            };
            self.step += 1;
            self.pulse_step += 1;
            self.pulse_time_step += pulse.step_time;
            event
        } else {
            self.pulse_step += 1;
            self.pulse_time_step += pulse.step_time;
            None
        }
    }

    fn duplicate(&self) -> Box<dyn EventIter> {
        Box::new(self.clone())
    }

    fn reset(&mut self) {
        // reset timeout
        self.timeout_hook.reset();
        // reset step counter
        self.step = 0;
        if let Err(err) = self.function.set_context_step(self.step) {
            self.function.handle_error(&err);
        }
        // reset pulse counter
        self.pulse_step = 0;
        self.pulse_time_step = 0.0;
        let pulse_pattern_length = 1;
        if let Err(err) = self.function.set_context_pulse_step(
            self.pulse_step,
            self.pulse_time_step,
            pulse_pattern_length,
        ) {
            self.function.handle_error(&err);
        }
        // restore function
        if let Err(err) = self.function.reset() {
            self.function.handle_error(&err);
        }
    }
}
