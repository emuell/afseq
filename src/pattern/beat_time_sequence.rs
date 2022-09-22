use crate::{
    events::{PatternEvent, PatternEventIter},
    time::{BeatTimeBase, BeatTimeStep},
    Pattern, SampleTime,
};

// -------------------------------------------------------------------------------------------------

/// Emits `Some(PatternEvent)` every nth [`BeatTimeStep`]::Beat or Bar with a specified pattern.
pub struct BeatTimeSequencePattern {
    time_base: BeatTimeBase,
    step: BeatTimeStep,
    offset: BeatTimeStep,
    pattern: Vec<bool>,
    pos_in_pattern: usize,
    current_sample_time: f64,
    event_iter: Box<dyn PatternEventIter>,
}

impl BeatTimeSequencePattern {
    /// Create a new pattern based emitter which emits `value` every beat_time `step`.
    pub fn new<Value: PatternEventIter + 'static>(
        time_base: BeatTimeBase,
        step: BeatTimeStep,
        pattern: Vec<bool>,
        event_iter: Value,
    ) -> Self {
        let offset = BeatTimeStep::Beats(0);
        Self::new_with_offset(time_base, step, offset, pattern, event_iter)
    }

    /// Create a new pattern based emitter which emits `value` every beat_time `step`
    /// starting at the given beat_time `offset`.  
    pub fn new_with_offset<Value: PatternEventIter + 'static>(
        time_base: BeatTimeBase,
        step: BeatTimeStep,
        offset: BeatTimeStep,
        pattern: Vec<bool>,
        event_iter: Value,
    ) -> Self {
        let sample_time = offset.to_samples(&time_base);
        let pos_in_pattern = 0;
        Self {
            time_base,
            step,
            offset,
            pattern,
            pos_in_pattern,
            current_sample_time: sample_time,
            event_iter: Box::new(event_iter),
        }
    }
}

impl Iterator for BeatTimeSequencePattern {
    type Item = (SampleTime, Option<PatternEvent>);

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
        self.current_sample_time += self.step.to_samples(&self.time_base);
        // move pattern pos
        self.pos_in_pattern += 1;
        if self.pos_in_pattern >= self.pattern.len() {
            self.pos_in_pattern = 0;
        }
        // return current value
        value
    }
}

impl Pattern for BeatTimeSequencePattern {
    fn current_event(&self) -> &dyn PatternEventIter {
        &*self.event_iter
    }
    fn current_sample_time(&self) -> SampleTime {
        self.current_sample_time as SampleTime
    }
    fn reset(&mut self) {
        self.event_iter.reset();
        self.current_sample_time = self.offset.to_samples(&self.time_base);
        self.pos_in_pattern = 0;
    }
}
