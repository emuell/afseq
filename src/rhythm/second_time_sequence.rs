use crate::{
    event::{Event, EventIter},
    time::{SecondTimeBase, SecondTimeStep},
    Rhythm, SampleTime,
};

// -------------------------------------------------------------------------------------------------

/// Emits `Some(Event)` every nth [`SecondTimeStep`]::Beat or Bar with a specified pattern.
pub struct SecondTimeSequenceRhythm {
    time_base: SecondTimeBase,
    step: SecondTimeStep,
    offset: SecondTimeStep,
    pattern: Vec<bool>,
    pos_in_pattern: usize,
    current_sample_time: f64,
    event_iter: Box<dyn EventIter>,
}

impl SecondTimeSequenceRhythm {
    /// Create a new pattern based emitter which emits `value` every beat_time `step`.
    ///
    /// Param `pattern` is evaluated as an array of boolean -> when to trigger an event and when not,
    /// but can be specified as number array as well to notate things shorter: e.g. via \[0,1,1,1,0\].  
    pub fn new<Value: EventIter + 'static, const N: usize, T: Ord + Default>(
        time_base: SecondTimeBase,
        step: SecondTimeStep,
        pattern: [T; N],
        event_iter: Value,
    ) -> Self {
        let offset = 0.0;
        Self::new_with_offset(time_base, step, offset, pattern, event_iter)
    }

    /// Create a new pattern based emitter which emits `value` every beat_time `step`
    /// starting at the given beat_time `offset`.  
    ///
    /// See `new` on how to specify the `pattern` parameter.
    pub fn new_with_offset<Value: EventIter + 'static, const N: usize, T: Ord + Default>(
        time_base: SecondTimeBase,
        step: SecondTimeStep,
        offset: SecondTimeStep,
        pattern: [T; N],
        event_iter: Value,
    ) -> Self {
        let sample_time = time_base.seconds_to_samples(offset) as f64;
        let pos_in_pattern = 0;
        let pattern_vec = pattern
            .iter()
            .map(|f| *f != T::default())
            .collect::<Vec<bool>>();
        Self {
            time_base,
            step,
            offset,
            pattern: pattern_vec,
            pos_in_pattern,
            current_sample_time: sample_time,
            event_iter: Box::new(event_iter),
        }
    }
}

impl Iterator for SecondTimeSequenceRhythm {
    type Item = (SampleTime, Option<Event>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.pattern.is_empty() {
            return None;
        }
        // fetch current value
        let sample_time = self.current_sample_time as SampleTime;
        let value = if self.pattern[self.pos_in_pattern] {
            Some((sample_time, self.event_iter.next()))
        } else {
            Some((sample_time, None))
        };
        // move sample_time
        self.current_sample_time += self.time_base.seconds_to_samples(self.step) as f64;
        // move pattern pos
        self.pos_in_pattern += 1;
        if self.pos_in_pattern >= self.pattern.len() {
            self.pos_in_pattern = 0;
        }
        // return current value
        value
    }
}

impl Rhythm for SecondTimeSequenceRhythm {
    fn reset(&mut self) {
        self.event_iter.reset();
        self.current_sample_time = self.time_base.seconds_to_samples(self.offset) as f64;
        self.pos_in_pattern = 0;
    }
}
