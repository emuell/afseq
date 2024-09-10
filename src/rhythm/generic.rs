//! Generic `Rhythm` implementation with custom time step and offset types.

use std::{
    borrow::{Borrow, Cow},
    cell::RefCell,
    collections::VecDeque,
    fmt::Debug,
    rc::Rc,
};

use fraction::{ConstOne, ConstZero, Fraction, ToPrimitive};

#[cfg(all(feature = "scripting", test))]
use std::borrow::BorrowMut;

use crate::{
    event::{fixed::FixedEventIter, Event, EventIter, EventIterItem, InstrumentId},
    gate::threshold::ThresholdGate,
    pattern::{fixed::FixedPattern, Pattern},
    time::{BeatTimeBase, SampleTimeDisplay},
    Gate, InputParameter, InputParameterSet, PulseIterItem, Rhythm, RhythmIter, RhythmIterItem,
    SampleTime,
};

// -------------------------------------------------------------------------------------------------

/// Time value of a `GenericRhythm`, used either as Step or Offset.
pub trait GenericRhythmTimeStep: Debug + Clone + Copy + 'static {
    /// The default offset value of the `RhythmTimeStep`. Usually some `0` value.
    fn default_offset() -> Self;
    /// The step value of the `RhythmTimeStep`. Usually some non `0` value.
    fn default_step() -> Self;

    /// Converts the `RhythmTimeStep` to an exact sample time.
    fn to_samples(&self, time_base: &BeatTimeBase) -> f64;
}

// -------------------------------------------------------------------------------------------------

/// Generic `Rhythm` impl which uses a [`Pattern`] to generate pulse events, filtered by a [`Gate`]
/// which then drives an [`EventIter`][`crate::EventIter`].
///
/// Internal time units are generics, and will usually be beats or seconds.
#[derive(Debug)]
pub struct GenericRhythm<Step: GenericRhythmTimeStep, Offset: GenericRhythmTimeStep> {
    time_base: BeatTimeBase,
    step: Step,
    offset: Offset,
    instrument: Option<InstrumentId>,
    input_parameters: InputParameterSet,
    pattern: Box<dyn Pattern>,
    pattern_repeat_count: Option<usize>,
    pattern_playback_finished: bool,
    gate: Box<dyn Gate>,
    event_iter: Box<dyn EventIter>,
    event_iter_sample_time: SampleTime,
    event_iter_next_sample_time: f64,
    event_iter_pulse_item: PulseIterItem,
    event_iter_items: VecDeque<EventIterItem>,
    sample_offset: SampleTime,
}

impl<Step: GenericRhythmTimeStep, Offset: GenericRhythmTimeStep> GenericRhythm<Step, Offset> {
    /// Create a new pattern based rhythm which emits `value` every `beat_time` `step`,
    /// and an optional seed for the random number generator.
    pub fn new(time_base: BeatTimeBase, step: Step) -> Self {
        let offset = Offset::default_offset();
        let instrument = None;
        let input_parameters = InputParameterSet::new();
        let pattern = Box::<FixedPattern>::default();
        let pattern_repeat_count = None;
        let pattern_playback_finished = false;
        let gate = Box::new(ThresholdGate::new());
        let event_iter = Box::<FixedEventIter>::default();
        let event_iter_sample_time = 0;
        let event_iter_next_sample_time = offset.to_samples(&time_base);
        let event_iter_pulse_item = PulseIterItem::default();
        let event_iter_items = VecDeque::new();
        let sample_offset = 0;
        Self {
            time_base,
            step,
            offset,
            instrument,
            input_parameters,
            pattern,
            pattern_repeat_count,
            pattern_playback_finished,
            gate,
            event_iter,
            event_iter_sample_time,
            event_iter_next_sample_time,
            event_iter_pulse_item,
            event_iter_items,
            sample_offset,
        }
    }

    /// Get current time base.
    pub fn time_base(&self) -> BeatTimeBase {
        self.time_base
    }
    /// Get current step.
    pub fn step(&self) -> Step {
        self.step
    }
    /// Get current offset.
    pub fn offset(&self) -> Offset {
        self.offset
    }
    /// Get current pattern.
    pub fn pattern(&self) -> &dyn Pattern {
        self.pattern.borrow()
    }
    /// Get mut access the current pattern (only allowed in tests).
    #[cfg(all(feature = "scripting", test))]
    pub(crate) fn pattern_mut(&mut self) -> &mut dyn Pattern {
        self.pattern.borrow_mut()
    }

