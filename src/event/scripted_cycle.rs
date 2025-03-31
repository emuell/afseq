use std::{borrow::Cow, collections::HashMap};

use fraction::ToPrimitive;
use mlua::prelude::{LuaError, LuaResult};

use crate::{
    bindings::{
        add_lua_callback_error, note_events_from_value, ContextPlaybackState, LuaCallback,
        LuaTimeoutHook,
    },
    event::cycle::CycleNoteEvents,
    BeatTimeBase, Cycle, CycleEvent, CycleValue, EventIter, EventIterItem, InputParameterSet,
    NoteEvent, PulseIterItem,
};

// -------------------------------------------------------------------------------------------------

/// Emits a vector of [`EventIterItem`] from a Tidal [`Cycle`].
///
/// Channels from cycle are merged down into note events on different voices.
/// Values in cycles can be mapped to notes with an optional mapping table or
/// callbacks from from scripts.
///
/// See also [`CycleEventIter`](`super::cycle::CycleEventIter`)
#[derive(Clone, Debug)]
pub struct ScriptedCycleEventIter {
    cycle: Cycle,
    mappings: HashMap<String, Vec<Option<NoteEvent>>>,
    mapping_callback: Option<LuaCallback>,
    timeout_hook: Option<LuaTimeoutHook>,
    channel_steps: Vec<usize>,
}

impl ScriptedCycleEventIter {
    /// Return a new cycle with the given value mappings applied.
    pub fn with_mappings(cycle: Cycle, mappings: Vec<(String, Vec<Option<NoteEvent>>)>) -> Self {
        let mappings = mappings.into_iter().collect();
        let mapping_callback = None;
        let timeout_hook = None;
        let channel_steps = vec![];
        Self {
            cycle,
            mappings,
            mapping_callback,
            timeout_hook,
            channel_steps,
        }
    }

    /// Return a new cycle with the given mapping callback applied.
    pub(crate) fn with_mapping_callback(
        cycle: Cycle,
        timeout_hook: &LuaTimeoutHook,
        mapping_callback: LuaCallback,
        time_base: &BeatTimeBase,
    ) -> LuaResult<Self> {
        // create a new timeout_hook instance and reset it before calling the function
        let mut timeout_hook = timeout_hook.clone();
        timeout_hook.reset();
        let mappings = HashMap::new();
        // initialize emitter context for the function
        let mut mapping_callback = mapping_callback;
        let playback_state = ContextPlaybackState::Running;
        let channel = 0;
        let step = 0;
        let step_length = 0.0;
        mapping_callback.set_cycle_context(
            playback_state,
            time_base,
            channel,
            step,
            step_length,
        )?;
        let channel_steps = vec![];
        Ok(Self {
            cycle,
            mappings,
            mapping_callback: Some(mapping_callback),
            timeout_hook: Some(timeout_hook),
            channel_steps,
        })
    }

    /// Generate a note event stack from a single cycle event, applying mappings if necessary
    fn cycle_to_note_event(
        &mut self,
        channel_index: usize,
        channel_step: usize,
        step_length: f64,
        event: CycleEvent,
    ) -> LuaResult<Vec<Option<NoteEvent>>> {
        let mut note_events = {
            if let Some(mapping_callback) = self.mapping_callback.as_mut() {
                // update step in context
                mapping_callback.set_context_cycle_step(
                    channel_index,
                    channel_step,
                    step_length,
                )?;
                // call mapping function
                let result = mapping_callback.call_with_arg(event.string())?;
                note_events_from_value(&result, None)?
            } else if let Some(note_events) = self.mappings.get(event.string()) {
                // apply custom note mapping
                note_events.clone()
            } else {
                // try converting the cycle value to a single note
                event.value().try_into().map_err(LuaError::RuntimeError)?
            }
        };
        // verify that all identifiers are mapped
        if (note_events.is_empty() || note_events.iter().all(|f| f.is_none()))
            && self.mapping_callback.is_none()
            && !matches!(event.value(), CycleValue::Rest | CycleValue::Hold)
        {
            return Err(LuaError::runtime(format!(
                "invalid/unknown identifier in cycle: '{}'. please check for typos or add a custom mapping for it.",
                event.string()
            )));
        }
        // inject target instrument, if present
        if let Some(target) = event.targets().first() {
            if let Some(instrument) = target.into() {
                for mut note_event in &mut note_events {
                    if let Some(note_event) = &mut note_event {
                        note_event.instrument = Some(instrument);
                    }
                }
            }
        }
        Ok(note_events)
    }

