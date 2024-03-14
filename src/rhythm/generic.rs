//! Generic `Rhythm` implementation with custom time step and offset types.

use std::{cell::RefCell, fmt::Debug, rc::Rc};

use rand::{thread_rng, Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

use crate::{
    event::{empty::EmptyEventIter, Event, EventIter, InstrumentId},
    gate::ProbabilityGate,
    pattern::{fixed::FixedPattern, Pattern},
    time::{BeatTimeBase, SampleTimeDisplay},
    Gate, Rhythm, RhythmSampleIter, SampleTime,
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

/// Generic `Rhythm` impl which uses a [Pattern] to generate pulse events, filtered by a [Gate]
/// which then drives an [EventIter].
///
/// Internal time units are generics, and will usually be beats or seconds.
#[derive(Clone, Debug)]
pub struct GenericRhythm<Step: GenericRhythmTimeStep, Offset: GenericRhythmTimeStep> {
    time_base: BeatTimeBase,
    step: Step,
    offset: Offset,
    instrument: Option<InstrumentId>,
    pattern: Rc<RefCell<dyn Pattern>>,
    gate: Rc<RefCell<dyn Gate>>,
    event_iter: Rc<RefCell<dyn EventIter>>,
    event_iter_sample_time: SampleTime,
    event_iter_next_sample_time: f64,
    sample_offset: SampleTime,
    rand: Xoshiro256PlusPlus,
}

impl<Step: GenericRhythmTimeStep, Offset: GenericRhythmTimeStep> GenericRhythm<Step, Offset> {
    /// Create a new pattern based emitter which emits `value` every beat_time `step`.  
    pub fn new(time_base: BeatTimeBase, step: Step) -> Self {
        let seed = thread_rng().gen();
        Self::new_with_seed(time_base, step, &seed)
    }
    /// Create a new pattern based emitter which emits `value` every beat_time `step`,
    /// initializing the internal random number generator with the specified seed value.
    pub fn new_with_seed(time_base: BeatTimeBase, step: Step, seed: &[u8; 32]) -> Self {
        let offset = Offset::default_offset();
        let rand = Xoshiro256PlusPlus::from_seed(*seed);
        let instrument = None;
        let pattern = Rc::new(RefCell::new(FixedPattern::default()));
        let gate = Rc::new(RefCell::new(ProbabilityGate::new(rand.clone())));
        let event_iter = Rc::new(RefCell::new(EmptyEventIter {}));
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
            rand,
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
    pub fn pattern(&self) -> Rc<RefCell<dyn Pattern>> {
        Rc::clone(&self.pattern)
    }

    /// Apply the given step offset to all events.
    pub fn with_offset<O: Into<Option<Offset>>>(&self, offset: O) -> Self {
        let offset = offset.into().unwrap_or(Offset::default_offset());
        let event_iter_sample_time = 0;
        let event_iter_next_sample_time = offset.to_samples(&self.time_base);
        Self {
            offset,
            pattern: Rc::clone(&self.pattern),
            gate: Rc::clone(&self.gate),
            event_iter: Rc::clone(&self.event_iter),
            event_iter_sample_time,
            event_iter_next_sample_time,
            rand: self.rand.clone(),
            ..*self
        }
    }

    /// Use the given instrument for all note events which have no instrument set
    pub fn with_instrument<I: Into<Option<InstrumentId>>>(&self, instrument: I) -> Self {
        let instrument = instrument.into();
        Self {
            instrument,
            pattern: Rc::clone(&self.pattern),
            gate: Rc::clone(&self.gate),
            event_iter: Rc::clone(&self.event_iter),
            rand: self.rand.clone(),
            ..*self
        }
    }

    /// Trigger events with the given [`Pattern`].  
    pub fn with_pattern<T: Pattern + Sized + 'static>(&self, pattern: T) -> Self {
        self.with_pattern_dyn(Rc::new(RefCell::new(pattern)))
    }

    /// Trigger events with the given [`Pattern`].  
    pub fn with_pattern_dyn(&self, pattern: Rc<RefCell<dyn Pattern>>) -> Self {
        Self {
            pattern,
            gate: Rc::clone(&self.gate),
            event_iter: Rc::clone(&self.event_iter),
            rand: self.rand.clone(),
            ..*self
        }
    }

    /// Use the given [`Gate`] instead of the default probability gate.  
    pub fn with_gate<T: Gate + Sized + 'static>(&self, gate: T) -> Self {
        self.with_gate_dyn(Rc::new(RefCell::new(gate)))
    }

    /// Use the given [`Gate`] instead of the default probability gate.  
    pub fn with_gate_dyn(&self, gate: Rc<RefCell<dyn Gate>>) -> Self {
        Self {
            pattern: Rc::clone(&self.pattern),
            gate,
            event_iter: Rc::clone(&self.event_iter),
            rand: self.rand.clone(),
            ..*self
        }
    }

    /// Use the given [`EventIter`] to trigger events.
    pub fn trigger<Iter: EventIter + 'static>(&self, iter: Iter) -> Self {
        self.trigger_dyn(Rc::new(RefCell::new(iter)))
    }

    /// Use the given dyn [`EventIter`] to trigger events.
    pub fn trigger_dyn(&self, event_iter: Rc<RefCell<dyn EventIter>>) -> Self {
        Self {
            pattern: Rc::clone(&self.pattern),
            gate: Rc::clone(&self.gate),
            event_iter,
            rand: self.rand.clone(),
            ..*self
        }
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

impl<Step: GenericRhythmTimeStep, Offset: GenericRhythmTimeStep> Iterator
    for GenericRhythm<Step, Offset>
{
    type Item = (SampleTime, Option<Event>, SampleTime);

    fn next(&mut self) -> Option<Self::Item> {
        let mut pattern = self.pattern.borrow_mut();
        if pattern.is_empty() {
            None
        } else {
            // generate a pulse from the pattern
            let pulse = pattern.run();
            // pass pulse to gate
            let emit_event = self.gate.borrow_mut().run(&pulse);
            // generate an event from the event iter
            let mut event_iter = self.event_iter.borrow_mut();
            let event = self.event_with_default_instrument(event_iter.run(pulse, emit_event));
            // return event as sample timed rhythm iter item
            let sample_time = self.sample_offset + self.event_iter_next_sample_time as SampleTime;
            let event_duration =
                (self.step.to_samples(&self.time_base) * pulse.step_time) as SampleTime;
            self.event_iter_next_sample_time +=
                self.step.to_samples(&self.time_base) * pulse.step_time;
            Some((sample_time, event, event_duration))
        }
    }
}

impl<Step: GenericRhythmTimeStep, Offset: GenericRhythmTimeStep> RhythmSampleIter
    for GenericRhythm<Step, Offset>
{
    fn sample_time_display(&self) -> Box<dyn SampleTimeDisplay> {
        Box::new(self.time_base)
    }

    fn sample_offset(&self) -> SampleTime {
        self.sample_offset
    }
    fn set_sample_offset(&mut self, sample_offset: SampleTime) {
        self.sample_offset = sample_offset
    }

    fn next_until_time(
        &mut self,
        sample_time: SampleTime,
    ) -> Option<(SampleTime, Option<Event>, SampleTime)> {
        // memorize target sample time for self.set_time_base updates
        self.event_iter_sample_time = sample_time;
        // check if the next event is scheduled before the given target time
        let next_sample_time = self.sample_offset + self.event_iter_next_sample_time as SampleTime;
        if next_sample_time < sample_time {
            self.next()
        } else {
            None
        }
    }
}

impl<Step: GenericRhythmTimeStep, Offset: GenericRhythmTimeStep> Rhythm
    for GenericRhythm<Step, Offset>
{
    fn pattern_step_length(&self) -> f64 {
        self.step.to_samples(&self.time_base)
    }

    fn pattern_length(&self) -> usize {
        self.pattern.borrow().len()
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
        self.pattern.borrow_mut().set_time_base(time_base);
        self.gate.borrow_mut().set_time_base(time_base);
        self.event_iter.borrow_mut().set_time_base(time_base);
    }

    fn set_instrument(&mut self, instrument: Option<InstrumentId>) {
        self.instrument = instrument;
    }

    fn duplicate(&self) -> Rc<RefCell<dyn Rhythm>> {
        Rc::new(RefCell::new(Self {
            pattern: self.pattern.borrow().duplicate(),
            event_iter: self.event_iter.borrow().duplicate(),
            ..self.clone()
        }))
    }

    fn reset(&mut self) {
        // reset sample offset
        self.sample_offset = 0;
        // reset iterator state
        self.event_iter.borrow_mut().reset();
        self.event_iter_sample_time = 0;
        self.event_iter_next_sample_time = self.offset.to_samples(&self.time_base);
        self.pattern.borrow_mut().reset();
    }
}
