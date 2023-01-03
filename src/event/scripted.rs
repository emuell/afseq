use std::path::PathBuf;

use rhai::{Array, Engine, EvalAltResult, FnPtr, NativeCallContext, Position, AST, INT};

use crate::bindings::{
    new_engine,
    unwrap::{is_empty_note_value, unwrap_array, unwrap_note_event, ErrorCallContext},
};

use crate::{
    event::{new_note_vector, InstrumentId},
    Event, EventIter,
};

// -------------------------------------------------------------------------------------------------

/// EventIter impl, which calls an existing rhai script function to generate new events.
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
        let array: Array = fn_ptr.call(engine, ast, {})?;
        // Supported array args:
        // [NOTE, VEL] -> single note
        // [[NOTE, VEL], ..] -> poly notes
        let mut sequence = Vec::with_capacity(array.len());
        if array.is_empty() {
            // []
            sequence.push(None);
        } else if array[0].type_name() == "string" || array[0].is::<INT>() || array[0].is::<()>() {
            // [NOTE, VEL]
            if is_empty_note_value(&array[0]) {
                sequence.push(None);
            } else {
                sequence.push(Some(unwrap_note_event(&context, array, instrument)?));
            }
        } else {
            // [[NOTE, VEL], ..]
            for item in array {
                if item.is::<()>() {
                    sequence.push(None);
                } else {
                    let note_item_array = unwrap_array(&context, item)?;
                    if note_item_array.is_empty() || is_empty_note_value(&note_item_array[0]) {
                        sequence.push(None);
                    } else {
                        sequence.push(Some(unwrap_note_event(
                            &context,
                            note_item_array,
                            instrument,
                        )?));
                    }
                }
            }
        }
        Ok(Event::NoteEvents(new_note_vector(sequence)))
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
