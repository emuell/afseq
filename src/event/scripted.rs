use std::{cell::RefCell, rc::Rc};

use mlua::prelude::*;

use crate::{
    bindings::{initialize_emitter_context, new_note_events_from_lua, LuaTimeoutHook},
    BeatTimeBase, Event, EventIter,
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
    step_count: usize,
    event: Option<Event>,
}

impl ScriptedEventIter {
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
            let function_environment = function.environment().map(|env| env.into_owned());
            let function_generator = Some(function.into_owned());
            let function = inner_function.clone().into_owned();
            let result = function.call::<_, LuaValue>(function_context.to_ref())?;
            let event = Some(Event::NoteEvents(new_note_events_from_lua(result, None)?));
            Ok(Self {
                timeout_hook,
                function_environment,
                function_generator,
                function_context,
                function,
                step_count,
                event,
            })
        } else {
            // function returned an event. use this function.
            let function_environment = None;
            let function_generator = None;
            let function = function.into_owned();
            let event = Some(Event::NoteEvents(new_note_events_from_lua(result, None)?));
            Ok(Self {
                timeout_hook,
                function_environment,
                function_generator,
                function_context,
                function,
                step_count,
                event,
            })
        }
    }

    fn next_event(&mut self, move_step: bool) -> LuaResult<Event> {
        // reset timeout
        self.timeout_hook.reset();
        // move step counter and update context
        if move_step {
            self.step_count += 1;
            self.function_context
                .to_ref()
                .raw_set("step", self.step_count + 1)?;
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
        let event = self.event.clone();
        let move_step = true;
        self.event = match self.next_event(move_step) {
            Ok(event) => Some(event),
            Err(err) => {
                self.event = None;
                log::warn!(
                    "Failed to run custom event emitter func '{}': {}",
                    function_name(&self.function),
                    err
                );
                None
            }
        };
        event
    }
}

impl EventIter for ScriptedEventIter {
    fn set_time_base(&mut self, time_base: &BeatTimeBase) {
        // update function context with the new time base
        if let Err(err) =
            initialize_emitter_context(&mut self.function_context, self.step_count, time_base)
        {
            log::warn!(
                "Failed to update context for custom event iter function '{}': {}",
                function_name(&self.function),
                err
            );
        }
        // TODO: rerun the generator with new context?
    }

    fn duplicate(&self) -> Rc<RefCell<dyn EventIter>> {
        Rc::new(RefCell::new(self.clone()))
    }

    fn reset(&mut self) {
        // reset timeout
        self.timeout_hook.reset();
        // reset step counter
        self.step_count = 0;
        // update step in context
        if let Err(err) = self
            .function_context
            .to_ref()
            .raw_set("step", self.step_count + 1)
        {
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
        // and set new initial event
        let move_step = false;
        self.event = match self.next_event(move_step) {
            Ok(event) => Some(event),
            Err(err) => {
                self.event = None;
                log::warn!(
                    "Failed to run custom event emitter func '{}': {}",
                    function_name(&self.function),
                    err
                );
                None
            }
        };
    }
}