    /// Return a new rhythm instance which applies the given step offset to all events.
    #[must_use]
    pub fn with_offset<O: Into<Option<Offset>>>(self, offset: O) -> Self {
        let offset = offset.into().unwrap_or(Offset::default_offset());
        let event_iter_sample_time = 0;
        let event_iter_next_sample_time = offset.to_samples(&self.time_base);
        Self {
            offset,
            event_iter_sample_time,
            event_iter_next_sample_time,
            ..self
        }
    }

    /// Return a new rhythm instance which uses the given instrument for all note events
    /// which have no instrument set.
    #[must_use]
    pub fn with_instrument<I: Into<Option<InstrumentId>>>(self, instrument: I) -> Self {
        let instrument = instrument.into();
        Self { instrument, ..self }
    }

    /// Return a new rhythm instance with the given input parameter map.  
    #[must_use]
    pub fn with_input_parameters(self, parameters: InputParameterSet) -> Self {
        let mut new = self;
        new.input_parameters.clone_from(&parameters);
        new.pattern.set_input_parameters(parameters.clone());
        new.gate.set_input_parameters(parameters.clone());
        new.event_iter.set_input_parameters(parameters);
        new
    }

    /// Return a new rhythm instance which trigger events with the given [`Pattern`].  
    #[must_use]
    pub fn with_pattern<T: Pattern + Sized + 'static>(self, pattern: T) -> Self {
        self.with_pattern_dyn(Box::new(pattern))
    }

    /// Return a new rhythm instance which triggers events with the given dyn [`Pattern`].  
    #[must_use]
    pub fn with_pattern_dyn(self, pattern: Box<dyn Pattern>) -> Self {
        let time_base = self.time_base;
        let input_parameters = self.input_parameters.clone();
        let pattern_repeat_count = self.pattern_repeat_count;
        let mut new = self;
        new.pattern = pattern;
        new.pattern.set_time_base(&time_base);
        new.pattern.set_input_parameters(input_parameters);
        new.pattern.set_repeat_count(pattern_repeat_count);
        new
    }

    /// Return a new rhythm instance which repeats the pattern up to `count` times.
    /// When None, it repeats forever.
    #[must_use]
    pub fn with_repeat(self, count: Option<usize>) -> Self {
        let mut new = self;
        new.pattern_repeat_count = count;
        new.pattern.set_repeat_count(count);
        new
    }

    /// Return a new rhythm instance which uses the given [`Gate`] instead of the default gate.  
    #[must_use]
    pub fn with_gate<T: Gate + Sized + 'static>(self, gate: T) -> Self {
        self.with_gate_dyn(Box::new(gate))
    }

    /// Return a new rhythm instance which uses the given dyn [`Gate`] instead of the default gate.  
    #[must_use]
    pub fn with_gate_dyn(self, gate: Box<dyn Gate>) -> Self {
        let time_base = self.time_base;
        let input_parameters = self.input_parameters.clone();
        let mut new = self;
        new.gate = gate;
        new.gate.set_time_base(&time_base);
        new.gate.set_input_parameters(input_parameters);
        new
    }

    /// Return a new rhythm instance which uses the given [`EventIter`] to trigger events.
    #[must_use]
    pub fn trigger<Iter: EventIter + 'static>(self, iter: Iter) -> Self {
        self.trigger_dyn(Box::new(iter))
    }

    /// Return a new rhythm instance which uses the given dyn [`EventIter`] to trigger events.
    #[must_use]
    pub fn trigger_dyn(self, event_iter: Box<dyn EventIter>) -> Self {
        let time_base = self.time_base;
        let input_parameters = self.input_parameters.clone();
        let mut new = self;
        new.event_iter = event_iter;
        new.event_iter.set_time_base(&time_base);
        new.event_iter.set_input_parameters(input_parameters);
        new
    }

    /// Return current pulse duration in samples
    #[inline]
    pub fn current_steps_sample_duration(&self) -> f64 {
        self.step.to_samples(&self.time_base) * self.event_iter_pulse_item.step_time
    }

    /// Return start sample time of the given event iter item
    #[inline]
    fn event_iter_item_start_time(&self, start: &Fraction) -> SampleTime {
        let step_time = self.current_steps_sample_duration();
        let event_iter_time = self.sample_offset as f64 + self.event_iter_next_sample_time;
        let start = start.to_f64().unwrap_or(0.0);
        (event_iter_time + (step_time * start)) as SampleTime
    }

    /// Return duration in sample time of the given event iter item
    #[inline]
    fn event_iter_item_duration(&self, length: &Fraction) -> SampleTime {
        let step_time = self.current_steps_sample_duration();
        let length = length.to_f64().unwrap_or(1.0);
        (step_time * length) as SampleTime
    }

    /// Set default instrument to event if none is set, else return the event as it is
    fn event_with_default_instrument(&self, mut event_item: EventIterItem) -> EventIterItem {
        if let Some(instrument) = self.instrument {
            if let Event::NoteEvents(note_events) = &mut event_item.event {
                for note_event in note_events.iter_mut().flatten() {
                    note_event.instrument = note_event.instrument.or(Some(instrument));
                }
            }
        }
        event_item
    }

    fn run_pattern(&mut self) -> Option<(PulseIterItem, bool)> {
        debug_assert!(
            self.event_iter_items.is_empty(),
            "Should only run patterns when there are no pending event iter items"
        );
        if let Some(pulse) = self.pattern.run() {
            let emit_event = self.gate.run(&pulse);
            self.event_iter_pulse_item = pulse;
            Some((pulse, emit_event))
        } else {
            None
        }
    }

    fn run_rhythm(
        &mut self,
        sample_time: SampleTime,
        fetch_new_event_iter_items: bool,
    ) -> Option<RhythmIterItem> {
        // quickly check if pattern playback finished
        if self.pattern_playback_finished {
            return None;
        }
        // quickly check if the next event is due before the given target time
        let next_sample_time = self.sample_offset + self.event_iter_next_sample_time as SampleTime;
        if next_sample_time >= sample_time {
            // next event is not yet due
            return None;
        }
        // fetch new event iter items, if necessary
        if self.event_iter_items.is_empty() {
            if !fetch_new_event_iter_items {
                // if we should not fetch new event iter items we're done here
                return None;
            }
            // generate a pulse from the pattern and pass the pulse to the gate
            if let Some((pulse, emit_event)) = self.run_pattern() {
                // generate new events from the gated pulse
                self.event_iter_items = self
                    .event_iter
                    .run(pulse, emit_event)
                    .map_or_else(VecDeque::default, VecDeque::from);
            } else {
                // pattern playback finished
                self.pattern_playback_finished = true;
                return None;
            }
        }
        // fetch a new event item from the event iter item deque
        if let Some(event_item) = self
            .event_iter_items
            .pop_front()
            .map(|event| self.event_with_default_instrument(event))
        {
            // return event as sample timed rhythm iter item
            let time = self.event_iter_item_start_time(&event_item.start);
            if time >= sample_time {
                // the given event iter item is not yet due: put it back
                self.event_iter_items.push_front(event_item);
                return None;
            }
            let event = Some(event_item.event);
            let duration = self.event_iter_item_duration(&event_item.length);
            // advance to the next pulse in the next iteration when all events got consumed
            if self.event_iter_items.is_empty() {
                self.event_iter_next_sample_time += self.current_steps_sample_duration();
            }
            // return event as rhythm iter item
            Some(RhythmIterItem {
                time,
                event,
                duration,
            })
        } else {
            // return 'None' event as sample timed rhythm iter item
            let time = self.event_iter_item_start_time(&Fraction::ZERO);
            debug_assert!(time < sample_time, "Event should be due here");
            let event = None;
            let duration = self.event_iter_item_duration(&Fraction::ONE);
            // advance to the next pulse in the next iteration
            self.event_iter_next_sample_time += self.current_steps_sample_duration();
            // return event as rhythm iter item
            Some(RhythmIterItem {
                time,
                event,
                duration,
            })
        }
    }
}

