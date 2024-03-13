use std::{cell::RefCell, rc::Rc};

use mlua::prelude::*;

use crate::{
    bindings::{
        initialize_context_pulse_value, initialize_context_step_count,
        initialize_context_time_base, initialize_emitter_context, new_note_events_from_lua,
        LuaTimeoutHook,
    },
    BeatTimeBase, Event, EventIter, Pattern, PulseIterItem,
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

/// EventIter impl, which calls an existing lua script function to generate new events.
#[derive(Debug, Clone)]
pub struct ScriptedEventIter {
    timeout_hook: LuaTimeoutHook,
    function_environment: Option<LuaOwnedTable>,
    function_generator: Option<LuaOwnedFunction>,
    function_context: LuaOwnedTable,
    function: LuaOwnedFunction,
    current_pulse: PulseIterItem,
    step_count: usize,
    step_time_count: f64,
}

impl ScriptedEventIter {
    pub(crate) fn new(
        lua: &Lua,
        timeout_hook: &LuaTimeoutHook,
        function: LuaFunction<'_>,
        time_base: &BeatTimeBase,
        pattern: Rc<RefCell<dyn Pattern>>,
    ) -> LuaResult<Self> {
        // create a new timeout_hook instance and reset it before calling the function
        let mut timeout_hook = timeout_hook.clone();
        timeout_hook.reset();
        // create a new function context
        let step_count = 0;
        let step_time_count = 0.0;
        let mut function_context = lua.create_table()?.into_owned();
        let pattern = pattern.borrow();
        let trigger = true;
        initialize_emitter_context(
            &mut function_context,
            time_base,
            step_count,
            step_time_count,
            pattern.peek(),
            pattern.len(),
            trigger,
        )?;
        // immediately fetch/evaluate the first event and get its environment, so we can immediately
        // show errors from the function and can reset the environment later on to this state.
        let result = function.call::<_, LuaValue>(function_context.to_ref())?;
        if let Some(inner_function) = result.as_function() {
            // function returned a function -> is an iterator. use inner function instead.
            let function_context = function_context.clone();
            let function_environment = function.environment().map(|env| env.into_owned());
            let function_generator = Some(function.into_owned());
            let function = inner_function.clone().into_owned();
            let result = function.call::<_, LuaValue>(function_context.to_ref())?;
            // evaluate and forget the result, just to show errors from the function, if any.
            new_note_events_from_lua(result, None)?;
            let current_pulse = PulseIterItem::default();
            Ok(Self {
                timeout_hook,
                function_environment,
                function_generator,
                function_context,
                function,
                current_pulse,
                step_count,
                step_time_count,
            })
        } else {
            // function returned an event. use this function.
            let function_environment = None;
            let function_generator = None;
            let function = function.into_owned();
            // evaluate and forget the result, just to show errors from the function, if any.
            new_note_events_from_lua(result, None)?;
            let current_pulse = PulseIterItem::default();
            Ok(Self {
                timeout_hook,
                function_environment,
                function_generator,
                function_context,
                function,
                step_count,
                step_time_count,
                current_pulse,
            })
        }
    }

    fn next_event(&mut self, move_step: bool) -> LuaResult<Event> {
        // reset timeout
        self.timeout_hook.reset();
        // move step counter and update context
        if move_step {
            self.step_count += 1;
            self.step_time_count += self.current_pulse.step_time;
            initialize_context_step_count(
                &mut self.function_context,
                self.step_count,
                self.step_time_count,
            )?;
        }
        // call function with context and evaluate result
        let result = self
            .function
            .call::<_, LuaValue>(self.function_context.to_ref())?;
        Ok(Event::NoteEvents(new_note_events_from_lua(result, None)?))
    }
}

impl Iterator for ScriptedEventIter {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        let move_step = true;
        match self.next_event(move_step) {
            Ok(event) => Some(event),
            Err(err) => {
                log::warn!(
                    "Failed to run custom event emitter func '{}': {}",
                    function_name(&self.function),
                    err
                );
                None
            }
        }
    }
}

impl EventIter for ScriptedEventIter {
    fn set_time_base(&mut self, time_base: &BeatTimeBase) {
        // reset timeout
        self.timeout_hook.reset();
        // update function context with the new time base
        if let Err(err) = initialize_context_time_base(&mut self.function_context, time_base) {
            log::warn!(
                "Failed to update context for custom event iter function '{}': {}",
                function_name(&self.function),
                err
            );
        }
    }

    fn set_pulse(&mut self, pulse: PulseIterItem, pattern_pulse_count: usize, emit_event: bool) {
        // reset timeout
        self.timeout_hook.reset();
        // memorize current pulse
        self.current_pulse = pulse;
        // update function context with the new pulse
        if let Err(err) = initialize_context_pulse_value(
            &mut self.function_context,
            pulse,
            pattern_pulse_count,
            emit_event,
        ) {
            log::warn!(
                "Failed to update context for custom event iter function '{}': {}",
                function_name(&self.function),
                err
            );
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
        if let Err(err) = initialize_context_step_count(
            &mut self.function_context,
            self.step_count,
            self.step_time_count,
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
                        "Failed to restore custom event emitter func environment '{}': {}",
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
                        "Failed to call custom event emitter generator function '{}': {}",
                        function_name(function_generator),
                        err
                    );
                }
                Ok(value) => {
                    if let Some(function) = value.as_function() {
                        self.function = function.clone().into_owned();
                    } else {
                        log::warn!(
                            "Failed to call custom event emitter generator function '{}': {}",
                            function_name(function_generator),
                            "Generator does not return a valid emitter function"
                        );
                    }
                }
            };
        }
    }
}
