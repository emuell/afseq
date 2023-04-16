use std::path::PathBuf;

use rhai::{Dynamic, Engine, EvalAltResult, FnPtr, NativeCallContext, Position, AST};

use crate::bindings::{
    new_engine,
    unwrap::{unwrap_note_events_from_dynamic, ErrorCallContext},
};

use crate::{event::InstrumentId, Event, EventIter};

// -------------------------------------------------------------------------------------------------

/// EventIter impl, which calls an existing rhai script function to generate new events.
///
/// NB: This event iter can not be cloned.
#[derive(Debug)]
pub struct ScriptedEventIter {
    engine: Engine,
    ast: AST,
    function: FnPtr,
    instrument: Option<InstrumentId>,
    event: Option<Event>,
}

impl ScriptedEventIter {
    pub fn new(
        context: &NativeCallContext,
        function: FnPtr,
        instrument: Option<InstrumentId>,
    ) -> Result<Self, Box<EvalAltResult>> {
        // create a new engine
        let engine = new_engine();
        // compile AST from the callback context's source
        let source_file = context.global_runtime_state().source();
        if let Some(source_file) = source_file {
            let ast = context.engine().compile_file(PathBuf::from(source_file))?;
            // immediately fetch/evaluate the first event, so we can immediately show errors
            let event = Self::next_event_from(&engine, &ast, &function, instrument)?;
            Ok(Self {
                engine,
                ast,
                function,
                event: Some(event),
                instrument,
            })
        } else {
            Err(EvalAltResult::ErrorModuleNotFound(
                function.fn_name().to_string(),
                context.position(),
            )
            .into())
        }
    }

    fn next_event(&self) -> Result<Event, Box<EvalAltResult>> {
        Self::next_event_from(&self.engine, &self.ast, &self.function, self.instrument)
    }

    fn next_event_from(
        engine: &Engine,
        ast: &AST,
        fn_ptr: &FnPtr,
        instrument: Option<InstrumentId>,
    ) -> Result<Event, Box<EvalAltResult>> {
        let context = ErrorCallContext::new(fn_ptr.fn_name(), Position::new(1, 1));
        let result: Dynamic = fn_ptr.call(engine, ast, {})?;
        Ok(Event::NoteEvents(unwrap_note_events_from_dynamic(
            &context, result, instrument,
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
                    self.function.fn_name(),
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
        // recreate our engine: this will recreate the function's scope as well.
        self.engine = Engine::new();
    }
}