    /// Generate next batch of events from the next cycle run.
    /// Converts cycle events to note events and flattens channels into note columns.
    fn generate(&mut self) -> Vec<EventIterItem> {
        // run the cycle event generator
        let events = {
            match self.cycle.generate() {
                Ok(events) => events,
                Err(err) => {
                    add_lua_callback_error("cycle", &LuaError::RuntimeError(err));
                    // skip processing events
                    return vec![];
                }
            }
        };
        // reset timeout hook for mapping functions
        if let Some(timeout_hook) = &mut self.timeout_hook {
            timeout_hook.reset();
        }
        // set callback playback state
        if let Some(callback) = &mut self.mapping_callback {
            let playback_state = ContextPlaybackState::Running;
            if let Err(err) = callback.set_context_playback_state(playback_state) {
                callback.handle_error(&err);
            }
        }
        // convert possibly mapped cycle channel items to a list of note events
        let mut timed_note_events = CycleNoteEvents::new();
        for (channel_index, channel_events) in events.into_iter().enumerate() {
            if self.channel_steps.len() <= channel_index {
                self.channel_steps.resize(channel_index + 1, 0);
            }
            for event in channel_events.into_iter() {
                // increase step counter
                let channel_step = self.channel_steps[channel_index];
                self.channel_steps[channel_index] += 1;
                // convert cycle to note event
                let start = event.span().start();
                let length = event.span().length();
                let step_length = length.to_f64().unwrap_or(0.0);
                match self.cycle_to_note_event(channel_index, channel_step, step_length, event) {
                    Err(err) => {
                        if let Some(callback) = &self.mapping_callback {
                            callback.handle_error(&err)
                        } else {
                            add_lua_callback_error("map", &err)
                        }
                    }
                    Ok(note_events) => {
                        if !note_events.is_empty() {
                            timed_note_events.add(channel_index, start, length, note_events);
                        }
                    }
                }
            }
        }
        // convert timed note events into EventIterItems
        timed_note_events.into_event_iter_items()
    }

    /// Skip next batch of events from the cycle.
    /// This maintains cycle mapping callback states as well, if needed.
    fn advance(&mut self) {
        if let Some(mapping_callback) = &mut self.mapping_callback {
            // run the cycle event generator
            let events = {
                match self.cycle.generate() {
                    Ok(events) => events,
                    Err(err) => {
                        add_lua_callback_error("cycle", &LuaError::RuntimeError(err));
                        return;
                    }
                }
            };
            if mapping_callback.is_stateful().unwrap_or(true) {
                // reset timeout hooks
                if let Some(timeout_hook) = &mut self.timeout_hook {
                    timeout_hook.reset();
                }
                // set playback state
                let playback_state = ContextPlaybackState::Seeking;
                if let Err(err) = mapping_callback.set_context_playback_state(playback_state) {
                    mapping_callback.handle_error(&err);
                }
                // run stateful callbacks but ignore results
                for (channel_index, channel_events) in events.into_iter().enumerate() {
                    if self.channel_steps.len() <= channel_index {
                        self.channel_steps.resize(channel_index + 1, 0);
                    }
                    for event in channel_events.into_iter() {
                        // move step counter
                        let channel_step = self.channel_steps[channel_index];
                        self.channel_steps[channel_index] += 1;
                        // update step in context
                        let step_length = event.span().length().to_f64().unwrap_or(0.0);
                        if let Err(err) = mapping_callback.set_context_cycle_step(
                            channel_index,
                            channel_step,
                            step_length,
                        ) {
                            add_lua_callback_error("cycle", &err);
                            return;
                        }
                        // call mapping function
                        if let Err(err) = mapping_callback.call_with_arg(event.string()) {
                            add_lua_callback_error("cycle", &err);
                            return;
                        }
                    }
                }
            } else {
                // advance channel_steps for generated each event
                for (channel_index, channel_events) in events.into_iter().enumerate() {
                    if self.channel_steps.len() <= channel_index {
                        self.channel_steps.resize(channel_index + 1, 0);
                    }
                    self.channel_steps[channel_index] += channel_events.len();
                }
            }
        } else {
            // no mapping callback present: just advance the cycle
            self.cycle.advance();
            self.channel_steps.clear();
        }
    }
}

impl EventIter for ScriptedCycleEventIter {
    fn set_time_base(&mut self, time_base: &BeatTimeBase) {
        if let Some(timeout_hook) = &mut self.timeout_hook {
            timeout_hook.reset();
        }
        if let Some(callback) = &mut self.mapping_callback {
            if let Err(err) = callback.set_context_time_base(time_base) {
                callback.handle_error(&err);
            }
        }
    }

    fn set_external_context(&mut self, data: &[(Cow<str>, f64)]) {
        if let Some(timeout_hook) = &mut self.timeout_hook {
            timeout_hook.reset();
        }
        if let Some(callback) = &mut self.mapping_callback {
            if let Err(err) = callback.set_context_external_data(data) {
                callback.handle_error(&err);
            }
        }
    }

    fn set_input_parameters(&mut self, parameters: InputParameterSet) {
        if let Some(timeout_hook) = &mut self.timeout_hook {
            timeout_hook.reset();
        }
        if let Some(callback) = &mut self.mapping_callback {
            if let Err(err) = callback.set_context_input_parameters(parameters) {
                callback.handle_error(&err);
            }
        }
    }

    fn run(&mut self, _pulse: PulseIterItem, emit_event: bool) -> Option<Vec<EventIterItem>> {
        if emit_event {
            Some(self.generate())
        } else {
            None
        }
    }

    fn advance(&mut self, _pulse: PulseIterItem, emit_event: bool) {
        if emit_event {
            self.advance();
        }
    }

    fn duplicate(&self) -> Box<dyn EventIter> {
        Box::new(self.clone())
    }

    fn reset(&mut self) {
        // reset cycle
        self.cycle.reset();
        if let Some(timeout_hook) = &mut self.timeout_hook {
            // reset timeout
            timeout_hook.reset();
        }
        if let Some(callback) = &mut self.mapping_callback {
            // reset step counter
            let channel = 0;
            let step = 0;
            let step_length = 0.0;
            self.channel_steps.clear();
            if let Err(err) = callback.set_context_cycle_step(channel, step, step_length) {
                callback.handle_error(&err);
            }
            // restore function
            if let Err(err) = callback.reset() {
                callback.handle_error(&err);
            }
        }
    }
}
