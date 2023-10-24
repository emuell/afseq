use std::{cell::RefCell, rc::Rc};

use crate::{
    event::{empty::EmptyEventIter, Event, EventIter},
    time::{BeatTimeBase, BeatTimeStep, SampleTimeDisplay},
    Rhythm, SampleTime,
};

use super::euclidian::euclidean;

// -------------------------------------------------------------------------------------------------

/// Emits `Option(Event)` every nth [`BeatTimeStep`] with an optional pattern and offset.
#[derive(Clone, Debug)]
pub struct BeatTimeRhythm {
    time_base: BeatTimeBase,
    step: BeatTimeStep,
    offset: BeatTimeStep,
    pattern: Rc<Vec<bool>>,
    pattern_pos: usize,
    event_iter: Rc<RefCell<dyn EventIter>>,
    event_iter_sample_time: f64,
    sample_offset: SampleTime,
}

impl BeatTimeRhythm {
    /// Create a new pattern based emitter which emits `value` every beat_time `step`.  
    pub fn new(time_base: BeatTimeBase, step: BeatTimeStep) -> Self {
        let offset = BeatTimeStep::Beats(0.0);
        let pattern = Rc::new(vec![true]);
        let pattern_pos = 0;
        let event_iter = Rc::new(RefCell::new(EmptyEventIter {}));
        let event_iter_sample_time = offset.to_samples(&time_base);
        let sample_offset = 0;
        Self {
            time_base,
            step,
            offset,
            pattern,
            pattern_pos,
            event_iter,
            event_iter_sample_time,
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
    pub fn pattern(&self) -> Vec<bool> {
        self.pattern.to_vec()
    }

    /// Trigger events with the given pattern. Param `pattern` is evaluated as an array of boolean:
    /// when to trigger an event and when not, but can be specified as number array as well to notate
    /// things shorter: e.g. via \[0,1,1,1,0\].  
    pub fn with_pattern_vector<T: Ord + Default>(&self, pattern: Vec<T>) -> Self {
        let pattern_vec = pattern
            .iter()
            .map(|f| *f != T::default())
            .collect::<Vec<_>>();
        Self {
            pattern: Rc::new(pattern_vec),
            event_iter: self.event_iter.clone(),
            ..*self
        }
    }
    pub fn with_pattern<const N: usize, T: Ord + Default + Clone>(&self, pattern: [T; N]) -> Self {
        self.with_pattern_vector(pattern.to_vec())
    }

    pub fn with_euclidian_pattern(&self, pulses: u32, steps: u32, offset: i32) -> Self {
        let pattern = euclidean(pulses, steps, offset);
        self.with_pattern_vector(pattern)
    }

    /// Apply the given beat-time step offset to all events.
    pub fn with_offset<O: Into<Option<BeatTimeStep>>>(&self, offset: O) -> Self {
        let offset = offset.into().unwrap_or(BeatTimeStep::Beats(0.0));
        let event_iter_sample_time = offset.to_samples(&self.time_base);
        Self {
            offset,
            pattern: self.pattern.clone(),
            event_iter: self.event_iter.clone(),
            event_iter_sample_time,
            ..*self
        }
    }
    /// Apply the given offset in out step's timing resolution to all events.
    pub fn with_offset_in_step(&self, offset: f32) -> Self {
        let mut offset_steps = self.step;
        offset_steps.set_steps(offset);
        self.with_offset(offset_steps)
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
}

impl Iterator for BeatTimeRhythm {
    type Item = (SampleTime, Option<Event>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.pattern.is_empty() {
            return None;
        }
        // fetch current value
        let sample_time = self.event_iter_sample_time as SampleTime + self.sample_offset;
        let value = if self.pattern[self.pattern_pos] {
            Some((sample_time, self.event_iter.borrow_mut().next()))
        } else {
            Some((sample_time, None))
        };
        // move sample_time
        self.event_iter_sample_time += self.step.to_samples(&self.time_base);
        // move pattern pos
        self.pattern_pos += 1;
        if self.pattern_pos >= self.pattern.len() {
            self.pattern_pos = 0;
        }
        // return current value
        value
    }
}

impl Rhythm for BeatTimeRhythm {
    fn time_display(&self) -> Box<dyn SampleTimeDisplay> {
        Box::new(self.time_base)
    }

    fn sample_offset(&self) -> SampleTime {
        self.sample_offset
    }
    fn set_sample_offset(&mut self, sample_offset: SampleTime) {
        self.sample_offset = sample_offset
    }

    fn reset(&mut self) {
        // reset sample offset
        self.sample_offset = 0;
        // reset iterator state
        self.event_iter.borrow_mut().reset();
        self.event_iter_sample_time = self.offset.to_samples(&self.time_base);
        self.pattern_pos = 0;
    }
}
