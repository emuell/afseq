use std::{cell::RefCell, fmt::Debug, rc::Rc};

use crate::{
    event::{empty::EmptyEventIter, Event, EventIter, InstrumentId},
    pattern::{fixed::FixedPattern, Pattern},
    time::{BeatTimeBase, SampleTimeDisplay, SecondTimeBase, SecondTimeStep, TimeBase},
    Rhythm, SampleTime,
};

// -------------------------------------------------------------------------------------------------

/// Emits `Option(Event)` every nth [`SecondTimeStep`] with an optional pattern and offset.
#[derive(Clone, Debug)]
pub struct SecondTimeRhythm {
    time_base: SecondTimeBase,
    step: SecondTimeStep,
    offset: SecondTimeStep,
    pattern: Rc<RefCell<dyn Pattern>>,
    instrument: Option<InstrumentId>,
    event_iter: Rc<RefCell<dyn EventIter>>,
    event_iter_sample_time: f64,
    sample_offset: SampleTime,
}

impl SecondTimeRhythm {
    /// Create a new pattern based emitter which emits `value` every beat_time `step`.
    pub fn new(time_base: SecondTimeBase, step: SecondTimeStep) -> Self {
        let offset = 0.0;
        let pattern = Rc::new(RefCell::new(FixedPattern::default()));
        let instrument = None;
        let event_iter = Rc::new(RefCell::new(EmptyEventIter {}));
        let event_iter_sample_time = time_base.seconds_to_samples(offset) as f64;
        let sample_offset = 0;
        Self {
            time_base,
            step,
            offset,
            pattern,
            instrument,
            event_iter,
            event_iter_sample_time,
            sample_offset,
        }
    }

    /// Get current time base.
    pub fn time_base(&self) -> SecondTimeBase {
        self.time_base
    }
    /// Get current step.
    pub fn step(&self) -> SecondTimeStep {
        self.step
    }
    /// Get current offset.
    pub fn offset(&self) -> SecondTimeStep {
        self.offset
    }
    /// Get current pattern.
    pub fn pattern(&self) -> Rc<RefCell<dyn Pattern>> {
        self.pattern.clone()
    }

    /// Apply the given second offset to all events.
    pub fn with_offset<O: Into<Option<SecondTimeStep>>>(&self, offset: O) -> Self {
        let offset = offset.into().unwrap_or(0.0);
        let event_iter_sample_time = self.time_base.samples_per_second() as f64 * offset;
        Self {
            offset,
            pattern: self.pattern.clone(),
            event_iter: self.event_iter.clone(),
            event_iter_sample_time,
            ..*self
        }
    }

    /// Use the given instrument for all note events which have no instrument set
    pub fn with_instrument<I: Into<Option<InstrumentId>>>(&self, instrument: I) -> Self {
        let instrument = instrument.into();
        Self {
            instrument,
            pattern: self.pattern.clone(),
            event_iter: self.event_iter.clone(),
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
            event_iter: self.event_iter.clone(),
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
            pattern: self.pattern.clone(),
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

impl Iterator for SecondTimeRhythm {
    type Item = (SampleTime, Option<Event>);

    fn next(&mut self) -> Option<Self::Item> {
        let mut pattern = self.pattern.borrow_mut();
        if pattern.is_empty() {
            None
        } else {
            let sample_time = self.event_iter_sample_time as SampleTime + self.sample_offset;
            let value = if pattern.run() > 0.0 {
                let mut event_iter = self.event_iter.borrow_mut();
                Some((
                    sample_time,
                    self.event_with_default_instrument(event_iter.next()),
                ))
            } else {
                Some((sample_time, None))
            };
            self.event_iter_sample_time += self.samples_per_step();
            value
        }
    }
}

impl Rhythm for SecondTimeRhythm {
    fn time_display(&self) -> Box<dyn SampleTimeDisplay> {
        Box::new(self.time_base)
    }

    fn set_time_base(&mut self, beat_time_base: BeatTimeBase) {
        self.time_base = beat_time_base.into();
    }

    fn samples_per_step(&self) -> f64 {
        self.step * self.time_base.samples_per_second() as f64
    }
    fn pattern_length(&self) -> usize {
        self.pattern.borrow().len()
    }

    fn sample_offset(&self) -> SampleTime {
        self.sample_offset
    }
    fn set_sample_offset(&mut self, sample_offset: SampleTime) {
        self.sample_offset = sample_offset
    }

    fn next_until_time(&mut self, sample_time: SampleTime) -> Option<(SampleTime, Option<Event>)> {
        let current_sample_time = self.event_iter_sample_time + self.sample_offset as f64;
        if (current_sample_time as SampleTime) < sample_time {
            self.next()
        } else {
            None
        }
    }

    fn reset(&mut self) {
        // reset sample offset
        self.sample_offset = 0;
        // reset iterator state
        self.event_iter.borrow_mut().reset();
        self.event_iter_sample_time = self.time_base.samples_per_second() as f64 * self.offset;
        self.pattern.borrow_mut().reset();
    }
}
