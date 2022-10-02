use rhai::{
    packages::Package, Array, Engine, EvalAltResult, FnPtr, NativeCallContext, Position, AST, INT,
};
use rhai_sci::SciPackage;

use super::{
    eval_default_instrument,
    unwrap::{unwrap_array, unwrap_note_event, ErrorCallContext},
};
use crate::{
    event::{new_note_vector, InstrumentId},
    Event, EventIter,
};

// -------------------------------------------------------------------------------------------------

/// EventIter impl, which calls a rhai script function to generate new events.
pub struct FnEventIter {
    engine: Engine,
    ast: AST,
    fn_ptr: FnPtr,
    instrument: Option<InstrumentId>,
    event: Option<Event>,
}

impl FnEventIter {
    pub fn new(context: &NativeCallContext, fn_ptr: FnPtr) -> Result<Self, Box<EvalAltResult>> {
        // fetch default instrument from calling context
        let instrument = eval_default_instrument(context.engine())?;

        // create a new engine and fetch AST for the callback context
        let mut engine = Engine::new();
        engine.set_max_expr_depths(0, 0);

        let sci = SciPackage::new();
        sci.register_into_engine(&mut engine);

        let ast = context
            .engine()
            .eval_expression::<Option<AST>>("__callback_context()")?;

        if let Some(ast) = ast {
            // immediately fetch first event, so we can immediately show errors from the fn_ptrs
            let event = Self::next_event_from(&engine, &ast, &fn_ptr, instrument)?;
            Ok(Self {
                engine,
                ast,
                fn_ptr,
                event: Some(event),
                instrument,
            })
        } else {
            Err(EvalAltResult::ErrorMismatchDataType(
                "AST".to_string(),
                "None".to_string(),
                context.position(),
            )
            .into())
        }
    }

    fn next_event(&self) -> Result<Event, Box<EvalAltResult>> {
        Self::next_event_from(&self.engine, &self.ast, &self.fn_ptr, self.instrument)
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
        if !array.is_empty() && (array[0].type_name() == "string" || array[0].is::<INT>()) {
            // [NOTE, VEL]
            let (note, velocity) = unwrap_note_event(&context, array)?;
            sequence.push((instrument, note, velocity));
        } else {
            // [[NOTE, VEL], ..]
            for item in array {
                let note_item_array = unwrap_array(&context, item)?;
                let (note, velocity) = unwrap_note_event(&context, note_item_array)?;
                sequence.push((instrument, note, velocity));
            }
        }
        Ok(Event::NoteEvents(new_note_vector(sequence)))
    }
}

impl Iterator for FnEventIter {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        let event = self.event.clone();
        self.event = match self.next_event() {
            Ok(event) => Some(event),
            Err(err) => {
                self.event = None;
                println!(
                    "Failed to run custom event emitter func '{}': {}",
                    self.fn_ptr.fn_name(),
                    err
                );
                None
            }
        };
        event
    }
}

impl EventIter for FnEventIter {
    fn reset(&mut self) {
        // recreate our engine: this will recreate the function's scope as well.
        self.engine = Engine::new();
    }
}
