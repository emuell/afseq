//! Generic `Rhythm` implementation with custom time step and offset types.

use std::{
    borrow::{Borrow, Cow},
    fmt::Debug,
};

#[cfg(test)]
use std::borrow::BorrowMut;

use crate::{
    event::{empty::EmptyEventIter, Event, EventIter, InstrumentId},
    gate::ProbabilityGate,
    pattern::{fixed::FixedPattern, Pattern},
    time::{BeatTimeBase, SampleTimeDisplay},
    Gate, Rhythm, RhythmIter, RhythmIterItem, SampleTime,
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
    pattern: Box<dyn Pattern>,
    gate: Box<dyn Gate>,
    event_iter: Box<dyn EventIter>,
    event_iter_sample_time: SampleTime,
    event_iter_next_sample_time: f64,
    sample_offset: SampleTime,
}

impl<Step: GenericRhythmTimeStep, Offset: GenericRhythmTimeStep> GenericRhythm<Step, Offset> {
    /// Create a new pattern based emitter which emits `value` every `beat_time` `step`,
    /// and an optional seed for the random number generator.
    pub fn new(time_base: BeatTimeBase, step: Step, seed: Option<[u8; 32]>) -> Self {
        let offset = Offset::default_offset();
        let instrument = None;
        let pattern = Box::<FixedPattern>::default();
        let gate = Box::new(ProbabilityGate::new(seed));
        let event_iter = Box::new(EmptyEventIter {});
        let event_iter_sample_time = 0;
        let event_iter_next_sample_time = offset.to_samples(&time_base);
        let sample_offset = 0;
        Self {
            time_base,
            step,
            offset,
            instrument,
            pattern,
            gate,
            event_iter,
            event_iter_sample_time,
            event_iter_next_sample_time,
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
    #[cfg(test)]
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

    /// Return a new rhythm instance which trigger events with the given [`Pattern`].  
    #[must_use]
    pub fn with_pattern<T: Pattern + Sized + 'static>(self, pattern: T) -> Self {
        self.with_pattern_dyn(Box::new(pattern))
    }

    /// Return a new rhythm instance which triggers events with the given dyn [`Pattern`].  
    #[must_use]
    pub fn with_pattern_dyn(self, pattern: Box<dyn Pattern>) -> Self {
        Self { pattern, ..self }
    }

    /// Return a new rhythm instance which repeats the pattern up to `count` times.
    /// When None, it repeats forever.
    #[must_use]
    pub fn with_repeat(self, count: Option<usize>) -> Self {
        let mut new = self;
        new.pattern.set_repeat_count(count);
        new
    }

    /// Return a new rhythm instance which uses the given [`Gate`] instead of the default
    /// probability gate.  
    #[must_use]
    pub fn with_gate<T: Gate + Sized + 'static>(self, gate: T) -> Self {
        self.with_gate_dyn(Box::new(gate))
    }

    /// Return a new rhythm instance which uses the given dyn [`Gate`] instead of the default
    /// probability gate.  
    #[must_use]
    pub fn with_gate_dyn(self, gate: Box<dyn Gate>) -> Self {
        Self { gate, ..self }
    }

    /// Return a new rhythm instance which uses the given [`EventIter`] to trigger events.
    #[must_use]
    pub fn trigger<Iter: EventIter + 'static>(self, iter: Iter) -> Self {
        self.trigger_dyn(Box::new(iter))
    }

    /// Return a new rhythm instance which uses the given dyn [`EventIter`] to trigger events.
    #[must_use]
    pub fn trigger_dyn(self, event_iter: Box<dyn EventIter>) -> Self {
        Self { event_iter, ..self }
    }

    /// Set default instrument to event if none is set, else return the event as it is
    fn event_with_default_instrument(&self, event: Option<Event>) -> Option<Event> {
        if let Some(instrument) = self.instrument {
            if let Some(event) = event {
                if let Event::NoteEvents(note_events) = event {
                    let new_note_events = note_events
                        .into_iter()
                        .map(|note_event| {
                            note_event.map(|mut note_event| {
                                note_event.instrument = note_event.instrument.or(Some(instrument));
                                note_event
                            })
                        })
                        .collect::<Vec<_>>();
                    Some(Event::NoteEvents(new_note_events))
                } else {
                    Some(event)
                }
            } else {
                event
            }
        } else {
            event
        }
    }
}

impl<Step: GenericRhythmTimeStep, Offset: GenericRhythmTimeStep> Clone
    for GenericRhythm<Step, Offset>
{
    fn clone(&self) -> Self {
        Self {
            pattern: self.pattern.duplicate(),
            event_iter: self.event_iter.duplicate(),
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
        self.run()
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
        // check if the next event is scheduled before the given target time
        self.event_iter_sample_time = sample_time;
        let next_sample_time = self.sample_offset + self.event_iter_next_sample_time as SampleTime;
        if next_sample_time >= sample_time {
            // next event is not yet due
            return None;
        }
        // generate a pulse from the pattern and pass the pulse to the gate
        let (pulse, emit_event) = {
            if let Some(pulse) = self.pattern.run() {
                let emit_event = self.gate.run(&pulse);
                (pulse, emit_event)
            } else {
                // pattern playback finished
                return None;
            }
        };
        // generate an event from the event iter
        let mut event = self.event_iter.run(pulse, emit_event);
        event = self.event_with_default_instrument(event);
        // return event as sample timed rhythm iter item
        let time = self.sample_offset + self.event_iter_next_sample_time as SampleTime;
        let duration = (self.step.to_samples(&self.time_base) * pulse.step_time) as SampleTime;
        self.event_iter_next_sample_time += self.step.to_samples(&self.time_base) * pulse.step_time;
        Some(RhythmIterItem {
            time,
            event,
            duration,
        })
    }
}

impl<Step: GenericRhythmTimeStep, Offset: GenericRhythmTimeStep> Rhythm
    for GenericRhythm<Step, Offset>
{
    fn pattern_step_length(&self) -> f64 {
        self.step.to_samples(&self.time_base)
    }

    fn pattern_length(&self) -> usize {
        self.pattern.len()
    }

    fn set_time_base(&mut self, time_base: &BeatTimeBase) {
        // reschedule next event's sample time to the new time base
        if self.event_iter_sample_time > 0 {
            self.event_iter_next_sample_time = self.event_iter_sample_time as f64
                + (self.event_iter_next_sample_time - self.event_iter_sample_time as f64)
                    / self.step.to_samples(&self.time_base)
                    * self.step.to_samples(time_base);
        }
        self.time_base = *time_base;
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
        self.event_iter.set_external_context(data);
    }

    fn duplicate(&self) -> Box<dyn Rhythm> {
        Box::new(self.clone())
    }

    fn reset(&mut self) {
        // reset sample offset
        self.sample_offset = 0;
        // reset iterator state
        self.event_iter.reset();
        self.event_iter_sample_time = 0;
        self.event_iter_next_sample_time = self.offset.to_samples(&self.time_base);
        self.pattern.reset();
        self.gate.reset();
    }
}
