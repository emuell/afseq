use std::{borrow::Cow, cell::RefCell, rc::Rc};

use mlua::prelude::*;

use crate::{
    bindings::{
        callback::LuaFunctionCallback, initialize_context_external_data,
        initialize_context_pulse_count, initialize_context_pulse_value,
        initialize_context_step_count, initialize_context_time_base, initialize_emitter_context,
        new_note_events_from_lua, timeout::LuaTimeoutHook,
    },
    BeatTimeBase, Event, EventIter, PulseIterItem,
};

// -------------------------------------------------------------------------------------------------

/// eventiter impl, which calls an existing lua script function to generate new events.
#[derive(Debug, Clone)]
pub struct ScriptedEventIter {
    timeout_hook: LuaTimeoutHook,
    function: LuaFunctionCallback,
    pulse_count: usize,
    pulse_time_count: f64,
    step_count: usize,
    step_time_count: f64,
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
        let step_count = 0;
        let step_time_count = 0.0;
        let pulse_count = 0;
        let pulse_time_count = 0.0;
        let pulse = PulseIterItem::default();
        initialize_emitter_context(
            function.context(),
            time_base,
            pulse,
            pulse_count,
            pulse_time_count,
            step_count,
            step_time_count,
        )?;
        Ok(Self {
            timeout_hook,
            function,
            pulse_count,
            pulse_time_count,
            step_count,
            step_time_count,
        })
    }

    fn next_event(&mut self, pulse: PulseIterItem) -> LuaResult<Event> {
        // reset timeout
        self.timeout_hook.reset();
        // update function context
        initialize_context_pulse_value(self.function.context(), pulse)?;
        initialize_context_pulse_count(
            self.function.context(),
            self.pulse_count,
            self.pulse_time_count,
        )?;
        initialize_context_step_count(
            self.function.context(),
            self.step_count,
            self.step_time_count,
        )?;
        // call function with the context and evaluate the result
        Ok(Event::NoteEvents(new_note_events_from_lua(
            self.function.call()?,
            None,
        )?))
    }
}

impl EventIter for ScriptedEventIter {
    fn set_time_base(&mut self, time_base: &BeatTimeBase) {
        // reset timeout
        self.timeout_hook.reset();
        // update function context with the new time base
        if let Err(err) = initialize_context_time_base(self.function.context(), time_base) {
            log::warn!(
                "Failed to update context for custom event iter function '{}': {}",
                self.function.name(),
                err
            );
        }
    }

    fn set_external_context(&mut self, data: &[(Cow<str>, f64)]) {
        // update function context from the new time base
        if let Err(err) = initialize_context_external_data(self.function.context(), data) {
            log::warn!(
                "Failed to update context for custom pattern function '{}': {}",
                self.function.name(),
                err
            );
        }
    }

    fn run(&mut self, pulse: PulseIterItem, emit_event: bool) -> Option<Event> {
        // generate a new event and move or only update pulse counters
        if emit_event {
            let event = match self.next_event(pulse) {
                Ok(event) => Some(event),
                Err(err) => {
                    log::warn!(
                        "Failed to run custom event emitter func '{}': {}",
                        self.function.name(),
                        err
                    );
                    None
                }
            };
            self.pulse_count += 1;
            self.pulse_time_count += pulse.step_time;
            self.step_count += 1;
            self.step_time_count += pulse.step_time;
            event
        } else {
            self.pulse_count += 1;
            self.pulse_time_count += pulse.step_time;
            None
        }
    }

    fn duplicate(&self) -> Rc<RefCell<dyn EventIter>> {
        Rc::new(RefCell::new(self.clone()))
    }

    fn reset(&mut self) {
        // reset timeout
        self.timeout_hook.reset();
        // reset step counter
        self.step_count = 0;
        self.step_time_count = 0.0;
        self.pulse_count = 0;
        self.pulse_time_count = 0.0;
        if let Err(err) = initialize_context_step_count(
            self.function.context(),
            self.step_count,
            self.step_time_count,
        )
        .and_then(|_| {
            initialize_context_pulse_count(
                self.function.context(),
                self.pulse_count,
                self.pulse_time_count,
            )
        }) {
            log::warn!(
                "Failed to update context for custom pattern function '{}': {}",
                self.function.name(),
                err
            );
        }
        // restore function
        if let Err(err) = self.function.reset() {
            log::warn!(
                "Failed to restore custom event emitter func environment '{}': {}",
                self.function.name(),
                err
            );
        }
    }
}