impl<Step: GenericRhythmTimeStep, Offset: GenericRhythmTimeStep> Clone
    for GenericRhythm<Step, Offset>
{
    fn clone(&self) -> Self {
        Self {
            input_parameters: self.input_parameters.clone(),
            pattern: self.pattern.duplicate(),
            event_iter: self.event_iter.duplicate(),
            event_iter_items: self.event_iter_items.clone(),
            gate: self.gate.duplicate(),
            ..*self
        }
    }
}

impl<Step: GenericRhythmTimeStep, Offset: GenericRhythmTimeStep> Iterator
    for GenericRhythm<Step, Offset>
{
    type Item = RhythmIterItem;

    fn next(&mut self) -> Option<Self::Item> {
        self.run_until_time(SampleTime::MAX)
    }
}

impl<Step: GenericRhythmTimeStep, Offset: GenericRhythmTimeStep> RhythmIter
    for GenericRhythm<Step, Offset>
{
    fn sample_time_display(&self) -> Box<dyn SampleTimeDisplay> {
        Box::new(self.time_base)
    }

    fn sample_offset(&self) -> SampleTime {
        self.sample_offset
    }
    fn set_sample_offset(&mut self, sample_offset: SampleTime) {
        self.sample_offset = sample_offset;
    }

    fn run_until_time(&mut self, sample_time: SampleTime) -> Option<RhythmIterItem> {
        // memorize current time
        self.event_iter_sample_time = sample_time;
        // fetch events
        let fetch_new_items = true;
        self.run_rhythm(sample_time, fetch_new_items)
    }

    fn advance_until_time(&mut self, sample_time: SampleTime) {
        // memorize current time
        self.event_iter_sample_time = sample_time;
        // clear pending event iter items with regular runs
        while !self.event_iter_items.is_empty() {
            let fetch_new_items = false;
            if self.run_rhythm(sample_time, fetch_new_items).is_none() {
                break;
            }
        }
        // when the are still pending events, they are not yet due, so we are done
        if !self.event_iter_items.is_empty() {
            return;
        }
        // quickly check if pattern playback finished
        if self.pattern_playback_finished {
            return;
        }
        // batch advance event iter in full pulse steps
        loop {
            // quickly check if the next event is due before the given target time
            let next_sample_time = self.sample_offset as f64 + self.event_iter_next_sample_time;
            if (next_sample_time as SampleTime) >= sample_time {
                // next event is not yet due: we're done
                return;
            }
            // generate a pulse from the pattern and pass the pulse to the gate
            if let Some((pulse, emit_event)) = self.run_pattern() {
                // test if the event iter crosses the target time
                let step_duration = self.current_steps_sample_duration();
                if ((next_sample_time + step_duration) as SampleTime) < sample_time {
                    // skip all events from the gated pulse
                    self.event_iter.advance(pulse, emit_event);
                    self.event_iter_next_sample_time += step_duration;
                } else {
                    // generate new events from the gated pulse
                    self.event_iter_items = self
                        .event_iter
                        .run(pulse, emit_event)
                        .map_or_else(VecDeque::default, VecDeque::from);
                    // when the remaining step is empty advance to next step
                    if self.event_iter_items.is_empty() {
                        self.event_iter_next_sample_time += self.current_steps_sample_duration();
                    }
                    // we're done either way now...
                    break;
                }
            } else {
                // pattern playback finished: we're done here
                self.pattern_playback_finished = true;
                return;
            }
        }
        // clear remaining event iter items with regular runs
        while !self.event_iter_items.is_empty() {
            let fetch_new_items = true;
            if self.run_rhythm(sample_time, fetch_new_items).is_none() {
                break;
            }
        }
    }
}

