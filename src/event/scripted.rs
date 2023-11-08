use mlua::prelude::*;

use crate::bindings::unwrap::note_events_from_value;
use crate::{event::InstrumentId, Event, EventIter};

// -------------------------------------------------------------------------------------------------

/// EventIter impl, which calls an existing lua script function to generate new events.
#[derive(Debug, Clone)]
pub struct ScriptedEventIter {
    environment: Option<LuaOwnedTable>,
    function: LuaOwnedFunction,
    instrument: Option<InstrumentId>,
    event: Option<Event>,
}

impl ScriptedEventIter {
    pub fn new(function: LuaFunction<'_>, instrument: Option<InstrumentId>) -> mlua::Result<Self> {
        // immediately fetch/evaluate the first event and get its environment, so we can immediately
        // show errors from the function and can reset the environment later on to this state.
        let result = function.call::<(), LuaValue>(())?;
        if let Some(inner_function) = result.as_function() {
            // function returned a function -> is an iterator. use inner function instead.
            let result = inner_function.call::<(), LuaValue>(())?;
            let environment = inner_function.environment().map(|env| env.into_owned());
            let function = inner_function.clone().into_owned();
            let event = Some(Event::NoteEvents(note_events_from_value(
                result, None, instrument,
            )?));
            Ok(Self {
                environment,
                function,
                event,
                instrument,
            })
        } else {
            // function returned an event. use this function.
            let environment = function.environment().map(|env| env.into_owned());
            let function = function.into_owned();
            let event = Some(Event::NoteEvents(note_events_from_value(
                result, None, instrument,
            )?));
            Ok(Self {
                environment,
                function,
                event,
                instrument,
            })
        }
    }

    fn next_event(&self) -> mlua::Result<Event> {
        let result = self.function.call::<(), LuaValue>(())?;
        Ok(Event::NoteEvents(note_events_from_value(
            result,
            None,
            self.instrument,
        )?))
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
        // fetch new event
        self.event = self.next_event().ok();
    }
}
