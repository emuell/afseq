use std::{cell::RefCell, rc::Rc};

use mlua::prelude::*;

use crate::{
    bindings::{initialize_emitter_context, new_note_events_from_lua, LuaTimeoutHook},
    BeatTimeBase, Event, EventIter,
};

// -------------------------------------------------------------------------------------------------

/// EventIter impl, which calls an existing lua script function to generate new events.
#[derive(Debug, Clone)]
pub struct ScriptedEventIter {
    environment: Option<LuaOwnedTable>,
    timeout_hook: LuaTimeoutHook,
    function: LuaOwnedFunction,
    function_context: LuaOwnedTable,
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
            let environment = inner_function.environment().map(|env| env.into_owned());
            let function = inner_function.clone().into_owned();
            let result = function.call::<_, LuaValue>(function_context.to_ref())?;
            let event = Some(Event::NoteEvents(new_note_events_from_lua(result, None)?));
            Ok(Self {
                environment,
                timeout_hook,
                function,
                function_context,
                step_count,
                event,
            })
        } else {
            // function returned an event. use this function.
            let environment = function.environment().map(|env| env.into_owned());
            let function = function.into_owned();
            let event = Some(Event::NoteEvents(new_note_events_from_lua(result, None)?));
            Ok(Self {
                environment,
                timeout_hook,
                function,
                function_context,
                step_count,
                event,
            })
        }
    }

    fn next_event(&mut self) -> LuaResult<Event> {
        // reset timeout
        self.timeout_hook.reset();
        // update context
        self.step_count += 1;
        self.function_context
            .to_ref()
            .raw_set("step", self.step_count + 1)?;
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
        self.event = match self.next_event() {
            Ok(event) => Some(event),
            Err(err) => {
                self.event = None;
                log::warn!(
                    "Failed to run custom event emitter func '{}': {}",
                    self.function
                        .to_ref()
                        .info()
                        .name
                        .unwrap_or("annonymous function".to_string()),
                    err
                );
                None
            }
        };
        event
    }
}

impl EventIter for ScriptedEventIter {
    fn update_time_base(&mut self, time_base: &BeatTimeBase) {
        // update function context with the new time base
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

    fn clone_dyn(&self) -> Rc<RefCell<dyn EventIter>> {
        Rc::new(RefCell::new(self.clone()))
    }
    fn reset(&mut self) {
        // restore function environment
        if let Some(env) = &self.environment {
            if let Err(err) = self.function.to_ref().set_environment(env.to_ref()) {
                log::warn!(
                    "Failed to restore custom event emitter func environment '{}': {}",
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
        // and set new initial event, discarding the last one
        let _ = self.next();
    }
}