impl<Step: GenericRhythmTimeStep, Offset: GenericRhythmTimeStep> Rhythm
    for GenericRhythm<Step, Offset>
{
    fn input_parameters(&self) -> &[Rc<RefCell<InputParameter>>] {
        &self.input_parameters
    }

    fn pattern_step_length(&self) -> f64 {
        self.step.to_samples(&self.time_base)
    }

    fn pattern_length(&self) -> usize {
        self.pattern.len()
    }

    fn time_base(&self) -> &BeatTimeBase {
        &self.time_base
    }

    fn set_time_base(&mut self, time_base: &BeatTimeBase) {
        // reschedule next event's sample time to the new time base
        if self.event_iter_sample_time > 0 {
            self.event_iter_next_sample_time = self.event_iter_sample_time as f64
                + (self.event_iter_next_sample_time - self.event_iter_sample_time as f64)
                    / self.step.to_samples(&self.time_base)
                    * self.step.to_samples(time_base);
        }
        self.time_base.clone_from(time_base);
        // update pattern, gate and event iter
        self.pattern.set_time_base(time_base);
        self.gate.set_time_base(time_base);
        self.event_iter.set_time_base(time_base);
    }

    fn set_instrument(&mut self, instrument: Option<InstrumentId>) {
        self.instrument = instrument;
    }

    fn set_external_context(&mut self, data: &[(Cow<str>, f64)]) {
        self.pattern.set_external_context(data);
        self.gate.set_external_context(data);
        self.event_iter.set_external_context(data);
    }

    fn duplicate(&self) -> Rc<RefCell<dyn Rhythm>> {
        Rc::new(RefCell::new(self.clone()))
    }

    fn reset(&mut self) {
        // reset sample offset
        self.sample_offset = 0;
        // reset pattern and gate
        self.pattern.reset();
        self.pattern_playback_finished = false;
        self.gate.reset();
        // reset iterator state
        self.event_iter.reset();
        self.event_iter_sample_time = 0;
        self.event_iter_next_sample_time = self.offset.to_samples(&self.time_base);
        self.event_iter_pulse_item = PulseIterItem::default();
        self.event_iter_items.clear();
    }
}
