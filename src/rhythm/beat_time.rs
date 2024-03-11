//! Beat time based `Rhythm` implementations.

use std::{cell::RefCell, rc::Rc};

use crate::{
    event::{empty::EmptyEventIter, Event, EventIter, InstrumentId},
    pattern::{fixed::FixedPattern, Pattern},
    time::{BeatTimeBase, BeatTimeStep, SampleTimeDisplay},
    Rhythm, RhythmSampleIter, SampleTime,
};

// -------------------------------------------------------------------------------------------------

/// Emits `Option(Event)` every nth [`BeatTimeStep`] with an optional pattern and offset.
#[derive(Clone, Debug)]
pub struct BeatTimeRhythm {
    time_base: BeatTimeBase,
    step: BeatTimeStep,
    offset: BeatTimeStep,
    instrument: Option<InstrumentId>,
    pattern: Rc<RefCell<dyn Pattern>>,
    event_iter: Rc<RefCell<dyn EventIter>>,
    event_iter_sample_time: SampleTime,
    event_iter_next_sample_time: f64,
    sample_offset: SampleTime,
}

impl BeatTimeRhythm {
    /// Create a new pattern based emitter which emits `value` every beat_time `step`.  
    pub fn new(time_base: BeatTimeBase, step: BeatTimeStep) -> Self {
        let offset = BeatTimeStep::Beats(0.0);
        let instrument = None;
        let pattern = Rc::new(RefCell::new(FixedPattern::default()));
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
    pub fn step(&self) -> BeatTimeStep {
        self.step
    }
    /// Get current offset.
    pub fn offset(&self) -> BeatTimeStep {
        self.offset
    }
    /// Get current pattern.
    pub fn pattern(&self) -> Rc<RefCell<dyn Pattern>> {
        Rc::clone(&self.pattern)
    }

    /// Apply the given beat-time step offset to all events.
    pub fn with_offset<O: Into<Option<BeatTimeStep>>>(&self, offset: O) -> Self {
        let offset = offset.into().unwrap_or(BeatTimeStep::Beats(0.0));
        let event_iter_sample_time = 0;
        let event_iter_next_sample_time = offset.to_samples(&self.time_base);
        Self {
            offset,
            pattern: Rc::clone(&self.pattern),
            event_iter: Rc::clone(&self.event_iter),
            event_iter_sample_time,
            event_iter_next_sample_time,
            ..*self
        }
    }
    /// Apply the given offset in out step's timing resolution to all events.
    pub fn with_offset_in_step(&self, offset: f32) -> Self {
        let mut offset_steps = self.step;
        offset_steps.set_steps(offset);
        self.with_offset(offset_steps)
    }

    /// Use the given instrument for all note events which have no instrument set
    pub fn with_instrument<I: Into<Option<InstrumentId>>>(&self, instrument: I) -> Self {
        let instrument = instrument.into();
        Self {
            instrument,
            pattern: Rc::clone(&self.pattern),
            event_iter: Rc::clone(&self.event_iter),
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
            event_iter: Rc::clone(&self.event_iter),
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
            event_iter,
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

impl Iterator for BeatTimeRhythm {
    type Item = (SampleTime, Option<Event>, SampleTime);

    fn next(&mut self) -> Option<Self::Item> {
        let mut pattern = self.pattern.borrow_mut();
        if pattern.is_empty() {
            None
        } else {
            let sample_time = self.sample_offset + self.event_iter_next_sample_time as SampleTime;
            let pulse = pattern.run();
            let event_duration = (pulse.step_time * self.pattern_step_length()) as SampleTime;
            let event = if pulse.value > 0.0 {
                let mut event_iter = self.event_iter.borrow_mut();
                event_iter.set_context(pulse, pattern.len());
                Some((
                    sample_time,
                    self.event_with_default_instrument(event_iter.next()),
                    event_duration,
                ))
            } else {
                Some((sample_time, None, event_duration))
            };
            self.event_iter_next_sample_time +=
                self.step.to_samples(&self.time_base) * pulse.step_time;
            event
        }
    }
}

impl RhythmSampleIter for BeatTimeRhythm {
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

impl Rhythm for BeatTimeRhythm {
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
        // update pattern end event iter
        self.pattern.borrow_mut().set_time_base(time_base);
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
