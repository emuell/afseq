use std::borrow::Cow;

use mlua::prelude::*;

use crate::{
    bindings::{note_events_from_value, LuaCallback, LuaTimeoutHook},
    event::{fixed::FixedEventIter, NoteEvent},
    BeatTimeBase, Event, EventIter, EventIterItem, PulseIterItem,
};

// -------------------------------------------------------------------------------------------------

/// eventiter impl, which calls an existing lua script function to generate new events.
#[derive(Debug)]
pub struct ScriptedEventIter {
    timeout_hook: LuaTimeoutHook,
    callback: LuaCallback,
    note_event_state: Vec<Option<NoteEvent>>,
    pulse_step: usize,
    pulse_time_step: f64,
    step: usize,
}

impl ScriptedEventIter {
    pub(crate) fn new(
        timeout_hook: &LuaTimeoutHook,
        callback: LuaCallback,
        time_base: &BeatTimeBase,
    ) -> LuaResult<Self> {
        // create a new timeout_hook instance and reset it before calling the function
        let mut timeout_hook = timeout_hook.clone();
        timeout_hook.reset();
        // initialize emitter context for the function
        let mut callback = callback;
        let note_event_state = Vec::new();
        let pulse = PulseIterItem::default();
        let pulse_step = 0;
        let pulse_time_step = 0.0;
        let step = 0;
        callback.set_emitter_context(time_base, pulse, pulse_step, pulse_time_step, step)?;
        Ok(Self {
            timeout_hook,
            callback,
            note_event_state,
            pulse_step,
            pulse_time_step,
            step,
        })
    }

    fn generate_event(&mut self, pulse: PulseIterItem) -> LuaResult<Option<Vec<EventIterItem>>> {
        // reset timeout
        self.timeout_hook.reset();
        // update function context
        self.callback.set_context_pulse_value(pulse)?;
        self.callback
            .set_context_pulse_step(self.pulse_step, self.pulse_time_step)?;
        self.callback.set_context_step(self.step)?;
        // invoke callback and evaluate the result
        let events = note_events_from_value(&self.callback.call()?, None)?;
        // normalize event
        let mut event = Event::NoteEvents(events);
        FixedEventIter::normalize_event(&mut event, &mut self.note_event_state);
        // return as EventIterItem
        Ok(Some(vec![EventIterItem::new(event)]))
    }

    fn omit_event(&mut self, pulse: PulseIterItem) -> LuaResult<()> {
        if self.callback.is_stateful().unwrap_or(true) {
            // reset timeout
            self.timeout_hook.reset();
            // update function context
            self.callback.set_context_pulse_value(pulse)?;
            self.callback
                .set_context_pulse_step(self.pulse_step, self.pulse_time_step)?;
            self.callback.set_context_step(self.step)?;
            // invoke callback and ignore the result
            self.callback.call()?;
            Ok(())
        } else {
            Ok(())
        }
    }
}

impl Clone for ScriptedEventIter {
    fn clone(&self) -> Self {
        Self {
            timeout_hook: self.timeout_hook.clone(),
            callback: self.callback.clone(),
            note_event_state: self.note_event_state.clone(),
            pulse_step: self.pulse_step,
            pulse_time_step: self.pulse_time_step,
            step: self.step,
        }
    }
}

impl EventIter for ScriptedEventIter {
    fn set_time_base(&mut self, time_base: &BeatTimeBase) {
        // reset timeout
        self.timeout_hook.reset();
        // update function context with the new time base
        if let Err(err) = self.callback.set_context_time_base(time_base) {
            self.callback.handle_error(&err);
        }
    }

    fn set_external_context(&mut self, data: &[(Cow<str>, f64)]) {
        // update function context from the new time base
        if let Err(err) = self.callback.set_context_external_data(data) {
            self.callback.handle_error(&err);
        }
    }

    fn run(&mut self, pulse: PulseIterItem, emit_event: bool) -> Option<Vec<EventIterItem>> {
        // generate a new event and move or only update pulse counters
        if emit_event {
            let event = match self.generate_event(pulse) {
                Ok(event) => event,
                Err(err) => {
                    self.callback.handle_error(&err);
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

    fn omit(&mut self, pulse: PulseIterItem, emit_event: bool) {
        // generate a new event and move or only update pulse counters
        if emit_event {
            if let Err(err) = self.omit_event(pulse) {
                self.callback.handle_error(&err);
            }
            self.step += 1;
        }
        self.pulse_step += 1;
        self.pulse_time_step += pulse.step_time;
    }

    fn duplicate(&self) -> Box<dyn EventIter> {
        Box::new(self.clone())
    }

    fn reset(&mut self) {
        // reset timeout
        self.timeout_hook.reset();
        // reset step counter
        self.step = 0;
        if let Err(err) = self.callback.set_context_step(self.step) {
            self.callback.handle_error(&err);
        }
        // reset pulse counter
        self.pulse_step = 0;
        self.pulse_time_step = 0.0;
        if let Err(err) = self
            .callback
            .set_context_pulse_step(self.pulse_step, self.pulse_time_step)
        {
            self.callback.handle_error(&err);
        }
        // restore function
        if let Err(err) = self.callback.reset() {
            self.callback.handle_error(&err);
        }
        // reset last event
        self.note_event_state.clear();
    }
}
