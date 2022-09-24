use std::{cell::RefCell, rc::Rc};

use crate::{
    event::{empty::EmptyEventIter, Event, EventIter},
    time::{SecondTimeBase, SecondTimeStep, TimeBase},
    Rhythm, SampleTime,
};

// -------------------------------------------------------------------------------------------------

/// Emits `Option(Event)` every nth [`SecondTimeStep`] with an optional pattern and offset.
pub struct SecondTimeRhythm {
    time_base: SecondTimeBase,
    step: SecondTimeStep,
    offset: SecondTimeStep,
    pattern: Rc<Vec<bool>>,
    pattern_pos: usize,
    event_iter: Rc<RefCell<dyn EventIter>>,
    event_iter_sample_time: f64,
}

impl SecondTimeRhythm {
    /// Create a new pattern based emitter which emits `value` every beat_time `step`.
    pub fn new(time_base: SecondTimeBase, step: SecondTimeStep) -> Self {
        let offset = 0.0;
        let pattern = Rc::new(vec![true]);
        let pattern_pos = 0;
        let event_iter = Rc::new(RefCell::new(EmptyEventIter {}));
        let event_iter_sample_time = time_base.seconds_to_samples(offset) as f64;
        Self {
            time_base,
            step,
            offset,
            pattern,
            pattern_pos,
            event_iter,
            event_iter_sample_time,
        }
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

    /// Trigger events with the given pattern. Param `pattern` is evaluated as an array of boolean:
    /// when to trigger an event and when not, but can be specified as number array as well to notate
    /// things shorter: e.g. via \[0,1,1,1,0\].  
    pub fn with_pattern<const N: usize, T: Ord + Default>(&self, pattern: [T; N]) -> Self {
        let pattern_vec = pattern
            .iter()
            .map(|f| *f != T::default())
            .collect::<Vec<bool>>();
        Self {
            pattern: Rc::new(pattern_vec),
            event_iter: self.event_iter.clone(),
            ..*self
        }
    }

    /// Use the given [`EventIter`] to trigger events.
    pub fn trigger<Iter: EventIter + 'static>(&self, iter: Iter) -> Self {
        Self {
            pattern: self.pattern.clone(),
            event_iter: Rc::new(RefCell::new(iter)),
            ..*self
        }
    }
}

impl Iterator for SecondTimeRhythm {
    type Item = (SampleTime, Option<Event>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.pattern.is_empty() {
            return None;
        }
        // fetch current value
        let sample_time = self.event_iter_sample_time as SampleTime;
        let value = if self.pattern[self.pattern_pos] {
            Some((sample_time, self.event_iter.borrow_mut().next()))
        } else {
            Some((sample_time, None))
        };
        // move sample_time
        self.event_iter_sample_time += self.time_base.samples_per_second() as f64 * self.step;
        // move pattern pos
        self.pattern_pos += 1;
        if self.pattern_pos >= self.pattern.len() {
            self.pattern_pos = 0;
        }
        // return current value
        value
    }
}

impl Rhythm for SecondTimeRhythm {
    fn reset(&mut self) {
        self.event_iter.borrow_mut().reset();
        self.event_iter_sample_time = self.time_base.samples_per_second() as f64 * self.offset;
        self.pattern_pos = 0;
    }
}
